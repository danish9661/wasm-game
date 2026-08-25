use game::building::{BUILDABLE, CHEST_RANGE, Structure, StructureKind, try_build};
use game::iso::iso_to_world;
use game::combat::{
    ARROW_DAMAGE, SWING_DAMAGE, SWING_REACH, Arrow,
    arrow_hits, swing_hits,
};
use game::daynight::{DAY_LENGTH, START_TIME, clock, daylight as daylight_at, temperature};
use game::enemy::{AGGRO_RANGE, AiState, EnemyRegistry, EnemyKind, spawner_on};
use game::items::{Inventory, ItemKind};
use game::player::{self, Player};
use game::poi::{ruins_at, ruins_walls};
use game::quest::QuestLog;
use game::render::{self, Camera, Sprite, SpriteStyle, VERTEX_STRIDE_BYTES};
use game::iso::{HALF_H, HALF_W};
use game::resources::{NodeRegistry, ResourceKind, resource_on, HARVEST_RANGE};
use game::world::{ChunkCache, TileKind, WorldGen, tile_at, CHUNK_SIZE};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;

/// Crafting recipes unlocked at an Anvil. Each is (label, cost pairs). The
/// effect is applied by index in `craft`.
const CRAFT_RECIPES: &[(&str, &[(ItemKind, u32)])] = &[
    ("Honed Tools", &[(ItemKind::Wood, 5), (ItemKind::Stone, 3), (ItemKind::Gem, 1)]),
    ("Iron Plate", &[(ItemKind::Stone, 4), (ItemKind::Gem, 2)]),
    ("Healing Salve x3", &[(ItemKind::Herb, 3), (ItemKind::Food, 2)]),
    ("Cook Meal (Food x2)", &[(ItemKind::Herb, 2)]),
];

/// Console-only log for the GPU pipeline (does not touch the #log HUD element).
fn glog(msg: &str) {
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(msg));
}

/// Campfire point light slots (each = position/intensity vec4 + color vec4).
const MAX_LIGHTS: usize = 8;
const LIGHT_FLOATS: usize = MAX_LIGHTS * 8;

fn kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Tree => "Tree",
        ResourceKind::Bush => "Bush",
        ResourceKind::Rock => "Rock",
        ResourceKind::Mushroom => "Mushroom",
        ResourceKind::Crystal => "Crystal",
        ResourceKind::Flower => "Flower",
        ResourceKind::GrassTuft => "Grass",
        ResourceKind::Fern => "Fern",
        ResourceKind::Ore => "Ore",
    }
}

fn struct_name(kind: StructureKind) -> &'static str {
    match kind {
        StructureKind::Campfire => "F",
        StructureKind::Wall => "W",
        StructureKind::Chest => "C",
        StructureKind::Altar => "A",
        StructureKind::Fence => "f",
        StructureKind::Torch => "T",
        StructureKind::Anvil => "a",
        StructureKind::Bed => "B",
        StructureKind::Well => "O",
        StructureKind::Sign => "s",
        StructureKind::Barrel => "b",
        StructureKind::Totem => "t",
        StructureKind::RockPile => "r",
        StructureKind::Statue => "S",
        StructureKind::Lantern => "L",
        StructureKind::Brazier => "Z",
        StructureKind::Crate => "c",
        StructureKind::Pillar => "P",
        StructureKind::BonePile => "x",
        StructureKind::Cactus => "k",
        StructureKind::Vines => "v",
        StructureKind::Lilypad => "l",
        StructureKind::Reed => "d",
        StructureKind::Rubble => "u",
        StructureKind::RuinTower => "U",
        StructureKind::Spike => "+",
        StructureKind::FarmPlot => "*",
    }
}

fn enemy_name(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::Slime => "Slime",
        EnemyKind::Boss => "Warden",
        EnemyKind::Skeleton => "Skeleton",
        EnemyKind::Goblin => "Goblin",
        EnemyKind::Bat => "Bat",
        EnemyKind::Spider => "Spider",
        EnemyKind::Imp => "Imp",
        EnemyKind::Ogre => "Ogre",
        EnemyKind::Wraith => "Wraith",
        EnemyKind::Stoneslinger => "Stoneslinger",
        EnemyKind::Colossus => "Colossus",
        EnemyKind::Brute => "Brute",
        EnemyKind::Stormcaller => "Stormcaller",
    }
}

/// Color grade applied to the whole scene by time of day: a cool blue at night
/// warming to neutral at noon, with a golden bump at dawn/dusk (where daylight
/// sits in the mid-range). Returned as an RGB multiplier.
fn sky_tint(t: f32) -> [f32; 3] {
    let d = daylight_at(t).clamp(0.25, 1.0);
    let day = (d - 0.25) / 0.75; // 0 = night .. 1 = noon
    let cool = [0.82, 0.88, 1.06];
    let neutral = [1.0, 1.0, 1.0];
    let warm = [1.10, 0.95, 0.78];
    let mut base = [0.0f32; 3];
    for i in 0..3 {
        base[i] = cool[i] + (neutral[i] - cool[i]) * day;
    }
    let w = (4.0 * day * (1.0 - day)).clamp(0.0, 1.0); // peaks at dawn/dusk
    let mut tint = [0.0f32; 3];
    for i in 0..3 {
        tint[i] = base[i] + (warm[i] - base[i]) * w * 0.6;
    }
    tint
}

const SHADER: &str = r#"
struct Uniforms {
    viewport: vec2<f32>,
    daylight: f32,
    _pad: f32,
    tint: vec3<f32>,
    _pad2: f32,
    lights: array<vec4<f32>, 16>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) color: vec4<f32>) -> VsOut {
    var out: VsOut;
    let ndc = vec2<f32>(
        pos.x * 2.0 / u.viewport.x - 1.0,
        1.0 - pos.y * 2.0 / u.viewport.y,
    );
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var col = in.color;
    // global day/night: blend toward a dim blue night palette. The night floor
    // is kept clearly above the background clear color so the world stays
    // visible (and assets readable) even at deep night.
    let night = vec4<f32>(0.22, 0.25, 0.38, 1.0);
    let d = clamp(u.daylight, 0.25, 1.0);
    col = mix(night, col, d);
    // color grade: a cool tint at night warming to neutral at noon, with a
    // golden bump at dawn/dusk (where daylight is mid-range). Applied to the
    // base scene; point lights are added on top, untinted.
    col = vec4<f32>(col.rgb * u.tint, col.a);
    // campfire point lights: warm additive glow with soft falloff
    let sp = (in.pos.xy * 0.5 + 0.5) * u.viewport;
    for (var i = 0u; i < 8u; i++) {
        let lp = u.lights[i * 2u];
        if (lp.w <= 0.0) { continue; }
        let d = distance(sp, lp.xy);
        let fall = lp.z * exp(-d * d / (lp.w * lp.w));
        col += vec4<f32>(u.lights[i * 2u + 1u].rgb * fall, 0.0);
    }
    return vec4<f32>(col.rgb, col.a);
}
"#;

const VERTEX_STRIDE: u64 = VERTEX_STRIDE_BYTES as u64;

static READBACK: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
static READBACK_INFLIGHT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
// Bumped whenever the readback buffer is (re)allocated (resize). A pending map
// callback captures the generation it was started with; on completion it only
// clears INFLIGHT if the generation still matches, so a stale callback from a
// discarded buffer can't clear the flag for the new buffer (which would let us
// copy into a still-mapped buffer -> "used in submit while pending map").
static READBACK_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// Internal render/readback resolution cap. (0, 0) means "native" (no cap).
// Changed at runtime from the settings menu via set_render_cap(); read by
// resize(). Smaller = faster readback/blit in software (SwiftShader).
static RENDER_CAP: std::sync::Mutex<(u32, u32)> = std::sync::Mutex::new((640, 400));

pub fn set_render_cap(w: u32, h: u32) {
    *RENDER_CAP.lock().unwrap() = (w, h);
}

pub fn get_render_cap() -> (u32, u32) {
    *RENDER_CAP.lock().unwrap()
}

fn readback_from_data(data: &[u8], width: u32, height: u32, bytes_per_row: u32) -> String {
    if data.is_empty() {
        return String::from("empty readback");
    }
    let bytes_per_row = bytes_per_row as usize;
    let (mut r_acc, mut g_acc, mut b_acc) = (0u64, 0u64, 0u64);
    let mut distinct = std::collections::HashSet::new();
    let mut nonbg = 0u64;
    let mut samples = 0u64;
    for y in (0..height).step_by(2) {
        let row = y as usize * bytes_per_row;
        for x in (0..width).step_by(7) {
            let i = row + x as usize * 4;
            if i + 3 >= data.len() {
                continue;
            }
            let (b, g, r) = (data[i], data[i + 1], data[i + 2]);
            r_acc += r as u64;
            g_acc += g as u64;
            b_acc += b as u64;
            distinct.insert((r, g, b));
            if r + g + b > 48 {
                nonbg += 1;
            }
            samples += 1;
        }
    }
    let avg = |v: u64| (v / samples.max(1)) as f32 / 255.0;
    format!(
        "avg=({:.2},{:.2},{:.2}) distinct={} nonbg={:.2}% w={width} h={height}",
        avg(r_acc),
        avg(g_acc),
        avg(b_acc),
        distinct.len(),
        nonbg as f32 / samples.max(1) as f32 * 100.0,
    )
}

/// Copy an Rgba8Unorm (row-padded) readback into a visible 2D `<canvas id="blit">`.
/// Used as the display path when the WebGPU canvas can't be composited to the
/// screen (e.g. SwiftShader-Vulkan in headed Chrome: the surface renders fine
/// but the headed compositor never shows it). A 2D canvas always composites.
struct BlitCache {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    buf: Vec<u8>,
    w: u32,
    h: u32,
}

thread_local! {
    static BLIT_CACHE: std::cell::RefCell<Option<BlitCache>> = const { std::cell::RefCell::new(None) };
}

fn blit_to_2d_canvas(
    data: &[u8],
    width: u32,
    height: u32,
    bytes_per_row: u32,
    tod: f32,
    aclock: f32,
    weather: u8,
) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let doc = match window.document() {
        Some(d) => d,
        None => return,
    };
    BLIT_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        let stale = match cache.as_ref() {
            Some(x) => x.w != width || x.h != height,
            None => true,
        };
        if stale {
            let canvas = match doc
                .get_element_by_id("blit")
                .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
            {
                Some(c) => c,
                None => {
                    glog("[gfx] blit: #blit canvas not found");
                    *cache = None;
                    return;
                }
            };
            canvas.set_width(width);
            canvas.set_height(height);
            let ctx = match canvas
                .get_context("2d")
                .ok()
                .flatten()
                .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
            {
                Some(c) => c,
                None => {
                    glog("[gfx] blit: 2d context unavailable");
                    *cache = None;
                    return;
                }
            };
            *cache = Some(BlitCache {
                canvas,
                ctx,
                buf: Vec::with_capacity(width as usize * height as usize * 4),
                w: width,
                h: height,
            });
        }
        let cache = match cache.as_mut() {
            Some(x) => x,
            None => return,
        };
        let w = width as usize;
        let h = height as usize;
        let bpr = bytes_per_row as usize;
        cache.buf.clear();
        for y in 0..h {
            let src = y * bpr;
            let take = (w * 4).min(data.len().saturating_sub(src));
            cache.buf.extend_from_slice(&data[src..src + take]);
            if take < w * 4 {
                let pad = w * 4 - take;
                cache.buf.resize(cache.buf.len() + pad, 0);
            }
        }
        // Force opaque alpha so the page background can't show through as holes.
        for a in cache.buf.iter_mut().skip(3).step_by(4) {
            *a = 255;
        }
        // vignette: gently darken toward the edges to focus the eye and add
        // mood (computed on the unpadded CPU copy since the 2D gradient API
        // isn't available in this build).
        {
        let wf = w as f64;
        let hf = h as f64;
        let cx = wf * 0.5;
        let cy = hf * 0.5;
        // Atmospheric distance fog: the far (upper) screen fades toward the
        // horizon tint so the flat isometric ground gains depth. Pre-compute
        // the fog target colour (0..255) from the time-of-day sky tint.
        let fog = sky_tint(tod);
        let fog_r = (fog[0] * 0.8 + 0.2).clamp(0.0, 1.0) as f64 * 255.0;
        let fog_g = (fog[1] * 0.8 + 0.2).clamp(0.0, 1.0) as f64 * 255.0;
        let fog_b = (fog[2] * 0.8 + 0.2).clamp(0.0, 1.0) as f64 * 255.0;
        const FOG_MAX: f64 = 0.26;
        for y in 0..h {
            let ny = (y as f64 - cy) / cy;
            // fog strength: 0 at ~55% screen height (player band) rising to
            // FOG_MAX at the top (horizon).
            let y_norm = y as f64 / hf;
            let fog_t = (((0.55 - y_norm) / 0.55).clamp(0.0, 1.0) * FOG_MAX).clamp(0.0, 1.0);
            for x in 0..w {
                let nx = (x as f64 - cx) / cx;
                let d = (nx * nx + ny * ny).sqrt();
                let v = if d <= 0.6 {
                    1.0
                } else {
                    let t = ((d - 0.6) / 0.55).clamp(0.0, 1.0);
                    let s = t * t * (3.0 - 2.0 * t);
                    1.0 - s * 0.45
                };
                let i = (y * w + x) * 4;
                let mut r = cache.buf[i] as f64 * v;
                let mut g = cache.buf[i + 1] as f64 * v;
                let mut b = cache.buf[i + 2] as f64 * v;
                // blend toward fog colour
                r = r + (fog_r - r) * fog_t;
                g = g + (fog_g - g) * fog_t;
                b = b + (fog_b - b) * fog_t;
                cache.buf[i] = r as u8;
                cache.buf[i + 1] = g as u8;
                cache.buf[i + 2] = b as u8;
            }
        }
        }
        let clamped = Clamped(cache.buf.as_slice());
        match ImageData::new_with_u8_clamped_array_and_sh(clamped, width, height) {
            Ok(img) => {
                let _ = cache.ctx.put_image_data(&img, 0.0, 0.0);
            }
            Err(e) => {
                glog(&format!("[gfx] blit: ImageData error {e:?}"));
            }
        }
        // ---- atmosphere overlay (2D canvas, drawn over the WebGPU readback) ----
        let ctx = &cache.ctx;
        let w = width as f64;
        let h = height as f64;
        // drifting motes: warm fireflies at night, pale pollen by day
        let day = daylight_at(tod).clamp(0.25_f32, 1.0_f32) as f64;
        let night = 1.0 - day;
        for i in 0..22u32 {
            let fi = i as f64;
            let r1 = (fi * 12.9898).sin().fract().abs();
            let r2 = (fi * 78.233).sin().fract().abs();
            let r3 = (fi * 37.719).sin().fract().abs();
            let speed = 4.0 + r3 * 10.0;
            let x = (((r1 * w + aclock as f64 * speed * (0.3 + r2)) % w) + w) % w;
            let y = (((r2 * h + aclock as f64 * speed * 0.45) % h) + h) % h;
            let blink = 0.5 + 0.5 * (aclock as f64 * 2.0 + fi).sin();
            let (a, color) = if night > 0.35 {
                let a = (0.25 + 0.55 * blink) * night;
                (a, format!("rgba(255,228,120,{a:.3})"))
            } else {
                let a = 0.10 * day * (0.5 + 0.5 * blink);
                (a, format!("rgba(245,245,210,{a:.3})"))
            };
            let radius = 1.0 + r3 * 1.5;
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(&color));
            let _ = ctx.begin_path();
            let _ = ctx.arc(x, y, radius, 0.0, std::f64::consts::TAU);
            let _ = ctx.fill();
        }
        // weather: snow (2) or rain (1) — drifting particles + a faint veil
        if weather != 0 {
            let snow = weather == 2;
            let fall = (aclock as f64 * (if snow { 90.0 } else { 380.0 })) % h;
            if snow {
                ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(255,255,255,0.85)"));
                let cols = 110u32;
                for i in 0..cols {
                    let fi = i as f64;
                    let x = (((fi * 37.3 + aclock as f64 * 30.0) % w) + w) % w;
                    let y = (((fi * 19.7 + fall) % h) + h) % h;
                    let r = 1.0 + (fi * 7.0).fract() * 1.6;
                    ctx.begin_path();
                    ctx.arc(x, y, r, 0.0, std::f64::consts::PI * 2.0);
                    ctx.fill();
                }
                ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(200,215,235,0.08)"));
                ctx.fill_rect(0.0, 0.0, w, h);
            } else {
                ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(170,200,230,0.35)"));
                ctx.set_line_width(1.0);
                let cols = 90u32;
                for i in 0..cols {
                    let fi = i as f64;
                    let x = (((fi * 53.7 + aclock as f64 * 120.0) % w) + w) % w;
                    let y = (((fi * 29.3 + fall) % h) + h) % h;
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x - 4.0, y + 14.0);
                    ctx.stroke();
                }
                ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(120,140,170,0.10)"));
                ctx.fill_rect(0.0, 0.0, w, h);
            }
        }
        // Night vignette: darken the edges when the sun is down, so campfires
        // and lanterns read as the only light sources.
        let night = ((tod - 0.5).abs() * 2.0).min(1.0);
        if night > 0.05 {
            if let Ok(grad) = ctx.create_radial_gradient(
                w / 2.0,
                h / 2.0,
                (w.min(h)) * 0.25,
                w / 2.0,
                h / 2.0,
                (w.max(h)) * 0.75,
            ) {
                let _ = grad.add_color_stop(0.0, "rgba(0,0,10,0)");
                let _ = grad.add_color_stop(1.0, &format!("rgba(0,0,12,{})", 0.55 * night));
                ctx.set_fill_style(grad.as_ref());
                ctx.fill_rect(0.0, 0.0, w, h);
            }
        }
    });
}

struct VertexBuffer {
    buffer: wgpu::Buffer,
    capacity: u32,
}

/// Short-lived visual particle (death puff, hit spark). Rendered as a fading
/// Generic quad; it lives entirely on the render side (no game state).
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    size: f32,
    color: [f32; 3],
}

impl VertexBuffer {
    fn new(device: &wgpu::Device, capacity: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tile_vertices"),
            size: capacity as u64 * VERTEX_STRIDE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self { buffer, capacity }
    }

    fn upload(&self, queue: &wgpu::Queue, data: &[f32]) {
        assert!((data.len() as u32) <= self.capacity * 6, "vertex buffer overflow");
        queue.write_buffer(&self.buffer, 0, bytemuck_cast(data));
    }
}

fn bytemuck_cast(data: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
}

/// Toggle which canvas the page shows. "gpu" = present via the WebGPU canvas
/// (#game) directly; "blit" = show the read-back 2D copy (#blit). Used to hide
/// the now-blank #blit once we know the WebGPU surface composites fine.
fn set_backend(mode: &str) {
    let _ = js_sys::eval(&format!("window.setBackend && window.setBackend('{mode}')"));
}

pub struct App {
    canvas: HtmlCanvasElement,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    offscreen: wgpu::Texture,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: VertexBuffer,
    viewport: [f32; 2],
    camera: Camera,
    /// Smoothed player velocity (tiles/sec) used for camera look-ahead.
    last_px: f32,
    last_py: f32,
    cam_lead: (f32, f32),
    keys: [bool; 4],
    world: WorldGen,
    world_seed: u32,
    cur_biome: TileKind,
    chunks: ChunkCache,
    /// Cached visible-tile list, recomputed only when the camera or viewport
    /// changes (avoids recomputing + re-sorting ~2400 tiles 3×/frame).
    visible_cache: (i32, i32, i32, i32, Vec<(i32, i32)>),
    player: Player,
    inventory: Inventory,
    nodes: NodeRegistry,
    structures: Vec<Structure>,
    enemies: EnemyRegistry,
    arrows: Vec<Arrow>,
    quest: QuestLog,
    ruins: (i32, i32),
    opened_chests: std::collections::HashSet<(i32, i32)>,
    slimes_killed: u32,
    boss_killed: u32,
    colossus_killed: u32,
    boss_spawned: bool,
    altar_placed: bool,
    altar_tile: Option<(i32, i32)>,
    near_altar: bool,
    ending_pending: bool,
    /// 0 = Reign, 1 = Shatter, None = campaign not finished.
    ending: Option<u8>,
    ng_plus: u32,
    spawn_point: (f32, f32),
    time_of_day: f32,
    anim_clock: f32,
    respawn_timer: f32,
    debug_swing_hits: u32,
    debug_attacks: u32,
    debug_shots: u32,
    vertices: Vec<f32>,
    quad_count: u32,
    frames: u64,
    player_in_mesh: bool,
    readback_buffer: Option<wgpu::Buffer>,
    capture_requested: bool,
    using_blit: bool,
    backend_mode: u8,
    /// Authoritative frames-per-second, measured from actual sim steps.
    fps: f32,
    fps_acc: u32,
    fps_time: f32,
    /// Player's current movement speed in tiles/second (0 while idle).
    speed: f32,
    prev_px: f32,
    prev_py: f32,
    /// True while the persistent readback buffer is mapped/in-flight; lets us
    /// skip a frame's readback instead of allocating a fresh buffer.
    readback_busy: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Diagnostic: count of keydown events received and the last key code.
    key_evt: u32,
    key_dbg: String,
    /// Transient visual particles (death puffs, hit sparks).
    particles: Vec<Particle>,
    /// Cursor position in internal canvas pixels (for build ghost placement).
    mouse_screen: Option<(f32, f32)>,
    /// Active build mode: the structure the player is placing (ghost preview).
    build_mode: Option<StructureKind>,
    /// Cached ghost preview: (kind, tx, ty, valid) for the hovered tile.
    build_ghost: Option<(StructureKind, i32, i32, bool)>,
    /// Crafting bonuses unlocked at an Anvil.
    craft_harvest: u32,
    craft_armor: f32,
    /// Healing salves crafted at an Anvil (consumed with the R key).
    salves: u32,
    /// Enemy kinds the player has seen (for the Bestiary / Codex panel).
    discovered: std::collections::HashSet<EnemyKind>,
    /// Weather state: 0 = clear, 1 = rain, 2 = snow. Drives the visual effect.
    weather: u8,
    /// Seconds until the weather may change again.
    weather_timer: f32,
    /// Farm plots: seconds remaining until each planted plot is ready to
    /// harvest again (keyed by tile). Plots not present are grown (0).
    farm_cd: std::collections::HashMap<(i32, i32), f32>,
    /// True once the player has crafted Iron Plate (used by the quest log).
    crafted_iron: bool,
}

impl App {
    pub fn quad_count(&self) -> u32 {
        self.quad_count
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }

    pub fn player_x(&self) -> f32 {
        self.player.x
    }

    pub fn player_y(&self) -> f32 {
        self.player.y
    }

    pub fn player_in_mesh(&self) -> bool {
        self.player_in_mesh
    }

    /// JSON payload for the Inventory & Crafting / Build panel:
    /// resource counts, every buildable recipe (with cost + affordability),
    /// and the current build-mode selection.
    pub fn ui_data(&self) -> String {
        let recipes: Vec<_> = BUILDABLE
            .iter()
            .map(|(kind, key, label)| {
                let cost: Vec<_> = kind
                    .cost()
                    .iter()
                    .map(|(item, n)| serde_json::json!([item.name(), n]))
                    .collect();
                let afford = kind
                    .cost()
                    .iter()
                    .all(|(item, n)| self.inventory.count(*item) >= *n);
                serde_json::json!({
                    "key": key,
                    "label": label,
                    "cost": cost,
                    "afford": afford,
                    "blocks": kind.blocks_movement(),
                    "light": kind.emits_light(),
                })
            })
            .collect();
        let selected = self
            .build_mode
            .and_then(|k| BUILDABLE.iter().find(|(bk, _, _)| *bk == k).map(|(_, _, l)| *l))
            .unwrap_or("");
        let has_anvil = self.has_anvil();
        let crafts: Vec<_> = CRAFT_RECIPES
            .iter()
            .enumerate()
            .map(|(i, (label, cost))| {
                let cost_json: Vec<_> = cost
                    .iter()
                    .map(|(item, n)| serde_json::json!([item.name(), n]))
                    .collect();
                let afford = cost.iter().all(|(item, n)| self.inventory.count(*item) >= *n);
                serde_json::json!({
                    "idx": i,
                    "label": label,
                    "cost": cost_json,
                    "afford": afford,
                })
            })
            .collect();
        serde_json::json!({
            "inv": {
                "wood": self.inventory.count(ItemKind::Wood),
                "stone": self.inventory.count(ItemKind::Stone),
                "food": self.inventory.count(ItemKind::Food),
                "herb": self.inventory.count(ItemKind::Herb),
                "gem": self.inventory.count(ItemKind::Gem),
                "fragment": self.inventory.count(ItemKind::Fragment),
            },
            "recipes": recipes,
            "buildMode": self.build_mode.is_some(),
            "selected": selected,
            "hasAnvil": has_anvil,
            "nearAnvil": self.near_anvil(),
            "salves": self.salves,
            "crafts": crafts,
        })
        .to_string()
    }

    /// Top-down minimap centered on the player: a grid of packed RGB terrain
    /// colors plus markers for enemies and player-built structures.
    pub fn minimap_data(&mut self) -> String {
        const N: i32 = 33;
        const R: i32 = N / 2;
        let ptx = self.player.x.floor() as i32;
        let pty = self.player.y.floor() as i32;
        let mut cells: Vec<u32> = Vec::with_capacity((N * N) as usize);
        for dy in -R..=R {
            for dx in -R..=R {
                let kind = tile_at(&self.world, &mut self.chunks, ptx + dx, pty + dy);
                let c = kind.color();
                let r = (c[0] * 255.0) as u32;
                let g = (c[1] * 255.0) as u32;
                let b = (c[2] * 255.0) as u32;
                cells.push((r << 16) | (g << 8) | b);
            }
        }
        let enemies: Vec<[f32; 2]> = self.enemies.enemies().map(|e| [e.x, e.y]).collect();
        let structs: Vec<(i32, i32, &str)> = self
            .structures
            .iter()
            .map(|s| (s.tx, s.ty, struct_name(s.kind)))
            .collect();
        // Biome legend: name + packed RGB, so the HUD can draw a key.
        let legend: Vec<serde_json::Value> = [
            TileKind::DeepWater,
            TileKind::Water,
            TileKind::ShallowWater,
            TileKind::Sand,
            TileKind::Grass,
            TileKind::Forest,
            TileKind::Swamp,
            TileKind::Snow,
            TileKind::Stone,
            TileKind::Tundra,
            TileKind::Desert,
        ]
        .iter()
        .map(|k| {
            let c = k.color();
            let rgb = ((c[0] * 255.0) as u32) << 16 | ((c[1] * 255.0) as u32) << 8 | ((c[2] * 255.0) as u32);
            serde_json::json!({ "name": format!("{:?}", k), "color": rgb })
        })
        .collect();
        serde_json::json!({
            "n": N,
            "cells": cells,
            "player": [self.player.x, self.player.y],
            "enemies": enemies,
            "structs": structs,
            "legend": legend,
        })
        .to_string()
    }

    /// Bestiary / Codex: every enemy kind the player has discovered so far,
    /// with its stats and behaviour. Returns a JSON array of objects.
    pub fn codex(&self) -> String {
        let mut kinds: Vec<EnemyKind> = self.discovered.iter().copied().collect();
        kinds.sort_by_key(|k| k.name());
        let entries: Vec<_> = kinds
            .iter()
            .map(|k| {
                serde_json::json!({
                    "name": k.name(),
                    "boss": k.is_boss(),
                    "flying": k.flying(),
                    "ranged": k.ranged(),
                    "hp": k.max_hp(),
                    "dmg": k.damage(),
                    "behavior": k.behavior(),
                    "drops": k.drops().iter().map(|i| i.name()).collect::<Vec<_>>(),
                })
            })
            .collect();
        serde_json::json!({
            "total": entries.len(),
            "discovered": entries,
        })
        .to_string()
    }

    /// Machine-readable game state for the JS HUD / test harness.
    pub fn stats_line(&mut self) -> String {
        let near = match self.nearest_resource() {
            Some((tx, ty, kind)) => format!("{}@({tx},{ty})", kind_name(kind)),
            None => String::from("none"),
        };
        let mob = self.nearest_enemy();
        let pack = self
            .enemies
            .enemies()
            .map(|e| format!("({:.1},{:.1})", e.x, e.y))
            .collect::<Vec<_>>()
            .join(",");
        let structs = self
            .structures
            .iter()
            .map(|s| format!("{}{}@({},{})", struct_name(s.kind), s.kind.blocks_movement() as u8, s.tx, s.ty))
            .collect::<Vec<_>>()
            .join(";");
        let boss_hp = self
            .enemies
            .enemies()
            .find(|e| e.kind == EnemyKind::Boss)
            .map(|e| (e.hp / e.kind.max_hp() * 100.0) as u32)
            .unwrap_or(0);
        let ending_str = match self.ending {
            None => "none",
            Some(0) => "reign",
            Some(1) => "shatter",
            Some(2) => "twin",
            Some(_) => "unknown",
        };
        format!(
            "quads={} frames={} player=({:.1},{:.1}) hp={:.0} hunger={:.0} stamina={:.0} alive={} inv=(w{},s{},f{},h{},g{}) structures={} structs={} mobs={} mob={} pack={} swings={} atk={} shots={} quest=S{} ruins=({},{}) chest={} time={}             near={} boss={} colossus={} frag={} altar={} nearaltar={} nearAnvil={} ending={} weather={} ng={} seed={} biome={:?} bosshp={} altartile={} fps={:.0} spd={:.2} kev={} klast={}",
            self.quad_count(),
            self.frames(),
            self.player_x(),
            self.player_y(),
            self.player.hp,
            self.player.hunger,
            self.player.stamina,
            self.player.alive as u8,
            self.inventory.count(ItemKind::Wood),
            self.inventory.count(ItemKind::Stone),
            self.inventory.count(ItemKind::Food),
            self.inventory.count(ItemKind::Herb),
            self.inventory.count(ItemKind::Gem),
            self.structures.len(),
            structs,
            self.enemies.count(),
            mob,
            pack,
            self.debug_swing_hits,
            self.debug_attacks,
            self.debug_shots,
            self.quest.stage,
            self.ruins.0,
            self.ruins.1,
            self.opened_chests.contains(&self.ruins) as u8,
            clock(self.time_of_day),
            near,
            self.boss_spawned as u8,
            self.colossus_killed,
            self.inventory.count(ItemKind::Fragment),
            self.altar_placed as u8,
            self.near_altar as u8,
            self.near_anvil() as u8,
            ending_str,
            self.weather,
            self.ng_plus,
            self.world_seed,
            self.cur_biome,
            boss_hp,
            self.altar_tile
                .map(|(ax, ay)| format!("({ax},{ay})"))
                .unwrap_or_else(|| "none".to_string()),
            self.fps,
            self.speed,
            self.key_evt,
            self.key_dbg,
        )
    }

    /// Nearest alive enemy within aggro range, as "Kind@(tx,ty)" or "none".
    fn nearest_enemy(&self) -> String {
        let px = self.player.x;
        let py = self.player.y;
        let mut best: Option<(f32, i32, i32, &'static str)> = None;
        for e in self.enemies.enemies() {
            let d = (e.x - px).abs().max((e.y - py).abs());
            if d <= AGGRO_RANGE && best.map_or(true, |b| d < b.0) {
                best = Some((d, e.x.floor() as i32, e.y.floor() as i32, enemy_name(e.kind)));
            }
        }
        match best {
            Some((_, tx, ty, kind)) => format!("{kind}@({tx},{ty})"),
            None => String::from("none"),
        }
    }

    pub fn fps_of(dt: f32) -> f32 {
        (1.0 / dt.max(0.0001)).min(999.0)
    }

    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        glog("[gfx] Instance::new");
        // IMPORTANT: size the canvas BEFORE obtaining the WebGPU context.
        // Resizing a canvas (setting width/height) AFTER getContext('webgpu')
        // unconfigures it on real GPUs, which leaves the surface blank/black
        // even though draws are issued. SwiftShader tolerates the bad order;
        // hardware adapters do not.
        let (width, height) = resize_canvas(&canvas);
        glog(&format!("[gfx] canvas backing size = {width}x{height} (pre-context)"));
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|e| format!("create_surface: {e}"))?;
        glog("[gfx] surface created (Canvas target)");

        let fallback_only = web_sys::window()
            .and_then(|w| w.get("__adapter"))
            .and_then(|v| v.as_string())
            .map(|s| s == "sw")
            .unwrap_or(false);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: if fallback_only {
                    wgpu::PowerPreference::LowPower
                } else {
                    wgpu::PowerPreference::HighPerformance
                },
                force_fallback_adapter: fallback_only,
                compatible_surface: if fallback_only {
                    None
                } else {
                    Some(&surface)
                },
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| format!("request_adapter: {e}"))?;
        glog(&format!("[gfx] adapter obtained (fallback_only={fallback_only})"));

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("game_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .map_err(|e| format!("request_device: {e}"))?;
        glog("[gfx] device + queue obtained");
        device.on_uncaptured_error(std::sync::Arc::new(|e| {
            web_sys::console::error_1(&format!("[gfx] UNCAPTURED GPU ERROR: {e}").into());
        }));

        let format = surface.get_capabilities(&adapter).formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        glog(&format!(
            "[gfx] surface configured: {width}x{height} format={format:?} alpha=Opaque present=Fifo"
        ));
        let offscreen = create_offscreen(&device, format, width, height);

        let (uniform_buffer, bind_group) = create_uniforms(&device);
        let pipeline = create_pipeline(&device, format);
        let vertex_buffer = VertexBuffer::new(&device, 128 * 1024);
        glog("[gfx] App::new complete — entering render loop");

        let world = WorldGen::new(1337);
        let mut chunks = ChunkCache::new(256);
        let (px, py) = player::find_spawn(&world, &mut chunks);
        let ruins = ruins_at(1337, |tx, ty| tile_at(&world, &mut chunks, tx, ty).walkable());
        let mut structures = Vec::new();
        structures.push(Structure { tx: ruins.0, ty: ruins.1, kind: StructureKind::Chest });
        for (wx, wy) in ruins_walls(ruins.0, ruins.1) {
            structures.push(Structure { tx: wx, ty: wy, kind: StructureKind::Wall });
        }

        Ok(Self {
            canvas,
            surface,
            device,
            queue,
            config,
            offscreen,
            pipeline,
            uniform_buffer,
            bind_group,
            vertex_buffer,
            viewport: [width as f32, height as f32],
            camera: Camera::new(0.0, 0.0),
            last_px: px,
            last_py: py,
            cam_lead: (0.0, 0.0),
            keys: [false; 4],
            world,
            world_seed: 1337,
            cur_biome: TileKind::Grass,
            chunks,
            visible_cache: (i32::MIN, i32::MIN, i32::MIN, i32::MIN, Vec::new()),
            player: Player::new(px, py),
            inventory: Inventory::new(),
            nodes: NodeRegistry::new(),
            structures,
            enemies: EnemyRegistry::new(),
            arrows: Vec::new(),
            quest: QuestLog::new(),
            ruins,
            opened_chests: std::collections::HashSet::new(),
            slimes_killed: 0,
            boss_killed: 0,
            colossus_killed: 0,
            boss_spawned: false,
            altar_placed: false,
            altar_tile: None,
            near_altar: false,
            ending_pending: false,
            ending: None,
            ng_plus: 0,
            spawn_point: (px, py),
            time_of_day: START_TIME,
            anim_clock: 0.0,
            respawn_timer: 0.0,
            debug_swing_hits: 0,
            debug_attacks: 0,
            debug_shots: 0,
            vertices: Vec::with_capacity(64 * 1024 * 6),
            quad_count: 0,
            frames: 0,
            player_in_mesh: false,
            readback_buffer: None,
            capture_requested: false,
            using_blit: false,
            backend_mode: 0,
            fps: 0.0,
            fps_acc: 0,
            fps_time: 0.0,
            speed: 0.0,
            prev_px: 0.0,
            prev_py: 0.0,
            readback_busy: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            key_evt: 0,
            key_dbg: String::new(),
            particles: Vec::new(),
            mouse_screen: None,
            build_mode: None,
            build_ghost: None,
            craft_harvest: 0,
            craft_armor: 0.0,
            salves: 0,
            discovered: std::collections::HashSet::new(),
            weather: 0,
            weather_timer: 25.0,
            farm_cd: std::collections::HashMap::new(),
            crafted_iron: false,
        })
    }

    pub fn set_key(&mut self, code: &str, down: bool) {
        self.key_evt += 1;
        self.key_dbg = code.to_string();
        let mv = match code {
            "KeyW" | "ArrowUp" => Some(0),
            "KeyS" | "ArrowDown" => Some(1),
            "KeyA" | "ArrowLeft" => Some(2),
            "KeyD" | "ArrowRight" => Some(3),
            _ => None,
        };
        if let Some(idx) = mv {
            self.keys[idx] = down;
            return;
        }
        if down {
            match code {
                "KeyE" => self.harvest(),
                "KeyF" => self.build(StructureKind::Campfire),
                "KeyV" => self.build(StructureKind::Wall),
                "KeyT" => self.build(StructureKind::Torch),
                "KeyG" => self.build(StructureKind::Fence),
                "KeyB" => self.build(StructureKind::Bed),
                "KeyN" => self.build(StructureKind::Anvil),
                "KeyH" => self.build(StructureKind::Well),
                "KeyX" => self.build(StructureKind::Spike),
                "KeyU" => self.build(StructureKind::FarmPlot),
                "KeyZ" => self.try_sleep(),
                "KeyJ" => self.swing(),
                "KeyK" => self.shoot_arrow(),
                "KeyC" => {
                    self.player.eat(&mut self.inventory);
                }
                "KeyR" => {
                    self.use_salve();
                }
                "Space" => {
                    self.dodge();
                }
                // Build mode: Q toggles it, Esc exits, 1-7 pick a structure.
                "KeyQ" => {
                    self.build_mode =
                        if self.build_mode.is_some() { None } else { Some(StructureKind::Campfire) };
                }
                "Escape" => self.build_mode = None,
                "Digit1" => self.select_build(0),
                "Digit2" => self.select_build(1),
                "Digit3" => self.select_build(2),
                "Digit4" => self.select_build(3),
                "Digit5" => self.select_build(4),
                "Digit6" => self.select_build(5),
                "Digit7" => self.select_build(6),
                "Digit8" => self.select_build(7),
                "Digit9" => self.select_build(8),
                _ => {}
            }
        }
    }

    /// Set the cursor position in internal canvas pixels (for build ghost).
    pub fn set_mouse(&mut self, x: f32, y: f32) {
        self.mouse_screen = Some((x, y));
    }

    /// Toggle build mode on/off (true = on).
    pub fn set_build_mode(&mut self, on: bool) {
        self.build_mode = if on { Some(StructureKind::Campfire) } else { None };
    }

    /// Select a buildable structure by its index in `BUILDABLE`.
    pub fn select_build(&mut self, idx: usize) {
        if idx < BUILDABLE.len() {
            self.build_mode = Some(BUILDABLE[idx].0);
        }
    }

    /// Place the currently-selected build structure at the cursor ghost tile.
    pub fn place_selected(&mut self) {
        if let Some(kind) = self.build_mode {
            self.build(kind);
        }
    }

    /// True if the player has built an Anvil (required to craft).
    pub fn has_anvil(&self) -> bool {
        self.structures.iter().any(|s| s.kind == StructureKind::Anvil)
    }

    /// True when the player is standing within ~1.5 tiles of an Anvil, so the
    /// crafting-station prompt can be shown.
    pub fn near_anvil(&self) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        self.structures.iter().any(|s| {
            s.kind == StructureKind::Anvil
                && (s.tx as f32 + 0.5 - px).abs() <= 1.5
                && (s.ty as f32 + 0.5 - py).abs() <= 1.5
        })
    }

    /// Craft recipe `idx` at an Anvil. Returns false if no anvil, unaffordable,
    /// or out of range.
    pub fn craft(&mut self, idx: usize) -> bool {
        if !self.has_anvil() {
            return false;
        }
        let &(_, cost) = match CRAFT_RECIPES.get(idx) {
            Some(r) => r,
            None => return false,
        };
        if !cost.iter().all(|(k, n)| self.inventory.count(*k) >= *n) {
            return false;
        }
        for (k, n) in cost {
            self.inventory.remove(*k, *n);
        }
        match idx {
            0 => self.craft_harvest = (self.craft_harvest + 1).min(3),
            1 => {
                self.craft_armor = (self.craft_armor + 0.15).min(0.6);
                self.crafted_iron = true;
            }
            2 => self.salves += 3,
            3 => self.inventory.add(ItemKind::Food, 2),
            _ => {}
        }
        true
    }

    /// Consume a healing salve (R key): restores 40 HP if one is held.
    pub fn use_salve(&mut self) -> bool {
        if self.salves == 0 || self.player.hp >= player::MAX_HP {
            return false;
        }
        self.salves -= 1;
        self.player.hp = (self.player.hp + 40.0).min(player::MAX_HP);
        true
    }

    /// Trigger a dodge roll (Space): a brief speed burst with i-frames.
    pub fn dodge(&mut self) {
        if !self.player.alive {
            return;
        }
        let dir = player::input_dir(self.keys[0], self.keys[1], self.keys[2], self.keys[3]);
        self.player.try_dodge(dir);
    }

    /// Spawn `count` particles bursting from `(x, y)` in a ring, tinted `color`.
    fn spawn_particles(
        &mut self,
        x: f32,
        y: f32,
        color: [f32; 3],
        count: u32,
        speed: f32,
        life: f32,
        size: f32,
    ) {
        let base = (x * 12.9898 + y * 78.233).fract().abs() * std::f32::consts::TAU;
        for i in 0..count {
            let ang = base + (i as f32 / count.max(1) as f32) * std::f32::consts::TAU;
            let jitter = ((x * 3.1 + y * 1.7 + i as f32 * 0.37).fract().abs());
            let sp = speed * (0.45 + 0.55 * jitter);
            self.particles.push(Particle {
                x,
                y,
                vx: ang.cos() * sp,
                vy: ang.sin() * sp,
                life,
                max_life: life,
                size,
                color,
            });
        }
    }

    /// Melee swing in the facing direction: hits every enemy in the arc.
    fn swing(&mut self) {
        if !self.player.spend_stamina(6.0) {
            return;
        }
        self.debug_attacks += 1;
        let mut hits = swing_hits(
            &self.player,
            self.enemies.enemies_mut(),
            SWING_REACH,
        );
        self.debug_swing_hits += hits.len() as u32;
        let mut sparks = Vec::new();
        for e in &mut hits {
            e.take_damage(SWING_DAMAGE);
            sparks.push((e.x, e.y));
        }
        drop(hits);
        for (x, y) in sparks {
            self.spawn_particles(x, y, [1.0, 0.92, 0.62], 7, 55.0, 0.35, 3.5);
        }
        self.sweep_dead();
    }

    /// Resolve kills: drop loot and start respawn timers.
    fn sweep_dead(&mut self) {
        let drops: Vec<((i32, i32), EnemyKind, Vec<ItemKind>)> = self
            .enemies
            .iter_mut_with_key()
            .filter(|(_, e)| !e.alive())
            .map(|((tx, ty), e)| ((tx, ty), e.kind, e.drops()))
            .collect();
        for ((tx, ty), kind, items) in drops {
            for it in &items {
                self.inventory.add(*it, 1);
            }
            if !items.is_empty() {
            }
            match kind {
                EnemyKind::Slime => self.slimes_killed += 1,
                EnemyKind::Boss => {
                    self.boss_killed += 1;
                }
                EnemyKind::Colossus => {
                    self.colossus_killed += 1;
                }
                _ => {}
            }
            // bosses never respawn; slimes return after 15s
            let respawn = if matches!(kind, EnemyKind::Boss | EnemyKind::Colossus) { f32::MAX } else { 15.0 };
            self.enemies.kill(tx, ty, respawn);
        }
    }

    /// Shoot an arrow in the facing direction (costs stamina).
    fn shoot_arrow(&mut self) {
        if !self.player.spend_stamina(4.0) {
            return;
        }
        self.debug_shots += 1;
        self.arrows.push(Arrow::new(
            self.player.x,
            self.player.y,
            self.player.facing.0,
            self.player.facing.1,
        ));
    }

    fn harvest(&mut self) {
        // Reforge at the altar when carrying the fragment: this arms the
        // choice; the HUD shows Reign/Shatter and forwards the pick to reforge().
        if self.ending.is_none() {
            if let Some((ax, ay)) = self.altar_tile {
                let d = (self.player.x - (ax as f32 + 0.5))
                    .abs()
                    .max((self.player.y - (ay as f32 + 0.5)).abs());
                if d <= CHEST_RANGE && self.inventory.count(ItemKind::Fragment) > 0 {
                    self.ending_pending = true;
                    return;
                }
            }
        }
        if self.open_nearest_chest() {
            return;
        }
        // Harvest a ready farm plot if standing next to one.
        for s in self.structures.iter() {
            if s.kind == StructureKind::FarmPlot {
                let d = (self.player.x - (s.tx as f32 + 0.5))
                    .abs()
                    .max((self.player.y - (s.ty as f32 + 0.5)).abs());
                if d <= CHEST_RANGE {
                    let cd = self.farm_cd.entry((s.tx, s.ty)).or_insert(0.0);
                    if *cd <= 0.0 {
                        self.inventory.add(ItemKind::Food, 2);
                        *cd = 30.0;
                        return;
                    }
                }
            }
        }
        if let Some((tx, ty, kind)) = self.nearest_resource() {
            if let Some(item) = self.nodes.chop(tx, ty, kind) {
                // Honed Tools (crafted at an Anvil) yield bonus resources.
                self.inventory.add(item, 1 + self.craft_harvest);
            }
        }
    }

    /// Open a closed chest within reach: adds its loot once. Returns true
    /// if a chest was opened.
    fn open_nearest_chest(&mut self) -> bool {
        let mut best: Option<(f32, i32, i32)> = None;
        for s in &self.structures {
            if !s.kind.is_chest() || self.opened_chests.contains(&(s.tx, s.ty)) {
                continue;
            }
            let d = (self.player.x - (s.tx as f32 + 0.5))
                .abs()
                .max((self.player.y - (s.ty as f32 + 0.5)).abs());
            if d <= CHEST_RANGE && best.map_or(true, |b| d < b.0) {
                best = Some((d, s.tx, s.ty));
            }
        }
        if let Some((_, tx, ty)) = best {
            self.opened_chests.insert((tx, ty));
            self.inventory.add(ItemKind::Food, 2);
            self.inventory.add(ItemKind::Wood, 2);
            self.inventory.add(ItemKind::Stone, 1);
            true
        } else {
            false
        }
    }

    /// Can a structure of `kind` be placed on tile `(tx, ty)` right now?
    fn can_place(&mut self, kind: StructureKind, tx: i32, ty: i32) -> bool {
        if self.structures.iter().any(|s| s.tx == tx && s.ty == ty) {
            return false;
        }
        let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
        if !tile.walkable() {
            return false;
        }
        if resource_on(tx, ty, tile).is_some() && !self.nodes.is_depleted(tx, ty) {
            return false;
        }
        if kind.cost().iter().any(|(item, n)| self.inventory.count(*item) < *n) {
            return false;
        }
        true
    }

    /// Place `kind` at `(tx, ty)` if it is a legal spot (pays the cost).
    fn place(&mut self, kind: StructureKind, tx: i32, ty: i32) {
        if !self.can_place(kind, tx, ty) {
            return;
        }
        if let Ok(s) = try_build(kind, tx, ty, &mut self.inventory) {
            self.structures.push(s);
        }
    }

    /// Build a structure. In build mode the placement target is the hovered
    /// ghost tile (mouse); otherwise the structure is dropped on the player's
    /// own tile (the original hotkey behaviour).
    fn build(&mut self, kind: StructureKind) {
        if let Some((gk, gx, gy, valid)) = self.build_ghost {
            if gk == kind {
                if valid {
                    self.place(kind, gx, gy);
                }
                return;
            }
        }
        let tx = self.player.x.floor() as i32;
        let ty = self.player.y.floor() as i32;
        self.place(kind, tx, ty);
    }

    /// Sleep in a bed: skip the night and wake at dawn, restoring a little
    /// hunger and hp. Only works when standing next to a placed Bed.
    fn try_sleep(&mut self) {
        let near_bed = self
            .structures
            .iter()
            .any(|s| s.kind == StructureKind::Bed && {
                let dx = s.tx as f32 + 0.5 - self.player.x;
                let dy = s.ty as f32 + 0.5 - self.player.y;
                dx * dx + dy * dy < 4.0
            });
        if near_bed {
            self.time_of_day = 0.32; // wake ~07:40 with daylight climbing
            self.player.hunger = (self.player.hunger + 30.0).min(100.0);
            self.player.hp = (self.player.hp + 20.0).min(100.0);
        }
    }

    fn nearest_resource(&mut self) -> Option<(i32, i32, ResourceKind)> {
        let px = self.player.x;
        let py = self.player.y;
        let r = HARVEST_RANGE.ceil() as i32;
        let mut best: Option<(f32, i32, i32, ResourceKind)> = None;
        for ty in py.floor() as i32 - r..=py.floor() as i32 + r {
            for tx in px.floor() as i32 - r..=px.floor() as i32 + r {
                let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
                let Some(kind) = resource_on(tx, ty, tile) else {
                    continue;
                };
                if self.nodes.is_depleted(tx, ty) {
                    continue;
                }
                let d = (px - (tx as f32 + 0.5))
                    .abs()
                    .max((py - (ty as f32 + 0.5)).abs());
                if d <= HARVEST_RANGE && best.map_or(true, |b| d < b.0) {
                    best = Some((d, tx, ty, kind));
                }
            }
        }
        best.map(|(_, tx, ty, k)| (tx, ty, k))
    }

    /// Regenerate the world from `seed` and reset all run state for a clean
    /// run (used by both New Game+ and the Save/Load "New Game" path).
    fn reset_world(&mut self, seed: u32) {
        self.world_seed = seed;
        self.world = WorldGen::new(seed);
        self.chunks = ChunkCache::new(256);
        self.ruins = ruins_at(seed, |tx, ty| tile_at(&self.world, &mut self.chunks, tx, ty).walkable());
        let (px, py) = player::find_spawn(&self.world, &mut self.chunks);
        self.spawn_point = (px, py);
        self.player = Player::new(px, py);
        self.inventory = Inventory::new();
        self.nodes = NodeRegistry::new();
        self.arrows = Vec::new();
        self.enemies = EnemyRegistry::new();
        self.opened_chests = std::collections::HashSet::new();
        self.craft_harvest = 0;
        self.craft_armor = 0.0;
        self.salves = 0;
        let mut structures = Vec::new();
        structures.push(Structure { tx: self.ruins.0, ty: self.ruins.1, kind: StructureKind::Chest });
        for (wx, wy) in ruins_walls(self.ruins.0, self.ruins.1) {
            structures.push(Structure { tx: wx, ty: wy, kind: StructureKind::Wall });
        }
        self.structures = structures;
        self.quest = QuestLog::new();
        self.boss_killed = 0;
        self.colossus_killed = 0;
        self.discovered.clear();
        self.weather = 0;
        self.weather_timer = 25.0;
        self.boss_spawned = false;
        self.altar_placed = false;
        self.altar_tile = None;
        self.near_altar = false;
        self.ending_pending = false;
        self.time_of_day = START_TIME;
        self.respawn_timer = 0.0;
        self.build_mode = None;
        self.build_ghost = None;
        self.mouse_screen = None;
    }

    /// Reforge the Crown (campaign finale). `choice`: 0 = Reign (victory),
    /// 1 = Shatter (New Game+ with a harder, reseeded world). If the Colossus
    /// has also been defeated, a Reign reforge becomes the **true** ending
    /// (code 2, the Twin Star Crowns) — that is the hard gate on the second
    /// ending. Both start a fresh run with `ng_plus` incremented.
    pub fn reforge(&mut self, choice: u8) {
        if self.ending.is_some() || self.inventory.count(ItemKind::Fragment) == 0 {
            return;
        }
        let ending = if choice % 2 == 1 {
            1
        } else if self.colossus_killed >= 1 {
            2
        } else {
            0
        };
        self.ending = Some(ending);
        self.ng_plus += 1;
        let seed = 1338 + (self.ng_plus - 1);
        self.reset_world(seed);
    }

    /// Start a brand-new run with a randomly-chosen seed.
    pub fn new_game(&mut self) {
        self.ng_plus = 0;
        self.ending = None;
        let seed = {
            let mut s = 1337u32;
            if let Some(win) = web_sys::window() {
                if let Some(p) = win.performance() {
                    s = (p.now() as u32) ^ 0x5eed_1337;
                }
            }
            s
        };
        self.reset_world(seed);
        self.reset_run_state();
    }

    /// Start a brand-new run at a specific seed (player-entered world seed).
    pub fn new_game_with_seed(&mut self, seed: u32) {
        self.ng_plus = 0;
        self.ending = None;
        self.reset_world(seed);
        self.reset_run_state();
    }

    /// Reset transient per-run state that isn't rebuilt by `reset_world`.
    fn reset_run_state(&mut self) {
        self.farm_cd.clear();
        self.crafted_iron = false;
    }

    // ---- Save / Load ------------------------------------------------------

    pub fn to_save(&self) -> crate::save::SaveState {
        use game::items::ItemKind;
        let inv = [
            ItemKind::Wood,
            ItemKind::Stone,
            ItemKind::Food,
            ItemKind::Fragment,
            ItemKind::Herb,
            ItemKind::Gem,
        ]
        .iter()
            .map(|k| (*k, self.inventory.count(*k)))
            .collect();
        crate::save::SaveState {
            version: crate::save::CURRENT_SAVE_VERSION,
            world_seed: self.world_seed,
            player: crate::save::PlayerSave {
                x: self.player.x,
                y: self.player.y,
                hp: self.player.hp,
                hunger: self.player.hunger,
                stamina: self.player.stamina,
                facing: self.player.facing,
            },
            inv,
            structures: self.structures.clone(),
            opened_chests: self.opened_chests.iter().cloned().collect(),
            depleted_nodes: self.nodes.depleted_list(),
            enemies: self.enemies.enemies().map(|e| (e.kind, e.x, e.y, e.hp)).collect(),
            quest_stage: self.quest.stage,
            slimes_killed: self.slimes_killed,
            boss_killed: self.boss_killed,
            colossus_killed: self.colossus_killed,
            discovered: self.discovered.iter().cloned().collect(),
            boss_spawned: self.boss_spawned,
            altar_placed: self.altar_placed,
            altar_tile: self.altar_tile,
            ending_pending: self.ending_pending,
            ending: self.ending,
            ng_plus: self.ng_plus,
            time_of_day: self.time_of_day,
            spawn_point: self.spawn_point,
            craft_harvest: self.craft_harvest,
            craft_armor: self.craft_armor,
            salves: self.salves,
        }
    }

    pub fn apply_save(&mut self, s: &crate::save::SaveState) {
        use game::items::Inventory;
        use game::resources::NodeRegistry;
        self.reset_world(s.world_seed);
        // overlay the saved dynamic state
        self.player.x = s.player.x;
        self.player.y = s.player.y;
        self.player.hp = s.player.hp;
        self.player.hunger = s.player.hunger;
        self.player.stamina = s.player.stamina;
        self.player.facing = s.player.facing;

        self.inventory = Inventory::new();
        for (k, n) in &s.inv {
            self.inventory.add(*k, *n);
        }
        self.structures = s.structures.clone();
        self.opened_chests = s.opened_chests.iter().cloned().collect();
        self.reset_run_state();

        self.enemies = EnemyRegistry::new();
        for (kind, x, y, hp) in &s.enemies {
            let tx = x.floor() as i32;
            let ty = y.floor() as i32;
            if let Some(e) = self.enemies.get(tx, ty, *kind, 0.0) {
                e.x = *x;
                e.y = *y;
                e.hp = *hp;
                e.state = AiState::Idle;
                e.attack_timer = 0.0;
            }
        }

        self.quest = QuestLog::new();
        self.quest.stage = s.quest_stage;
        self.slimes_killed = s.slimes_killed;
        self.boss_killed = s.boss_killed;
        self.colossus_killed = s.colossus_killed;
        self.discovered = s.discovered.iter().copied().collect();
        self.boss_spawned = s.boss_spawned;
        self.altar_placed = s.altar_placed;
        self.altar_tile = s.altar_tile;
        self.near_altar = false;
        self.ending_pending = s.ending_pending;
        self.ending = s.ending;
        self.ng_plus = s.ng_plus;
        self.time_of_day = s.time_of_day;
        self.spawn_point = s.spawn_point;
        self.craft_harvest = s.craft_harvest;
        self.craft_armor = s.craft_armor;
        self.salves = s.salves;
        self.respawn_timer = 0.0;
        self.arrows = Vec::new();
        self.nodes = NodeRegistry::new();
        for (tx, ty, kind) in &s.depleted_nodes {
            self.nodes.restore_depleted(*tx, *ty, *kind);
        }
    }

    pub fn resize(&mut self) {
        let (mut width, mut height) = resize_canvas(&self.canvas);
        // Cap the internal render/readback resolution. The scene is rasterized
        // and read back in software (SwiftShader), so a large backing makes each
        // readback+blit very expensive and the #blit display lags behind the
        // simulation -> movement looks "very slow". Render at a smaller backing
        // and let CSS upscale it (pixelated). (0, 0) = native (no cap).
        let (cap_w, cap_h) = get_render_cap();
        if cap_w > 0 && cap_h > 0 {
            let scale = (cap_w as f64 / width as f64)
                .min(cap_h as f64 / height as f64)
                .min(1.0);
            if scale < 1.0 {
                width = (width as f64 * scale).max(1.0) as u32;
                height = (height as f64 * scale).max(1.0) as u32;
                self.canvas.set_width(width);
                self.canvas.set_height(height);
            }
        }
        glog(&format!("[gfx] resize -> {width}x{height} (cap {cap_w}x{cap_h})"));
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.offscreen = create_offscreen(&self.device, self.config.format, width, height);
        self.alloc_readback();
        self.viewport = [width as f32, height as f32];
        self.write_uniforms();
    }

    /// (Re)allocate the reusable GPU→CPU readback buffer for the current size.
    /// A single buffer is reused across frames; the old one is dropped.
    fn alloc_readback(&mut self) {
        let width = self.config.width;
        let height = self.config.height;
        let bytes_per_row = ((width * 4 + 255) / 256) * 256;
        self.readback_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: bytes_per_row as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));
        READBACK_GEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        READBACK_INFLIGHT.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    fn write_uniforms(&self) {
        let mut data = [0.0f32; 8 + LIGHT_FLOATS];
        data[0] = self.viewport[0];
        data[1] = self.viewport[1];
        data[2] = daylight_at(self.time_of_day);
        let tint = sky_tint(self.time_of_day);
        data[4] = tint[0];
        data[5] = tint[1];
        data[6] = tint[2];
        let lights = self.light_data();
        data[8..].copy_from_slice(&lights);
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck_cast(&data));
    }

    pub fn update(&mut self, dt: f32) {
        self.frames += 1;
        self.ensure_visible();
        self.time_of_day = (self.time_of_day + dt / DAY_LENGTH).rem_euclid(1.0);
        self.anim_clock = (self.anim_clock + dt).rem_euclid(3600.0);

        // Weather: periodically reconsider rain. Storms last ~20-40s; clear
        // spells ~25-45s. Cheap deterministic-ish roll from the clock.
        self.weather_timer -= dt;
        if self.weather_timer <= 0.0 {
            let r = (self.anim_clock * 7.0 + self.time_of_day * 311.0).fract();
            // Clear weather ends; otherwise roll a new storm (rain or snow).
            if self.weather != 0 {
                self.weather = 0;
                self.weather_timer = 25.0 + r * 20.0;
            } else if r < 0.30 {
                self.weather = if r < 0.15 { 1 } else { 2 };
                self.weather_timer = 20.0 + r * 20.0;
            } else {
                self.weather_timer = 25.0 + r * 20.0;
            }
        }

        // survival: hunger/stamina/temperature. Standing near any light source
        // (campfire/torch/lantern/brazier) counts as "warm" — slower hunger
        // drain and no starvation damage.
        let warm = self.player.alive
            && self.structures.iter().any(|s| {
                s.kind.emits_light()
                    && {
                        let dx = s.tx as f32 + 0.5 - self.player.x;
                        let dy = s.ty as f32 + 0.5 - self.player.y;
                        dx * dx + dy * dy < 2.5 * 2.5
                    }
            });
        // Tile under the player drives biome-specific survival + movement.
        let ptx = self.player.x.floor() as i32;
        let pty = self.player.y.floor() as i32;
        let biome = tile_at(&self.world, &mut self.chunks, ptx, pty);
        self.cur_biome = biome;
        if self.player.alive {
            let wet = self.weather == 1;
            self.player
                .tick(dt, temperature(self.time_of_day), warm, wet, biome);
            // Resting by a fire slowly mends wounds.
            if warm && self.player.hp < 100.0 {
                self.player.hp = (self.player.hp + dt * 3.0).min(100.0);
            }
        } else {
            self.respawn_timer -= dt;
            if self.respawn_timer <= 0.0 {
                self.player.respawn();
                // respawn at the actual waking tile (where the altar sits)
                self.player.x = self.spawn_point.0;
                self.player.y = self.spawn_point.1;
            }
        }

        // hydrate slime spawners on swamp tiles in view, then run AI
        for &(tx, ty) in &self.visible_cache.4 {
            let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
            if let Some(kind) = spawner_on(tx, ty, tile) {
                self.enemies.get(tx, ty, kind, dt);
            }
        }
        let px = self.player.x;
        let py = self.player.y;
        let mut contact: Option<f32> = None;
        for e in self.enemies.enemies_mut() {
            self.discovered.insert(e.kind);
            if let Some(dmg) = e.update((px, py), dt, |tx, ty| {
                !tile_at(&self.world, &mut self.chunks, tx, ty).walkable()
                    || self
                        .structures
                        .iter()
                        .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
            }) {
                contact = Some(dmg);
            }
            // Ranged enemies fire: turn the pending shot into an enemy arrow.
            if let Some((dx, dy)) = e.pending_shot.take() {
                self.arrows.push(Arrow::enemy(e.x, e.y, dx, dy));
            }
            // Spike traps: any enemy standing on a Spike tile takes continuous
            // damage (the player's defensive hazard).
            let etx = e.x.floor() as i32;
            let ety = e.y.floor() as i32;
            if self
                .structures
                .iter()
                .any(|s| s.tx == etx && s.ty == ety && s.kind == StructureKind::Spike)
            {
                e.take_damage(12.0 * dt);
            }
        }
        if let Some(dmg) = contact {
            // Iron Plate (crafted at an Anvil) reduces incoming damage.
            let dmg = dmg * (1.0 - self.craft_armor);
            self.player.take_damage(dmg);
        }
        self.sweep_dead();

        // Farm plots regrow their crops over time.
        const FARM_GROW: f32 = 30.0;
        for s in self.structures.iter() {
            if s.kind == StructureKind::FarmPlot {
                let cd = self.farm_cd.entry((s.tx, s.ty)).or_insert(0.0);
                *cd = (*cd - dt).max(0.0);
            }
        }

        // story beats: cheap facts from the session state
        let near_ruins = (self.player.x - (self.ruins.0 as f32 + 0.5))
            .abs()
            .max((self.player.y - (self.ruins.1 as f32 + 0.5)).abs())
            <= 4.0;

        // The Forest Warden spawns at the ruins once Chapter 1 is complete.
        if self.quest.stage >= 5 && !self.boss_spawned {
            self.enemies.get(self.ruins.0, self.ruins.1, EnemyKind::Boss, dt);
            self.boss_spawned = true;
        }
        // The Reforging Altar rises at the waking place once the fragment is taken.
        if self.quest.stage >= 6 && !self.altar_placed {
            let (ax, ay) = (
                self.spawn_point.0.floor() as i32,
                self.spawn_point.1.floor() as i32,
            );
            self.structures.push(Structure { tx: ax, ty: ay, kind: StructureKind::Altar });
            self.altar_placed = true;
            self.altar_tile = Some((ax, ay));
        }
        self.near_altar = self
            .altar_tile
            .map_or(false, |(ax, ay)| {
                (self.player.x - (ax as f32 + 0.5))
                    .abs()
                    .max((self.player.y - (ay as f32 + 0.5)).abs())
                    <= CHEST_RANGE
            });

        self.quest.update(
            self.inventory.count(ItemKind::Wood),
            self.inventory.count(ItemKind::Stone),
            self.structures
                .iter()
                .any(|s| s.kind == StructureKind::Wall),
            self.structures
                .iter()
                .any(|s| s.kind == StructureKind::Campfire),
            self.has_anvil(),
            self.crafted_iron,
            self.slimes_killed,
            near_ruins,
            self.opened_chests.contains(&self.ruins),
            self.boss_killed >= 1,
            self.inventory.count(ItemKind::Fragment) > 0,
            self.ending.is_some(),
            self.colossus_killed >= 1,
        );

        // arrows fly, hit, and expire (a hit removes the arrow)
        let mut hit_pos = Vec::new();
        self.arrows.retain_mut(|a| {
            if !a.step(dt) {
                return false;
            }
            if a.from_player {
                for (_key, e) in self.enemies.iter_mut_with_key() {
                    if arrow_hits(a, std::iter::once(&*e)).is_some() {
                        e.take_damage(ARROW_DAMAGE);
                        hit_pos.push((e.x, e.y));
                        return false;
                    }
                }
            } else {
                // enemy arrow: hits the player
                let dx = self.player.x - a.x;
                let dy = self.player.y - a.y;
                if dx * dx + dy * dy <= 0.8 * 0.8 {
                    self.player.take_damage(ARROW_DAMAGE * (1.0 - self.craft_armor));
                    hit_pos.push((a.x, a.y));
                    return false;
                }
            }
            true
        });
        for (x, y) in hit_pos {
            self.spawn_particles(x, y, [1.0, 0.92, 0.62], 5, 45.0, 0.35, 3.0);
        }
        self.sweep_dead();

        // integrate + cull particles
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vx *= 0.88;
            p.vy *= 0.88;
            p.life -= dt;
            p.size *= 0.95;
        }
        self.particles.retain(|p| p.life > 0.0);

        let dir = player::input_dir(self.keys[0], self.keys[1], self.keys[2], self.keys[3]);
        let bx = self.player.x;
        let by = self.player.y;
        // Wading through shallow water (rivers/streams) slows you down.
        let wade = tile_at(
            &self.world,
            &mut self.chunks,
            self.player.x.floor() as i32,
            self.player.y.floor() as i32,
        )
        .wadable();
        let move_dt = if wade { dt * 0.55 } else { dt };
        // Rough terrain under the player slows travel (snow especially).
        let speed_mul = match biome {
            TileKind::Snow => 0.7,
            TileKind::Swamp => 0.85,
            TileKind::Sand => 0.9,
            _ => 1.0,
        };
        // During a dodge roll, move in the dodge direction at boosted speed.
        let (move_dir, move_dt2) = if self.player.dodge_timer > 0.0 {
            (self.player.dodge_dir, move_dt * player::DODGE_BOOST)
        } else {
            (dir, move_dt)
        };
        player::move_player(&mut self.player, move_dir, move_dt2, speed_mul, |tx, ty| {
            !tile_at(&self.world, &mut self.chunks, tx, ty).walkable()
                || self
                    .structures
                    .iter()
                    .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
        });
        let moved = ((self.player.x - bx).powi(2) + (self.player.y - by).powi(2)).sqrt();
        self.speed = if dt > 0.0 { moved / dt } else { 0.0 };

        // authoritative FPS from real sim steps
        self.fps_acc += 1;
        self.fps_time += dt;
        if self.fps_time >= 0.5 {
            self.fps = self.fps_acc as f32 / self.fps_time;
            self.fps_acc = 0;
            self.fps_time = 0.0;
        }

        // camera look-ahead: lead the camera a little in the direction of
        // travel so the player sees what's coming, easing back to center.
        let vx = (self.player.x - self.last_px) / dt.max(1e-4);
        let vy = (self.player.y - self.last_py) / dt.max(1e-4);
        self.last_px = self.player.x;
        self.last_py = self.player.y;
        let lead_target = (vx * 0.35, vy * 0.35);
        let ka = (dt * 3.0).min(1.0);
        self.cam_lead.0 += (lead_target.0 - self.cam_lead.0) * ka;
        self.cam_lead.1 += (lead_target.1 - self.cam_lead.1) * ka;
        let focus = render::focus_target(&self.player, (self.viewport[0], self.viewport[1]));
        let focus = (focus.0 + self.cam_lead.0, focus.1 + self.cam_lead.1);
        player::follow_camera(&mut self.camera, focus, dt);
        self.ensure_visible();
        let sprites = self.sprites();
        let tiles = &self.visible_cache.4;
        self.quad_count = render::build_tile_mesh(
            &self.world,
            &mut self.chunks,
            self.camera,
            (self.viewport[0], self.viewport[1]),
            tiles,
            &sprites,
            Some(&self.player),
            &mut self.vertices,
            self.anim_clock,
        );
        self.player_in_mesh = self
            .vertices
            .chunks_exact(render::VERTEX_FLOATS)
            .any(|v| v[2] == render::PLAYER_COLOR[0] && v[3] == render::PLAYER_COLOR[1] && v[4] == render::PLAYER_COLOR[2]);
    }

    /// Recompute the visible-tile list only when the camera or viewport
    /// changes; otherwise reuse the cached set (avoids sorting ~2400 tiles
    /// multiple times per frame).
    fn ensure_visible(&mut self) {
        let key = (
            self.camera.x as i32,
            self.camera.y as i32,
            self.viewport[0] as i32,
            self.viewport[1] as i32,
        );
        if self.visible_cache.0 != key.0
            || self.visible_cache.1 != key.1
            || self.visible_cache.2 != key.2
            || self.visible_cache.3 != key.3
        {
            self.visible_cache = (
                key.0,
                key.1,
                key.2,
                key.3,
                render::visible_tiles(self.camera, (self.viewport[0], self.viewport[1])),
            );
        }
    }

    /// Resource nodes + structures + enemies visible in the current view.
    /// Resource/decor sprites come from the per-chunk cache (resolved once at
    /// generation), so we iterate the few visible chunks instead of re-hashing
    /// ~2400 tiles every frame.
    fn sprites(&mut self) -> Vec<Sprite> {
        let mut sprites = Vec::new();
        // Visible chunk range (matches the tile range used by `visible_tiles`).
        let r = ((self.viewport[0] / HALF_W + self.viewport[1] / HALF_H) / 2.0).ceil() as i32 + 2;
        let min_cx = ((self.camera.x as i32) - r).div_euclid(CHUNK_SIZE);
        let max_cx = ((self.camera.x as i32) + r).div_euclid(CHUNK_SIZE);
        let min_cy = ((self.camera.y as i32) - r).div_euclid(CHUNK_SIZE);
        let max_cy = ((self.camera.y as i32) + r).div_euclid(CHUNK_SIZE);
        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                let chunk = self.chunks.get(&self.world, cx * CHUNK_SIZE, cy * CHUNK_SIZE);
                for &(tx, ty, kind) in &chunk.resources {
                    if !self.nodes.is_depleted(tx, ty) {
                        sprites.push(kind.sprite(tx, ty));
                    }
                }
                for &(tx, ty, kind) in &chunk.decor {
                    sprites.push(kind.sprite(tx, ty));
                }
            }
        }
        for s in &self.structures {
            if s.kind == StructureKind::Chest && self.opened_chests.contains(&(s.tx, s.ty)) {
                sprites.push(Sprite::new(s.tx, s.ty, [0.40, 0.26, 0.10], 16.0, 12.0, 6.0));
            } else {
                sprites.push(s.kind.sprite(s.tx, s.ty));
            }
        }
        for e in self.enemies.enemies() {
            let hp_frac = (e.hp / e.kind.max_hp()).clamp(0.0, 1.0);
            let mut sp = e.kind.sprite(e.x, e.y, hp_frac, e.facing);
            let f = e.flash.min(1.0);
            if f > 0.0 {
                sp.color = [
                    sp.color[0] + (1.0 - sp.color[0]) * f,
                    sp.color[1] + (1.0 - sp.color[1]) * f,
                    sp.color[2] + (1.0 - sp.color[2]) * f,
                ];
            }
            sprites.push(sp);
            // hp bar: dark framed plate + colored fill, floating above the figure
            let bar_lift = match e.kind {
                EnemyKind::Boss => 54.0,
                EnemyKind::Slime => 18.0,
                EnemyKind::Skeleton => 22.0,
                EnemyKind::Goblin => 22.0,
                EnemyKind::Bat => 12.0,
                EnemyKind::Spider => 14.0,
                EnemyKind::Imp => 14.0,
                EnemyKind::Ogre => 28.0,
                EnemyKind::Wraith => 24.0,
                EnemyKind::Stoneslinger => 24.0,
                EnemyKind::Colossus => 64.0,
                EnemyKind::Brute => 30.0,
                EnemyKind::Stormcaller => 26.0,
            };
            sprites.push(
                Sprite::new_center(e.x, e.y, [0.0, 0.0, 0.0], 11.0, 2.5, bar_lift)
                    .with_style(SpriteStyle::HpBack),
            );
            sprites.push(
                Sprite::new_center(
                    e.x,
                    e.y,
                    [1.0 - hp_frac, hp_frac, 0.1],
                    11.0 * hp_frac.max(0.05),
                    2.5,
                    bar_lift,
                )
                .with_style(SpriteStyle::HpFill),
            );
        }
        for a in &self.arrows {
            sprites.push(
                Sprite::new_center(a.x, a.y, [0.95, 0.90, 0.85], 5.0, 2.0, 0.0)
                    .with_facing((a.dx, a.dy))
                    .with_style(SpriteStyle::Arrow),
            );
        }
        // particles (death puffs, hit sparks) — fading Generic quads
        for p in &self.particles {
            let a = (p.life / p.max_life).clamp(0.0, 1.0);
            let mut ps = Sprite::new_center(p.x, p.y, p.color, p.size, p.size, 4.0)
                .with_style(SpriteStyle::Generic);
            ps.alpha = a;
            sprites.push(ps);
        }
        // build-mode ghost preview: a translucent tinted copy of the selected
        // structure on the tile under the cursor — green if it can be placed,
        // red if blocked/occupied/too expensive.
        self.build_ghost = None;
        if let Some(kind) = self.build_mode {
            if let Some((mx, my)) = self.mouse_screen {
                let (wx, wy) = iso_to_world(mx, my);
                let tx = (wx + self.camera.x).floor() as i32;
                let ty = (wy + self.camera.y).floor() as i32;
                let valid = self.can_place(kind, tx, ty);
                self.build_ghost = Some((kind, tx, ty, valid));
                let mut g = kind.sprite(tx, ty);
                g.color = if valid { [0.35, 1.0, 0.45] } else { [1.0, 0.35, 0.35] };
                g.alpha = 0.55;
                sprites.push(g);
            }
        }
        sprites
    }

    /// Campfire point lights in screen pixels: [x, y, intensity, radius, r, g, b, 0] per slot.
    fn light_data(&self) -> [f32; LIGHT_FLOATS] {
        let mut data = [0.0f32; LIGHT_FLOATS];
        let mut n = 0;
        // The player carries a soft, cool lantern-like aura so they stay
        // readable at night and exploring feels atmospheric. Slot 0 is reserved
        // for it; world lights fill the remaining slots.
        {
            let (sx, sy) = game::iso::world_to_iso(
                self.player.x - self.camera.x,
                self.player.y - self.camera.y,
            );
            data[0..4].copy_from_slice(&[sx, sy, 0.32, 66.0]);
            data[4..8].copy_from_slice(&[0.85, 0.78, 0.55, 0.0]);
            n = 1;
        }
        for s in self
            .structures
            .iter()
            .filter(|s| s.kind.emits_light())
        {
            if n >= MAX_LIGHTS {
                break;
            }
            let (sx, sy) = game::iso::world_to_iso(
                s.tx as f32 + 0.5 - self.camera.x,
                s.ty as f32 + 0.5 - self.camera.y,
            );
            let (intensity, radius, rgb) = match s.kind {
                StructureKind::Lantern => (0.40, 70.0, [1.0, 0.80, 0.40]),
                StructureKind::Brazier => (0.70, 120.0, [1.0, 0.50, 0.20]),
                _ => (0.55, 90.0, [1.0, 0.62, 0.28]),
            };
            let slot = n * 8;
            data[slot..slot + 4].copy_from_slice(&[sx, sy, intensity, radius]);
            data[slot + 4..slot + 8].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 0.0]);
            n += 1;
        }
        data
    }

    fn record_pass(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tile_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.07,
                        b: 0.11,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if self.quad_count > 0 {
            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.bind_group, &[]);
            rp.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
            rp.draw(0..self.quad_count * 6, 0..1);
        }
    }

    /// Force a readback on the next frame if one isn't already in flight.
    pub fn trigger_readback(&mut self) {
        READBACK_INFLIGHT.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Last completed readback result (or "pending"/"no app").
    pub fn readback_stats(&self) -> String {
        READBACK.lock().unwrap().clone()
    }

    pub fn render(&mut self) {
        self.write_uniforms();
        if self.quad_count > 0 {
            self.vertex_buffer.upload(&self.queue, &self.vertices);
        }
        if self.frames <= 2 {
            glog(&format!(
                "[gfx] first render: quads={} viewport=({:.0},{:.0})",
                self.quad_count, self.viewport[0], self.viewport[1]
            ));
        }

        // Prefer presenting directly to the WebGPU surface (#game): a single GPU
        // present per frame that never stalls. The read-back-to-#blit path is
        // only a fallback for environments where the surface can't reach the
        // screen (e.g. SwiftShader where present() is a no-op). Forcing it
        // unconditionally made the whole game freeze on normal GPUs, because the
        // readback buffer stayed busy and frames were skipped.
        let frame = self.surface.get_current_texture();
        let surface_tex: Option<wgpu::SurfaceTexture> = match frame {
            wgpu::CurrentSurfaceTexture::Success(f) => Some(f),
            wgpu::CurrentSurfaceTexture::Suboptimal(f) => Some(f),
            // Momentarily unavailable (tab hidden / vsync timeout): keep the last
            // presented frame on screen.
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => None,
            // Genuinely broken surface: drop to the blit readback fallback.
            _ => {
                if self.backend_mode != 2 {
                    self.backend_mode = 2;
                    self.using_blit = true;
                    set_backend("blit");
                }
                None
            }
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        if let Some(f) = surface_tex {
            if self.backend_mode != 1 {
                self.backend_mode = 1;
                self.using_blit = false;
                set_backend("gpu");
            }
            let view = f.texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.record_pass(&view, &mut encoder);
            let cmd = encoder.finish();
            self.queue.submit([cmd]);
            self.queue.present(f);
        } else if self.backend_mode == 2 {
            // Blit fallback: render to the offscreen target and read it back to
            // the 2D #blit canvas (only used when the surface can't present).
            let off_view = self
                .offscreen
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.record_pass(&off_view, &mut encoder);

            // We reuse ONE readback buffer instead of allocating a fresh one every
            // frame. If the buffer is still mapped from the previous frame we skip
            // this frame's readback (cheaply presenting the last picture) rather
            // than double-mapping.
            let width = self.config.width;
            let height = self.config.height;
            let bytes_per_row = ((width * 4 + 255) / 256) * 256;

            let need_new = match &self.readback_buffer {
                Some(b) => b.size() != bytes_per_row as u64 * height as u64,
                None => true,
            };
            if need_new {
                self.readback_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("readback"),
                    size: bytes_per_row as u64 * height as u64,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }));
                self.readback_busy.store(false, std::sync::atomic::Ordering::Relaxed);
            }

            let cmd = encoder.finish();
            if !self.readback_busy.load(std::sync::atomic::Ordering::Relaxed) {
                self.readback_busy.store(true, std::sync::atomic::Ordering::Relaxed);
                let buf = self.readback_buffer.as_ref().unwrap().clone();
                let busy = self.readback_busy.clone();
                let tod = self.time_of_day;
                let aclock = self.anim_clock;
                let weather = self.weather;
                let mut enc = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("readback_encoder"),
                    });
                enc.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.offscreen,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyBufferInfo {
                        buffer: &buf,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(bytes_per_row),
                            rows_per_image: Some(height),
                        },
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
                let rc = enc.finish();
                rc.map_buffer_on_submit(
                    &buf.clone(),
                    wgpu::MapMode::Read,
                    ..,
                    move |res| {
                        match res {
                            Ok(()) => {
                                if let Ok(data) = buf.slice(..).get_mapped_range() {
                                    blit_to_2d_canvas(
                                        &data,
                                        width,
                                        height,
                                        bytes_per_row,
                                        tod,
                                        aclock,
                                        weather,
                                    );
                                    drop(data);
                                    *READBACK.lock().unwrap() = String::from("blitted");
                                } else {
                                    *READBACK.lock().unwrap() = String::from("mapped but no range");
                                }
                                buf.unmap();
                            }
                            Err(e) => {
                                *READBACK.lock().unwrap() = format!("map error: {e}");
                                buf.unmap();
                            }
                        }
                        busy.store(false, std::sync::atomic::Ordering::Relaxed);
                    },
                );
                self.queue.submit([cmd, rc]);
            } else {
                // Previous readback still in flight: just present this render.
                self.queue.submit([cmd]);
            }
        } else {
            // Surface transiently unavailable (Timeout/Occluded): nothing to
            // present this frame; drop the encoder so the last frame persists.
            drop(encoder);
        }

        if self.frames % 600 == 0 {
            glog(&format!(
                "[gfx] heartbeat #{} quads={} backend={}",
                self.frames,
                self.quad_count,
                if self.using_blit { "blit" } else { "gpu" }
            ));
        }
    }

    /// Request a one-off readback + #blit (used by the screenshot key).
    pub fn request_capture(&mut self) {
        self.capture_requested = true;
    }
}

fn create_offscreen(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn resize_canvas(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let window = web_sys::window().unwrap();
    let dpr = window.device_pixel_ratio();
    let width = (window.inner_width().unwrap().as_f64().unwrap() * dpr).round() as u32;
    let height = (window.inner_height().unwrap().as_f64().unwrap() * dpr).round() as u32;
    canvas.set_width(width.max(1));
    canvas.set_height(height.max(1));
    let _ = canvas.set_attribute(
        "style",
        &format!(
            "width:{}px;height:{}px",
            width as f64 / dpr,
            height as f64 / dpr
        ),
    );
    (width, height)
}

fn create_uniforms(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::BindGroup) {
    const UNIFORM_BYTES: u64 = (8 + LIGHT_FLOATS) as u64 * 4;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniforms"),
        size: UNIFORM_BYTES,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = create_bind_group_layout(device);
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniform_group"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (buffer, group)
}

fn create_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("tile_shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = create_pipeline_layout(device, &create_bind_group_layout(device));
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("tile_pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: VERTEX_STRIDE,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
            })],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("tile_pipeline_layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    })
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("uniform_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new((8 + LIGHT_FLOATS) as u64 * 4),
            },
            count: None,
        }],
    })
}