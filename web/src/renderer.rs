use game::building::{BUILDABLE, CHEST_RANGE, Structure, StructureKind, try_build};
use game::iso::iso_to_world;
use game::combat::{
    ARROW_DAMAGE, Arrow,
    arrow_hits, swing_hits,
};
use game::daynight::{DAY_LENGTH, START_TIME, clock, daylight as daylight_at, temperature};
use game::enemy::{AGGRO_RANGE, AiState, Enemy, EnemyRegistry, EnemyKind, WINDUP, spawner_on};
use game::items::{Inventory, ItemKind};
use game::npc::{Npc, NpcKind};
use game::player::{self, Player};
use game::poi::{ruins_at, ruins_walls, town_name, town_site, village_sites, village_name};
use game::weapons::WeaponKind;
use game::quest::QuestLog;
use game::elements::humanoid;
use game::render::{self, Camera, Sprite, SpriteStyle, VERTEX_STRIDE_BYTES};
use game::iso::{HALF_H, HALF_W};
use game::resources::{NodeRegistry, ResourceKind, resource_on, HARVEST_RANGE};
use game::sim::PlayerInput;
use game::world::{ChunkCache, TileKind, WorldGen, tile_at, CHUNK_SIZE};
use crate::network::NetClient;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;

/// Convert an HSV color (h in degrees, s/v in 0..1) to an RGB triplet.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    [r + m, g + m, b + m]
}

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

/// Fire a named SFX in the page's WebAudio engine (playSfx is a global defined
/// in index.html). No-op when audio is unavailable. Used to surface gameplay
/// outcomes (chops, pickups, footsteps, enemy deaths) that happen in Rust.
fn play_sfx(name: &str) {
    if let Some(win) = web_sys::window() {
        if let Ok(f) = js_sys::Reflect::get(&win, &wasm_bindgen::JsValue::from_str("playSfx"))
            .and_then(|v| v.dyn_into::<js_sys::Function>())
        {
            let _ = f.call1(
                &wasm_bindgen::JsValue::NULL,
                &wasm_bindgen::JsValue::from_str(name),
            );
        }
    }
}

/// Show a transient on-screen toast (the `toast` global in index.html). Used for
/// pickup / equip feedback. No-op if the page didn't define one.
fn toast(msg: &str) {
    if let Some(win) = web_sys::window() {
        if let Ok(f) = js_sys::Reflect::get(&win, &wasm_bindgen::JsValue::from_str("toast"))
            .and_then(|v| v.dyn_into::<js_sys::Function>())
        {
            let _ = f.call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str(msg));
        }
    }
}

/// Campfire point light slots (each = position/intensity vec4 + color vec4).
const MAX_LIGHTS: usize = 8;
const LIGHT_FLOATS: usize = MAX_LIGHTS * 8;
/// Seconds the "the city is being built" loading overlay shows after using the
/// village portal, before the player arrives in town.
const TOWN_LOAD_TIME: f32 = 2.6;
/// Seconds the in-world town build-in animation takes on first arrival.
const TOWN_BUILD_TIME: f32 = 2.8;

/// Deterministic 0..1 stagger for the town build-in: each tile reveals at a
/// different moment so the city appears to rise into place.
fn portal_reveal_at(tx: i32, ty: i32) -> f32 {
    let h = ((tx as u32).wrapping_mul(73856093) ^ (ty as u32).wrapping_mul(19349663)) % 100;
    (h as f32) / 100.0
}

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
        ResourceKind::Treasure => "Treasure",
    }
}

fn struct_name(kind: StructureKind) -> &'static str {
    match kind {
        StructureKind::Campfire => "F",
        StructureKind::Dungeon => "D",
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
        StructureKind::Turret => "Y",
        StructureKind::HealingTotem => "H",
        StructureKind::Trap => "Tr",
        StructureKind::House => "Hh",
        StructureKind::Cabin => "Cb",
        StructureKind::Hut => "Hu",
        StructureKind::Inn => "Inn",
        StructureKind::Barn => "Barn",
        StructureKind::Watchtower => "Twr",
        StructureKind::Car => "Car",
        StructureKind::Train => "Train",
        StructureKind::Rail => "Rail",
        StructureKind::Portal => "Portal",
        StructureKind::Banner => "Bn",
        StructureKind::EnchantingTable => "Eq",
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
        EnemyKind::ScorpionQueen => "Scorpion Queen",
        EnemyKind::FrostGolem => "Frost Golem",
        EnemyKind::ToadKing => "Toad King",
        EnemyKind::OceanLeviathan => "Ocean Leviathan",
        EnemyKind::Brute => "Brute",
        EnemyKind::Stormcaller => "Stormcaller",
        EnemyKind::Wolf => "Wolf",
        EnemyKind::Archer => "Archer",
        EnemyKind::Raider => "Raider",
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
    // Screen-space vignette + horizon fog, done on the GPU so the readback
    // fallback path (which previously ran a 500k-pixel CPU loop per frame)
    // stays cheap. u.tint already holds the sky tint for the time of day.
    let p = in.pos.xy;
    let cx = u.viewport.x * 0.5;
    let cy = u.viewport.y * 0.5;
    let nx = (p.x - cx) / cx;
    let ny = (p.y - cy) / cy;
    let dd = sqrt(nx * nx + ny * ny);
    var v = 1.0;
    if (dd > 0.6) {
        let t = clamp((dd - 0.6) / 0.55, 0.0, 1.0);
        let s = t * t * (3.0 - 2.0 * t);
        v = 1.0 - s * 0.45;
    }
    let y_norm = p.y / u.viewport.y;
    let fog_t = clamp((0.55 - y_norm) / 0.55, 0.0, 1.0) * 0.26;
    let fog = clamp(u.tint * 0.8 + 0.2, vec3<f32>(0.0), vec3<f32>(1.0));
    col = vec4<f32>(col.rgb * v, col.a);
    col = vec4<f32>(mix(col.rgb, fog, fog_t), col.a);
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

/// When true (default) the renderer auto-steps the internal resolution down on
/// slow backends (fps-driven). When false, the resolution stays pinned to the
/// user's chosen cap. Exposed so the settings menu can toggle it.
static ADAPTIVE_RES: std::sync::Mutex<bool> = std::sync::Mutex::new(true);

pub fn set_adaptive_res(v: bool) {
    *ADAPTIVE_RES.lock().unwrap() = v;
}

pub fn get_render_cap() -> (u32, u32) {
    *RENDER_CAP.lock().unwrap()
}

/// Internal render/readback resolution ladder (descending). Index 0 is the
/// highest quality; higher indices trade sharpness for fps. The adaptive
/// controller (see `update`) steps down when the measured fps is low so the
/// game stays smooth even on backends with a slow GPU->CPU readback path.
const RES_LEVELS: [(u32, u32); 6] = [
    (960, 540),
    (800, 450),
    (640, 400),
    (560, 315),
    (480, 270),
    (384, 216),
];

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
    hp01: f32,
    hurt01: f32,
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
        // (Vignette + horizon fog now run in the fragment shader on the GPU; the
        // readback path used to recompute them here in a 500k-pixel CPU loop
        // every frame, which was the fps bottleneck on slower machines.)
        for a in cache.buf.iter_mut().skip(3).step_by(4) {
            *a = 255;
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
        draw_atmosphere(&cache.ctx, width, height, tod, aclock, weather, hp01, hurt01);
    });
}

/// Atmosphere / weather / night-vignette overlay, drawn in 2D over the base
/// frame. Shared by both the readback path and the fast GPU->GPU blit path.
fn draw_atmosphere(
    ctx: &CanvasRenderingContext2d,
    width: u32,
    height: u32,
    tod: f32,
    aclock: f32,
    weather: u8,
    hp01: f32,
    hurt01: f32,
) {
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
    // weather: snow (2), rain (1), storm (3), or heat wave (4)
    if weather != 0 {
        let snow = weather == 2;
        let storm = weather == 3;
        let heat = weather == 4;
        if storm {
            // Storm: heavy, near-vertical rain driven on the wind + a dark veil.
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(150,175,205,0.5)"));
            ctx.set_line_width(1.5);
            let cols = 150u32;
            let fall = (aclock as f64 * 520.0) % h;
            for i in 0..cols {
                let fi = i as f64;
                let x = (((fi * 53.7 + aclock as f64 * 200.0) % w) + w) % w;
                let y = (((fi * 29.3 + fall) % h) + h) % h;
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x - 7.0, y + 22.0);
                ctx.stroke();
            }
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(70,85,110,0.22)"));
            ctx.fill_rect(0.0, 0.0, w, h);
        } else if heat {
            // Heat wave: a warm, shimmering veil that tints the world amber.
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(255,170,70,0.12)"));
            ctx.fill_rect(0.0, 0.0, w, h);
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(255,210,120,0.06)"));
            for i in 0..40u32 {
                let fi = i as f64;
                let x = (((fi * 91.3 + aclock as f64 * 18.0) % w) + w) % w;
                let y = (((fi * 47.1 - aclock as f64 * 12.0) % h) + h) % h;
                ctx.begin_path();
                ctx.arc(x, y, 2.0 + (fi * 3.0).fract() * 2.0, 0.0, std::f64::consts::TAU);
                ctx.fill();
            }
        } else if snow {
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(255,255,255,0.85)"));
            let cols = 110u32;
            let fall = (aclock as f64 * 90.0) % h;
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
            let fall = (aclock as f64 * 380.0) % h;
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
    // Low-HP warning: a pulsing red vignette when health is critical (<40%).
    let low = ((0.4 - hp01) / 0.4).clamp(0.0, 1.0);
    if low > 0.001 {
        let pulse = 0.55 + 0.45 * (aclock * 4.0).sin();
        let a = 0.5 * low * pulse;
        if let Ok(grad) = ctx.create_radial_gradient(
            w / 2.0, h / 2.0,
            (w.min(h)) * 0.18,
            w / 2.0, h / 2.0,
            (w.max(h)) * 0.78,
        ) {
            let _ = grad.add_color_stop(0.0, "rgba(150,0,0,0)");
            let _ = grad.add_color_stop(1.0, &format!("rgba(150,0,0,{})", a));
            ctx.set_fill_style(grad.as_ref());
            ctx.fill_rect(0.0, 0.0, w, h);
        }
    }
    // Instant red flash on taking a hit (decays quickly via hurt01).
    if hurt01 > 0.001 {
        ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(&format!(
            "rgba(190,10,10,{})",
            0.32 * hurt01
        )));
        ctx.fill_rect(0.0, 0.0, w, h);
    }
}

/// Fast display path: copy the WebGPU surface (#game) onto the 2D #blit canvas
/// with a GPU->GPU `drawImage` instead of a GPU->CPU readback + `putImageData`.
/// The surface's backing is still drawable even when the browser won't composite
/// the WebGPU canvas itself, so this avoids the slow CPU stall entirely.
fn blit_via_draw(width: u32, height: u32, tod: f32, aclock: f32, weather: u8, hp01: f32, hurt01: f32) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let doc = match window.document() {
        Some(d) => d,
        None => return,
    };
    let game = match doc
        .get_element_by_id("game")
        .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
    {
        Some(c) => c,
        None => return,
    };
    let blit = match doc
        .get_element_by_id("blit")
        .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
    {
        Some(c) => c,
        None => return,
    };
    if blit.width() != width || blit.height() != height {
        blit.set_width(width);
        blit.set_height(height);
    }
    let ctx = match blit
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
    {
        Some(c) => c,
        None => return,
    };
    let _ = ctx.draw_image_with_html_canvas_element(&game, 0.0, 0.0);
    draw_atmosphere(&ctx, width, height, tod, aclock, weather, hp01, hurt01);
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

/// EXPERIMENT switch: when true, on a successful surface present we display the
/// WebGPU surface directly (gpu mode) and SKIP the per-frame `drawImage` readback
/// to #blit that is the measured ~50-70ms fps wall. Set via the `?gpu` URL param.
static FORCE_GPU: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_force_gpu(v: bool) {
    FORCE_GPU.store(v, std::sync::atomic::Ordering::Relaxed);
}

/// A ground loot drop the player walks over to collect. Spawned when enemies
/// or resource nodes are destroyed; auto-collected on proximity.
#[derive(Clone, Copy)]
pub struct LootDrop {
    pub kind: game::items::ItemKind,
    pub x: f32,
    pub y: f32,
    /// Stack size carried by this drop.
    pub count: u32,
    /// Remaining lifetime in seconds; despawned at 0 so the world stays clean.
    pub ttl: f32,
    /// Animation phase for the bob/sparkle.
    pub phase: f32,
}

/// A weapon lying on the ground, collected by walking over it. Found in chests or
/// dropped by enemies.
#[derive(Clone, Copy)]
pub struct WeaponDrop {
    pub kind: game::weapons::WeaponKind,
    pub x: f32,
    pub y: f32,
    pub ttl: f32,
    pub phase: f32,
}

/// A building interior the player has stepped into. The room is drawn centered on
/// the player's world position (`bx,by`), so leaving restores the exact spot.
/// `px,py` are the player's local position within the room (tiles from center).
struct Interior {
    kind: StructureKind,
    floor: u8,
    max_floors: u8,
    rw: f32,
    rh: f32,
    px: f32,
    py: f32,
    bx: f32,
    by: f32,
    /// Spike hazard tiles (room-relative, integer) that damage the player.
    hazards: Vec<(i32, i32)>,
    /// True once the dungeon's vault loot has been claimed (one-time reward).
    loot_taken: bool,
}

/// A line of lore tied to each recovered Crown Fragment (Chapter 3 beats).
fn fragment_lore(bit: u8) -> &'static str {
    match bit {
        0 => "The Forest Warden's fragment hums with the grove's last breath.",
        1 => "The Scorpion Queen's fragment burns with desert sun.",
        2 => "The Frost Golem's fragment aches with tundra cold.",
        3 => "The Toad King's fragment drips with swamp venom.",
        4 => "The Ocean Leviathan's fragment tastes of distant tides.",
        _ => "A shard of the shattered Star Crown.",
    }
}

/// Which Crown Fragment guardian rules a given biome tile (None for neutral
/// ground). Used to spawn the right boss when the player explores a biome.
fn boss_for_biome(t: TileKind) -> Option<EnemyKind> {
    Some(match t {
        TileKind::Forest => EnemyKind::Boss,
        TileKind::Desert => EnemyKind::ScorpionQueen,
        TileKind::Snow | TileKind::Tundra => EnemyKind::FrostGolem,
        TileKind::Swamp => EnemyKind::ToadKing,
        TileKind::Water | TileKind::ShallowWater | TileKind::DeepWater => EnemyKind::OceanLeviathan,
        _ => return None,
    })
}

/// The next Crown Fragment still unrecovered, so the elite spawner can drive the
/// player toward collecting all five (cycling if they avoid a specific biome).
fn next_fragment_boss(fragments: u8) -> Option<EnemyKind> {
    for (bit, k) in [
        (0u8, EnemyKind::Boss),
        (1, EnemyKind::ScorpionQueen),
        (2, EnemyKind::FrostGolem),
        (3, EnemyKind::ToadKing),
        (4, EnemyKind::OceanLeviathan),
    ] {
        if fragments & (1 << bit) == 0 {
            return Some(k);
        }
    }
    None
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
    /// Ground loot dropped by enemies (and harvest), collected on proximity.
    loot: Vec<LootDrop>,
    /// Weapons lying on the ground (from enemy drops or chests).
    weapon_loot: Vec<WeaponDrop>,
    quest: QuestLog,
    ruins: (i32, i32),
    /// Village hamlets: (center_x, center_y, name). Generated at world init so the
    /// same seed always yields the same settlements. Houses there act as shelters.
    villages: Vec<(i32, i32, String)>,
    /// Village centers the player has already entered (so the welcome toast fires once).
    visited_villages: std::collections::HashSet<(i32, i32)>,
    /// The single city per world (walled, with a railway + old vehicles).
    towns: Vec<(i32, i32, String)>,
    visited_towns: std::collections::HashSet<(i32, i32)>,
    /// Village portal position (world coords). Stepping through it travels to the
    /// town. None until `reset_world` places it in the first village.
    portal: Option<(f32, f32)>,
    /// Loading-overlay timer (seconds) while the town is "being built" in the
    /// background after using the portal. > 0 means the transition is playing.
    town_transition: f32,
    /// In-world reveal progress (0..1) for the town's buildings the first time the
    /// player arrives. Ramps 0→1 so structures pop into place.
    town_build_t: f32,
    /// Town layout (tx, ty, kind) captured when first generated and persisted so
    /// re-visiting never re-rolls a different city.
    town_structures: Vec<(i32, i32, StructureKind)>,
    /// True once the player has visited the town (so its creation animation only
    /// plays on the very first arrival).
    town_visited: bool,
    /// Villagers, guards and merchants wandering the hamlets. Cosmetic/local.
    npcs: Vec<Npc>,
    /// Active building interior (None = outside). Entering a house paints a
    /// room centered on the player's world position; multi-floor buildings add a
    /// stair tile that climbs to the next floor.
    interior: Option<Interior>,
    opened_chests: std::collections::HashSet<(i32, i32)>,
    slimes_killed: u32,
    boss_killed: u32,
    colossus_killed: u32,
    /// Bitmask (bits 0..5) of the five Crown Fragments recovered from the biome
    /// bosses. 0b11111 = all five collected; gates the reforge finale.
    fragments: u8,
    boss_spawned: bool,
    altar_placed: bool,
    altar_tile: Option<(i32, i32)>,
    near_altar: bool,
    /// Debounce so the "altar is cold" hint only toasts once per altar visit.
    altar_hinted: bool,
    ending_pending: bool,
    /// 0 = Reign, 1 = Shatter, None = campaign not finished.
    ending: Option<u8>,
    ng_plus: u32,
    spawn_point: (f32, f32),
    time_of_day: f32,
    anim_clock: f32,
    /// Screen-shake amplitude (world units). Minimal: only nudged when a boss
    /// lands a hit, and decays quickly each frame.
    shake: f32,
    /// Previous frame's player.hurt_timer, for detecting the rising edge of a hit.
    prev_hurt: f32,
    respawn_timer: f32,
    debug_swing_hits: u32,
    debug_attacks: u32,
    debug_shots: u32,
    /// Remaining cooldown on the attack action (driven by the equipped weapon's
    /// cadence), so heavier weapons swing slower.
    swing_cd: f32,
    /// Brief hit-stop timer (seconds): while > 0 the simulation dt is scaled down
    /// to a near-freeze so hits read with impact. Cosmetic only.
    hitstop: f32,
    vertices: Vec<f32>,
    quad_count: u32,
    frames: u64,
    player_in_mesh: bool,
    readback_buffer: Option<wgpu::Buffer>,
    capture_requested: bool,
    using_blit: bool,
    /// Index into RES_LEVELS (higher index = smaller internal resolution).
    /// Driven by the fps-based adaptive controller so slow backends (e.g.
    /// default Linux Chrome without Vulkan, where WebGPU can't present a
    /// canvas and falls back to a slow GPU->CPU readback) still run smoothly.
    res_level: usize,
    /// Highest RES_LEVELS index allowed by the user's render-cap setting.
    max_res_level: usize,
    fps_est: f32,
    res_timer: f32,
    /// Throttle for footstep SFX (seconds until next step is allowed).
    step_timer: f32,
    backend_mode: u8,
    /// Authoritative frames-per-second, measured from actual sim steps.
    fps: f32,
    fps_acc: u32,
    fps_time: f32,
    /// Player's current movement speed in tiles/second (0 while idle).
    speed: f32,
    prev_px: f32,
    prev_py: f32,
    /// World position the HUD compass should point at (nearest unrecovered
    /// fragment's guardian, or the altar once all five are in hand). None hides
    /// the compass. Recomputed periodically, not every frame.
    objective: Option<(f32, f32)>,
    obj_timer: f32,
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
    /// Seconds until the next roaming elite (mini-boss) spawns into the world.
    elite_timer: f32,
    /// Seconds until the next night-raider band sweeps the player's base.
    raider_timer: f32,
    /// Farm plots: seconds remaining until each planted plot is ready to
    /// harvest again (keyed by tile). Plots not present are grown (0).
    farm_cd: std::collections::HashMap<(i32, i32), f32>,
    /// Turret emplacements: seconds until each may fire again (keyed by tile).
    turret_cd: std::collections::HashMap<(i32, i32), f32>,
    /// True once the player has crafted Iron Plate (used by the quest log).
    crafted_iron: bool,
    /// Red damage-flash intensity (0..1), decays each frame; set to 1 on hit.
    hurt_flash: f32,
    /// Virtual-joystick vector for touch/mobile movement (None = no analog
    /// input; Some((x,y)) is an un-normalized drag offset that yields a
    /// direction). Takes priority over the WASD keys while active.
    analog: Option<(f32, f32)>,
    /// Multiplayer client (None in single-player).
    net: Option<NetClient>,
    /// Our server-assigned player id (set once the server Welcomes us).
    net_id: Option<u32>,
    /// Co-op room code we joined (shown in the HUD so it can be shared).
    room_code: Option<String>,
    /// Remote co-op players, rebuilt each frame from the server snapshot.
    /// Stored as (server_id, Player) so we can tint each ally a distinct color.
    remote_players: Vec<(u32, Player)>,
    /// One-shot action intents latched by input handlers, consumed each
    /// frame when building the network PlayerInput.
    net_atk: bool,
    net_dodge: bool,
    net_harvest: bool,
    net_eat: bool,
    net_shoot: bool,
    net_build: Option<(StructureKind, i32, i32)>,
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
    /// Lightweight per-frame accessor for co-op name-tag screen positions
    /// (id + projected x/y). Cheaper than `ui_data` so the HUD can poll it every
    /// frame to float labels over remote players' heads.
    pub fn coop_tags(&self) -> String {
        serde_json::json!(
            self.remote_players
                .iter()
                .filter(|(_, rp)| rp.alive)
                .map(|(id, rp)| {
                    let (sx, sy) =
                        game::iso::world_to_iso(rp.x - self.camera.x, rp.y - self.camera.y);
                    serde_json::json!([*id, sx, sy - 26.0])
                })
                .collect::<Vec<_>>()
        )
        .to_string()
    }

    /// Cheap per-frame portal-transition status for the HUD loading overlay
    /// (driven every frame so the "city is being built" bar is smooth).
    pub fn town_status(&self) -> String {
        serde_json::json!({
            "transition": self.town_transition > 0.0,
            "build": self.town_build_t < 1.0,
            "name": self.towns.first().map(|t| t.2.clone()).unwrap_or_default(),
            "progress": if self.town_transition > 0.0 {
                1.0 - self.town_transition / TOWN_LOAD_TIME
            } else {
                1.0
            },
        })
        .to_string()
    }

    /// Headless "visual engine" frame dump: every sprite in screen space plus the
    /// player, so a non-multimodal agent can render the scene as ASCII and assert
    /// on layout/animation. Coordinates `sx, sy` are iso-screen pixels relative to
    /// the viewport center (same mapping the lights use), in world units of px.
    pub fn frame_dump(&mut self) -> String {
        let cam = self.camera;
        let mut sprites = Vec::new();
        for s in self.sprites() {
            let (sx, sy) = game::iso::world_to_iso(s.x - cam.x, s.y - cam.y);
            sprites.push(serde_json::json!({
                "label": game::render::style_label(s.style),
                "style": format!("{:?}", s.style),
                "x": s.x,
                "y": s.y,
                "sx": sx,
                "sy": sy,
                "hw": s.half_w,
                "hh": s.half_h,
                "r": s.color[0],
                "g": s.color[1],
                "b": s.color[2],
                "walk": s.walk,
                "attack": s.attack,
                "flash": s.flash,
            }));
        }
        let (psx, psy) = game::iso::world_to_iso(self.player.x - cam.x, self.player.y - cam.y);
        // Include the player as a sprite (with its real drawn size) so the
        // headless visualizer can compare the character to the buildings.
        let parts = humanoid::build(
            self.player.x,
            self.player.y,
            [0.82, 0.66, 0.5],
            1.0,
            self.player.facing,
            0.0,
            0.0,
            self.player.swing_t,
        );
        let (mut minx, mut maxx, mut miny, mut maxy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for p in &parts {
            minx = minx.min(p.cx - p.hw);
            maxx = maxx.max(p.cx + p.hw);
            miny = miny.min(p.cy - p.hh + p.lift);
            maxy = maxy.max(p.cy + p.hh + p.lift);
        }
        sprites.insert(
            0,
            serde_json::json!({
                "label": "P",
                "style": "Humanoid",
                "x": self.player.x,
                "y": self.player.y,
                "sx": psx,
                "sy": psy,
                "hw": (maxx - minx) / 2.0,
                "hh": (maxy - miny) / 2.0,
                "r": 0.82, "g": 0.66, "b": 0.5,
                "walk": 0.0,
                "attack": self.player.swing_t,
                "flash": 0.0,
            }),
        );
        let craft_hint = if self.near_anvil() {
            let (label, cost) = CRAFT_RECIPES[0];
            let ready = cost.iter().all(|(it, n)| self.inventory.count(*it) >= *n);
            format!("Anvil ready: craft {label} ({})", if ready { "materials OK" } else { "need materials" })
        } else if self.near_enchanting_table() {
            format!("Enchanting Table ready: spend gems to enchant weapon")
        } else {
            "No anvil nearby — build one (N) to craft".to_string()
        };
        let gold = self.inventory.count(ItemKind::Gold);
        let dump = serde_json::json!({
            "cam": { "x": cam.x, "y": cam.y },
            "interior": self.interior.is_some(),
            "quest_text": self.quest.quest_text(self.fragments),
            "craft_hint": craft_hint,
            "quest_stage": self.quest.stage,
            "gold": gold,
            "player": {
                "x": self.player.x,
                "y": self.player.y,
                "sx": psx,
                "sy": psy,
                "attack": self.player.swing_t,
            },
            "sprites": sprites,
        });
        dump.to_string()
    }

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
            // Portal transition: tells the HUD to show the "city is being built"
            // loading overlay and (once arrived) that the build-in animation is
            // still playing.
            "townTransition": self.town_transition > 0.0,
            "townBuild": self.town_build_t < 1.0,
            "townName": self.towns.first().map(|t| t.2.clone()).unwrap_or_default(),
            "townProgress": if self.town_transition > 0.0 {
                1.0 - self.town_transition / TOWN_LOAD_TIME
            } else {
                1.0
            },
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
        let enemies: Vec<serde_json::Value> = self
            .enemies
            .enemies()
            .map(|e| serde_json::json!([e.x, e.y, e.kind.name(), e.kind.is_boss()]))
            .collect();
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
            TileKind::Jungle,
            TileKind::Volcanic,
        ]
        .iter()
        .map(|k| {
            let c = k.color();
            let rgb = ((c[0] * 255.0) as u32) << 16 | ((c[1] * 255.0) as u32) << 8 | ((c[2] * 255.0) as u32);
            serde_json::json!({ "name": format!("{:?}", k), "color": rgb })
        })
        .collect();
        let village_markers: Vec<(i32, i32, String)> = self
            .villages
            .iter()
            .map(|(x, y, n)| (*x, *y, n.clone()))
            .collect();
        let town_markers: Vec<(i32, i32, String)> = self
            .towns
            .iter()
            .map(|(x, y, n)| (*x, *y, n.clone()))
            .collect();
        // Buried caches are only revealed on the minimap while a treasure map is
        // in the player's pack.
        let has_map = self.inventory.count(ItemKind::Map) > 0;
        let treasure: Vec<(i32, i32)> = if has_map {
            let mut v = Vec::new();
            for dy in -R..=R {
                for dx in -R..=R {
                    let tx = ptx + dx;
                    let ty = pty + dy;
                    let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
                    if let Some(ResourceKind::Treasure) = resource_on(tx, ty, tile) {
                        if !self.nodes.is_depleted(tx, ty) {
                            v.push((tx, ty));
                        }
                    }
                }
            }
            v
        } else {
            Vec::new()
        };
        serde_json::json!({
            "n": N,
            "cells": cells,
            "player": [self.player.x, self.player.y],
            "facing": [self.player.facing.0, self.player.facing.1],
            "enemies": enemies,
            "structs": structs,
            "villages": village_markers,
            "towns": town_markers,
            "treasure": treasure,
            "legend": legend,
        })
        .to_string()
    }

    /// Bestiary / Codex: every enemy kind the player has discovered so far,
    /// with its stats and behaviour. Returns a JSON array of objects.
    /// Human-readable name of the biome under the player (e.g. "Forest").
    pub fn biome_name(&self) -> String {
        format!("{:?}", self.cur_biome)
    }

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
        let px = self.player.x;
        let py = self.player.y;
        let boss_alive = self
            .enemies
            .enemies()
            .any(|e| e.kind.is_boss()) as u8;
        let boss_hp = self
            .enemies
            .enemies()
            .filter(|e| e.kind.is_boss())
            .map(|e| {
                let d = (e.x - px).powi(2) + (e.y - py).powi(2);
                (d, e)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, e)| (e.hp / e.kind.max_hp() * 100.0) as u32)
            .unwrap_or(0);
        let ending_str = match self.ending {
            None => "none",
            Some(0) => "reign",
            Some(1) => "shatter",
            Some(2) => "twin",
            Some(_) => "unknown",
        };
        let near_str = {
            let px = self.player.x;
            let py = self.player.y;
            let mut best: Option<(f32, String)> = None;
            for (vx, vy, name) in self.villages.iter().chain(self.towns.iter()) {
                let d = ((px - (*vx as f32 + 0.5)).powi(2) + (py - (*vy as f32 + 0.5)).powi(2)).sqrt();
                if d < 8.0 && best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, name.clone()));
                }
            }
            best.map(|(_, n)| n).unwrap_or_else(|| "wilderness".to_string())
        };
        let online = self.remote_players.len();
        let coopids = self
            .remote_players
            .iter()
            .map(|(id, _)| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "quads={} frames={} player=({:.1},{:.1}) hp={:.0} hunger={:.0} stamina={:.0} thirst={:.0} alive={} inv=(w{},s{},f{},h{},g{},gold{}) structures={} structs={} mobs={} mob={} pack={} swings={} atk={} shots={} quest=S{} ruins=({},{}) chest={} time={}             near={} boss={} colossus={} frag={} altar={} nearaltar={} nearAnvil={} nearEnch={} ending={} weather={} ng={} seed={} biome={:?} bosshp={} altartile={} fps={:.0} spd={:.2}              kev={} klast={} weapon={} enchant={} level={} maxhp={:.0} xp={} near2={} online={} coopids={} maps={} objdx={:.1} objdy={:.1} win={} endpend={}",

            self.quad_count(),
            self.frames(),
            self.player_x(),
            self.player_y(),
            self.player.hp,
            self.player.hunger,
            self.player.stamina,
            self.player.thirst,
            self.player.alive as u8,
            self.inventory.count(ItemKind::Wood),
            self.inventory.count(ItemKind::Stone),
            self.inventory.count(ItemKind::Food),
            self.inventory.count(ItemKind::Herb),
            self.inventory.count(ItemKind::Gem),
            self.inventory.count(ItemKind::Gold),
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
            boss_alive,
            self.colossus_killed,
            self.inventory.count(ItemKind::Fragment),
            self.altar_placed as u8,
            self.near_altar as u8,
            self.near_anvil() as u8,
            self.near_enchanting_table() as u8,
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
             self.player.weapon.name(),
             self.player.enchant,
             self.player.level,
             self.player.max_hp(),
             self.player.xp,
             near_str,
             online,
             coopids,
              self.inventory.count(ItemKind::Map),
              self.objective.map_or(0.0, |(ox, _)| ox - self.player.x),
              self.objective.map_or(0.0, |(_, oy)| oy - self.player.y),
              self.ending.is_some() as u8,
              self.ending_pending as u8,
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

        let mut app = Self {
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
            loot: Vec::new(),
            weapon_loot: Vec::new(),
            quest: QuestLog::new(),
            ruins,
            villages: Vec::new(),
            visited_villages: std::collections::HashSet::new(),
            towns: Vec::new(),
            visited_towns: std::collections::HashSet::new(),
            portal: None,
            town_transition: 0.0,
            town_build_t: 1.0,
            town_structures: Vec::new(),
            town_visited: false,
            npcs: Vec::new(),
            interior: None,
            opened_chests: std::collections::HashSet::new(),
            slimes_killed: 0,
            boss_killed: 0,
            colossus_killed: 0,
            boss_spawned: false,
            elite_timer: 60.0,
            raider_timer: 75.0,
            altar_placed: false,
            altar_tile: None,
            near_altar: false,
            altar_hinted: false,
            ending_pending: false,
            ending: None,
            ng_plus: 0,
            fragments: 0,
            spawn_point: (px, py),
            time_of_day: START_TIME,
            anim_clock: 0.0,
            shake: 0.0,
            prev_hurt: 0.0,
            respawn_timer: 0.0,
            debug_swing_hits: 0,
            debug_attacks: 0,
            debug_shots: 0,
            swing_cd: 0.0,
            hitstop: 0.0,
            vertices: Vec::with_capacity(64 * 1024 * 6),
            quad_count: 0,
            frames: 0,
            player_in_mesh: false,
            readback_buffer: None,
            capture_requested: false,
            using_blit: false,
            res_level: 0,
            max_res_level: 0,
            fps_est: 60.0,
            res_timer: 0.0,
            step_timer: 0.0,
            backend_mode: 0,
            fps: 0.0,
            fps_acc: 0,
            fps_time: 0.0,
            speed: 0.0,
            prev_px: 0.0,
            prev_py: 0.0,
            objective: None,
            obj_timer: 0.0,
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
            turret_cd: std::collections::HashMap::new(),
            crafted_iron: false,
            hurt_flash: 0.0,
            analog: None,
            net: None,
            net_id: None,
            room_code: None,
            remote_players: Vec::new(),
            net_atk: false,
            net_dodge: false,
            net_harvest: false,
            net_eat: false,
            net_shoot: false,
            net_build: None,
        };

        // Multiplayer: `?mp=ws://host:port[&name=Alias][&token=abc]` joins a
        // co-op server. The server is authoritative; the client overlays the
        // synced world each frame on top of its local (predictive) sim.
        let query: Vec<(String, String)> = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .map(|q| {
                q.trim_start_matches('?')
                    .split('&')
                    .filter_map(|kv| {
                        let mut it = kv.splitn(2, '=');
                        let k = it.next()?;
                        let v = it.next().unwrap_or("");
                        Some((k.to_string(), v.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        if let Some(url) = query
            .iter()
            .find(|(k, _)| k == "mp")
            .map(|(_, v)| v.clone())
        {
            let name = query
                .iter()
                .find(|(k, _)| k == "name")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "Wanderer".to_string());
            let token = query
                .iter()
                .find(|(k, _)| k == "token")
                .map(|(_, v)| v.clone());
            let room = query
                .iter()
                .find(|(k, _)| k == "room")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            if let Ok(client) = NetClient::connect(&url, &name, token, &room) {
                app.net = Some(client);
                app.room_code = if room.is_empty() { None } else { Some(room.clone()) };
                // Surface the room code in the HUD so it can be shared.
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(el) = doc.get_element_by_id("hud-room") {
                        let _ = el.set_text_content(Some(&format!("🛡 Room {}", room)));
                        let _ = el.set_attribute("style", "display:inline-block");
                    }
                }
                glog("[net] joined co-op server");
            }
        }

        Ok(app)
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
        // Shift (hold) raises the player's guard — handled every frame from the
        // key state, so both keydown and keyup must update it.
        if code == "ShiftLeft" || code == "ShiftRight" {
            self.player.blocking = down;
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
                "KeyP" => {
                    self.player.cycle_weapon();
                    toast(&format!("Equipped {}", self.player.weapon.name()));
                }
                "KeyJ" => {
                    self.attack();
                }
                "KeyK" => {
                    self.attack();
                }
                "KeyC" => {
                    self.net_eat = true;
                    if self.player.eat(&mut self.inventory) {
                        play_sfx("eat");
                    }
                }
                "KeyT" => {
                    if self.near_water() {
                        if self.player.drink_water() {
                            play_sfx("drink");
                            toast("Drank water");
                        }
                    } else {
                        toast("No water nearby");
                    }
                }
                "KeyR" => {
                    if self.use_salve() {
                        play_sfx("salve");
                    }
                }
                "KeyM" => self.craft_weapon(),
                "KeyO" => self.talk_nearest_npc(),
                "KeyL" => self.cook(),
                "Enter" => self.toggle_interior(),
                "Space" => {
                    self.dodge();
                    play_sfx("dodge");
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

    /// Virtual-joystick input for touch devices. `(x, y)` is the raw drag offset
    /// in pixels from the stick origin; `(0, 0)` clears analog control so the
    /// WASD keys take over again. The movement code normalizes this to a
    /// direction each tick.
    pub fn set_analog(&mut self, x: f32, y: f32) {
        if x == 0.0 && y == 0.0 {
            self.analog = None;
        } else {
            self.analog = Some((x, y));
        }
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
            play_sfx("build");
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

    /// True when the player is near an Enchanting Table (gem → enchant).
    pub fn near_enchanting_table(&self) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        self.structures.iter().any(|s| {
            s.kind == StructureKind::EnchantingTable
                && (s.tx as f32 + 0.5 - px).abs() <= 1.5
                && (s.ty as f32 + 0.5 - py).abs() <= 1.5
        })
    }

    /// True when the player is standing within ~1.5 tiles of a lit Campfire,
    /// which lets them cook raw food into a hot meal (see `cook`).
    pub fn near_campfire(&self) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        self.structures.iter().any(|s| {
            s.kind == StructureKind::Campfire
                && (s.tx as f32 + 0.5 - px).abs() <= 1.5
                && (s.ty as f32 + 0.5 - py).abs() <= 1.5
        })
    }

    /// Cook at a campfire: two raw Food become a hot meal that restores more
    /// health and a little thirst than eating raw. Bound to KeyL.
    pub fn cook(&mut self) {
        if !self.near_campfire() {
            toast("Stand near a campfire to cook");
            return;
        }
        if self.inventory.count(ItemKind::Food) < 2 {
            toast("Need 2 food to cook a meal");
            return;
        }
        self.inventory.remove(ItemKind::Food, 2);
        self.player.hp = (self.player.hp + 25.0).min(self.player.max_hp());
        self.player.hunger = (self.player.hunger + 20.0).min(100.0);
        self.player.thirst = (self.player.thirst + 10.0).min(100.0);
        play_sfx("eat");
        toast("Cooked a hot meal (+25 HP, +hunger)");
    }

    /// Craft the next uncrafted weapon at an anvil: spends its resource cost,
    /// unlocks it, and equips it immediately. Enchanting happens at an
    /// Enchanting Table (spend Gems, +15% dmg per level, cap 5).
    pub fn craft_weapon(&mut self) {
        // --- Enchanting Table path: spend Gems to enchant the equipped weapon ---
        if self.near_enchanting_table() {
            if self.player.weapon == game::weapons::WeaponKind::Fists {
                toast("Equip a weapon first to enchant it");
                return;
            }
            if self.player.enchant >= 5 {
                toast("Weapon is already maximally enchanted (★★★★★)");
                return;
            }
            let cost = 1 + self.player.enchant as u32;
            if self.inventory.count(ItemKind::Gem) < cost {
                toast(&format!(
                    "Need {} gems to enchant (have {})",
                    cost,
                    self.inventory.count(ItemKind::Gem)
                ));
                return;
            }
            self.inventory.remove(ItemKind::Gem, cost);
            self.player.enchant += 1;
            play_sfx("craft");
            toast(&format!(
                "Enchanted {}! ({}★) +{}% damage",
                self.player.weapon.name(),
                self.player.enchant,
                self.player.enchant * 15
            ));
            return;
        }
        // --- Anvil path: forge weapons + Iron Plate ---
        if !self.near_anvil() {
            toast("Stand near an anvil (forge) or enchanting table (enchant)");
            return;
        }
        let order = [
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Spear,
            WeaponKind::Hammer,
            WeaponKind::Bow,
        ];
        let target = order.iter().find(|&&k| !self.player.has_weapon(k));
        let k = match target {
            Some(&k) => k,
            None => {
                // All weapons forged. Forging Iron Plate next.
                let (w, s, h) = (2u32, 5u32, 3u32);
                if self.inventory.count(ItemKind::Wood) < w
                    || self.inventory.count(ItemKind::Stone) < s
                    || self.inventory.count(ItemKind::Herb) < h
                {
                    toast(&format!(
                        "Need {} wood, {} stone, {} herb for Iron Plate",
                        w, s, h
                    ));
                    return;
                }
                self.inventory.remove(ItemKind::Wood, w);
                self.inventory.remove(ItemKind::Stone, s);
                self.inventory.remove(ItemKind::Herb, h);
                self.craft_armor = 0.25;
                play_sfx("craft");
                toast("Forged Iron Plate! (-25% damage)");
                return;
            }
        };
        let (w, s, h) = k.craft_cost().unwrap();
        if self.inventory.count(ItemKind::Wood) < w
            || self.inventory.count(ItemKind::Stone) < s
            || self.inventory.count(ItemKind::Herb) < h
        {
            toast(&format!(
                "Need {} wood, {} stone, {} herb for {}",
                w, s, h, k.name()
            ));
            return;
        }
        self.inventory.remove(ItemKind::Wood, w);
        self.inventory.remove(ItemKind::Stone, s);
        self.inventory.remove(ItemKind::Herb, h);
        self.player.equip_weapon(k);
        play_sfx("craft");
        toast(&format!("Forged a {}!", k.name()));
    }

    /// Talk to the nearest townsperson within range: shows their name and a short
    /// line of dialogue (flavor + occasional hints).
    pub fn talk_nearest_npc(&mut self) {
        let mut best: Option<(f32, usize)> = None;
        for (i, n) in self.npcs.iter().enumerate() {
            let dx = n.x - self.player.x;
            let dy = n.y - self.player.y;
            let d2 = dx * dx + dy * dy;
            if d2 < 12.0 {
                if best.map_or(true, |(bd, _)| d2 < bd) {
                    best = Some((d2, i));
                }
            }
        }
        let idx = match best {
            Some((_, i)) => i,
            None => {
                toast("No one nearby to talk to");
                return;
            }
        };
        let seed = ((self.npcs[idx].x * 13.0 + self.npcs[idx].y * 7.0) as u32)
            ^ (idx as u32).wrapping_mul(2654435761);
        let npc = &mut self.npcs[idx];

        // Fulfill an active quest if the player carries the goods.
        if let Some((item, need, xp, ritem, rcount)) = npc.quest {
            let have = self.inventory.count(item);
            if have >= need {
                self.inventory.remove(item, need);
                self.inventory.add(ritem, rcount);
                let lvl_before = self.player.level;
                self.player.add_xp(xp);
                let reward_name = ritem.name();
                let mut msg =
                    format!("{}: Quest done! +{}xp, +{} {}", npc.name, xp, rcount, reward_name);
                if self.player.level > lvl_before {
                    play_sfx("levelup");
                    msg.push_str(&format!(" — Level {}!", self.player.level));
                } else {
                    play_sfx("pickup");
                }
                toast(&msg);
                npc.quest = None;
                return;
            }
            toast(&format!(
                "{}: Bring {} {} (you have {}). Reward: {}xp + {} {}",
                npc.name,
                need,
                item.name(),
                have,
                xp,
                rcount,
                ritem.name()
            ));
            return;
        }

        // Merchant trade: offer buy/sell instead of fetch quests.
        if npc.kind == game::npc::NpcKind::Merchant {
            self.buy_from_merchant();
            return;
        }

        // Otherwise hand the player a fresh fetch quest (seeded so it's stable
        // for this NPC until fulfilled).
        let pool: &[(ItemKind, u32, u32, ItemKind, u32)] = &[
            (ItemKind::Wood, 5, 15, ItemKind::Food, 3),
            (ItemKind::Stone, 4, 12, ItemKind::Food, 2),
            (ItemKind::Food, 3, 20, ItemKind::Gem, 1),
            (ItemKind::Herb, 3, 18, ItemKind::Food, 2),
            (ItemKind::Gem, 1, 30, ItemKind::Food, 3),
        ];
        let q = pool[(seed as usize) % pool.len()];
        npc.quest = Some(q);
        toast(&format!(
            "{}: \"Bring me {} {}. I'll pay {}xp and {} {}.\"",
            npc.name,
            q.1,
            q.0.name(),
            q.2,
            q.4,
            q.3.name()
        ));
        play_sfx("pickup");
    }

    /// Merchant trade: try to sell the player's most valuable sellable item for
    /// Gold. Each press sells one item. When nothing is left to sell, the
    /// merchant refuses.
    pub fn buy_from_merchant(&mut self) {
        // Sell order: most valuable first so the player earns Gold fastest.
        let sell_order = [
            ItemKind::IronPlate,
            ItemKind::Gem,
            ItemKind::Iron,
            ItemKind::Herb,
            ItemKind::Food,
            ItemKind::Wood,
            ItemKind::Stone,
        ];
        for &item in &sell_order {
            let price = game::trade::sell_price(item);
            if price > 0 && self.inventory.count(item) > 0 {
                self.inventory.remove(item, 1);
                self.inventory.add(ItemKind::Gold, price);
                play_sfx("pickup");
                toast(&format!(
                    "Sold {} for {} gold",
                    item.name(),
                    price
                ));
                return;
            }
        }
        toast("Merchant: \"Nothing to buy — bring me resources!\"");
    }

    /// Enter the building the player is standing next to, or leave the current
    /// interior. Bound to Enter.
    pub fn toggle_interior(&mut self) {
        if self.interior.is_some() {
            self.interior = None;
            toast("Left the building");
            return;
        }
        let (px, py) = (self.player.x, self.player.y);
        let mut best: Option<(f32, StructureKind)> = None;
        for s in &self.structures {
            if matches!(
                s.kind,
                StructureKind::House
                    | StructureKind::Cabin
                    | StructureKind::Hut
                    | StructureKind::Inn
                    | StructureKind::Dungeon
            ) {
                let dx = s.tx as f32 + 0.5 - px;
                let dy = s.ty as f32 + 0.5 - py;
                let d2 = dx * dx + dy * dy;
                if d2 < 3.0 && best.map_or(true, |(bd, _)| d2 < bd) {
                    best = Some((d2, s.kind));
                }
            }
        }
        match best {
            Some((_, kind)) => {
                let is_dungeon = kind == StructureKind::Dungeon;
                let max_floors = if kind == StructureKind::House { 2 } else { 1 };
                let name = match kind {
                    StructureKind::House => "House",
                    StructureKind::Cabin => "Cabin",
                    StructureKind::Hut => "Hut",
                    StructureKind::Inn => "Inn",
                    _ => "Dungeon",
                };
                // Dungeons seed a few spike traps on the floor; the vault loot is
                // claimed once when the player reaches the back wall.
                let hazards = if is_dungeon {
                    vec![(-1, -1), (1, 0), (0, 1), (-2, 1), (2, -1)]
                } else {
                    Vec::new()
                };
                self.interior = Some(Interior {
                    kind,
                    floor: 1,
                    max_floors,
                    rw: 3.5,
                    rh: 2.5,
                    px: 0.0,
                    py: 0.0,
                    bx: self.player.x,
                    by: self.player.y,
                    hazards,
                    loot_taken: false,
                });
                play_sfx("door");
                toast(&format!(
                    "Entered the {}{}{}",
                    name,
                    if max_floors > 1 { " — 2 floors (use the stairs)" } else { "" },
                    if is_dungeon { " — beware the traps!" } else { "" }
                ));
            }
            None => toast("Stand next to a house or dungeon to enter"),
        }
    }

    /// Per-frame logic while inside a building: walk the player around the room,
    /// exit via the door (right edge) and climb stairs (left edge) to the next floor.
    fn update_interior(&mut self, dt: f32) {
        let int = match self.interior.as_mut() {
            Some(i) => i,
            None => return,
        };
        let mut mx = 0.0f32;
        let mut my = 0.0f32;
        if self.keys[0] {
            my -= 1.0;
        }
        if self.keys[1] {
            my += 1.0;
        }
        if self.keys[2] {
            mx -= 1.0;
        }
        if self.keys[3] {
            mx += 1.0;
        }
        let len = (mx * mx + my * my).sqrt();
        if len > 0.0 {
            mx /= len;
            my /= len;
        }
        let sp = player::PLAYER_SPEED * 0.5 * dt;
        let mx2 = (int.rw - 0.5).max(0.5);
        let my2 = (int.rh - 0.5).max(0.5);
        int.px = (int.px + mx * sp).clamp(-mx2, mx2);
        int.py = (int.py + my * sp).clamp(-my2, my2);
        // Door on the right edge -> step out.
        if (int.px - mx2).abs() < 0.35 && int.py.abs() < 0.5 {
            self.interior = None;
            toast("Left the building");
            return;
        }
        // Stairs on the left edge -> climb to the next floor.
        if int.floor < int.max_floors && (int.px + mx2).abs() < 0.35 && int.py.abs() < 0.5 {
            int.floor += 1;
            int.px = 0.0;
            int.py = 0.0;
            play_sfx("door");
            toast(&format!("Climbed to floor {}", int.floor));
        }
        // Dungeon hazards: standing on a spike tile deals contact damage, and
        // reaching the back (left) wall center once cracks the vault for loot.
        if !int.hazards.is_empty() {
            let tx = int.px.round() as i32;
            let ty = int.py.round() as i32;
            if int.hazards.iter().any(|&(hx, hy)| hx == tx && hy == ty) {
                self.player.hp = (self.player.hp - 12.0 * dt).max(0.0);
                self.player.hurt_timer = 0.3;
                if self.player.hp <= 0.0 {
                    self.player.alive = false;
                }
            }
            if !int.loot_taken && (int.px + mx2).abs() < 0.5 && int.py.abs() < 0.5 {
                int.loot_taken = true;
                let r = ((int.bx as u32) ^ (int.by as u32).wrapping_mul(2654435761)) % 5;
                let reward: (ItemKind, u32) = match r {
                    0 => (ItemKind::Gem, 2),
                    1 => (ItemKind::Food, 4),
                    2 => (ItemKind::Herb, 3),
                    3 => (ItemKind::Wood, 6),
                    _ => (ItemKind::Stone, 6),
                };
                self.inventory.add(reward.0, reward.1);
                self.player.add_xp(25);
                play_sfx("pickup");
                toast(&format!(
                    "You cracked the vault! +{} {} and +25xp",
                    reward.1,
                    reward.0.name()
                ));
            }
        }
    }

    /// Build the sprite list for a building interior: a flat floor diamond, wall
    /// segments around the edges (with a gap for the door and the stairs), plus
    /// furniture and the player. Terrain is suppressed by passing an empty tile
    /// list to `build_tile_mesh` while inside.
    fn interior_sprites(&self, int: &Interior) -> Vec<Sprite> {
        let bx = int.bx;
        let by = int.by;
        let mut v = Vec::new();
        // Floor.
        let fw = (int.rw + 0.7) * 32.0;
        let fh = (int.rh + 0.7) * 16.0;
        v.push(
            Sprite::new_center(bx, by, [0.45, 0.33, 0.22], fw, fh, 0.0)
                .with_style(SpriteStyle::Floor),
        );
        let wall_col = [0.55, 0.45, 0.34];
        let n = 7;
        // Top & bottom edges.
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let ox = int.rw * (2.0 * t - 1.0);
            v.push(
                Sprite::new_center(bx + ox, by - int.rh, wall_col, 18.0, 44.0, 0.0)
                    .with_style(SpriteStyle::Wall),
            );
            v.push(
                Sprite::new_center(bx + ox, by + int.rh, wall_col, 18.0, 44.0, 0.0)
                    .with_style(SpriteStyle::Wall),
            );
        }
        // Left & right edges (skip the middle tile: left=stairs, right=door).
        for i in 1..n {
            let t = i as f32 / n as f32;
            let oy = int.rh * (2.0 * t - 1.0);
            if (t - 0.5).abs() > 0.12 {
                v.push(
                    Sprite::new_center(bx - int.rw, by + oy, wall_col, 18.0, 44.0, 0.0)
                        .with_style(SpriteStyle::Wall),
                );
            }
            if (t - 0.5).abs() > 0.12 {
                v.push(
                    Sprite::new_center(bx + int.rw, by + oy, wall_col, 18.0, 44.0, 0.0)
                        .with_style(SpriteStyle::Wall),
                );
            }
        }
        // Door opening (right middle): a dark floor diamond.
        v.push(
            Sprite::new_center(bx + int.rw, by, [0.22, 0.16, 0.11], 16.0, 36.0, 0.0)
                .with_style(SpriteStyle::Floor),
        );
        // Stairs (left middle) if the building has more floors.
        if int.floor < int.max_floors {
            v.push(
                Sprite::new_center(bx - int.rw, by, [0.55, 0.45, 0.28], 22.0, 24.0, 0.0)
                    .with_style(SpriteStyle::Floor),
            );
            v.push(
                Sprite::new_center(bx - int.rw, by, [0.7, 0.6, 0.4], 12.0, 18.0, 20.0)
                    .with_style(SpriteStyle::Wall),
            );
        }
        // Furniture.
        v.push(
            Sprite::new_center(bx - int.rw + 0.8, by - int.rh + 0.8, [0.8, 0.75, 0.6], 18.0, 12.0, 0.0)
                .with_style(SpriteStyle::Bed),
        );
        v.push(
            Sprite::new_center(bx + int.rw - 0.8, by + int.rh - 0.8, [0.6, 0.45, 0.3], 16.0, 18.0, 0.0)
                .with_style(SpriteStyle::Crate),
        );
        v.push(
            Sprite::new_center(bx + int.rw - 0.8, by - int.rh + 0.8, [0.5, 0.4, 0.3], 14.0, 20.0, 0.0)
                .with_style(SpriteStyle::Barrel),
        );
        v.push(
            Sprite::new_center(bx - int.rw + 0.8, by + int.rh - 0.8, [0.9, 0.8, 0.4], 10.0, 22.0, 0.0)
                .with_style(SpriteStyle::Lantern),
        );
        // Dungeon spike traps (room-relative tiles) and the sealed vault chest
        // at the back wall until it has been looted.
        for &(hx, hy) in &int.hazards {
            v.push(
                Sprite::new_center(bx + hx as f32, by + hy as f32, [0.72, 0.72, 0.78], 12.0, 14.0, 0.0)
                    .with_style(SpriteStyle::Spike),
            );
        }
        if int.kind == StructureKind::Dungeon && !int.loot_taken {
            v.push(
                Sprite::new_center(bx - int.rw + 0.8, by, [0.95, 0.8, 0.3], 16.0, 18.0, 0.0)
                    .with_style(SpriteStyle::Chest),
            );
        }
        // Player (sits on the floor — no lift).
        v.push(
            Sprite::new_center(bx + int.px, by + int.py, [0.45, 0.55, 0.85], 16.0, 22.0, 0.0)
                .with_style(SpriteStyle::Humanoid),
        );
        v
    }

    /// True when the player is next to a Well (or shoreline) to drink from.
    pub fn near_water(&mut self) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        if self.structures.iter().any(|s| {
            s.kind == StructureKind::Well
                && (s.tx as f32 + 0.5 - px).abs() <= 1.6
                && (s.ty as f32 + 0.5 - py).abs() <= 1.6
        }) {
            return true;
        }
        // Standing on a shoreline (shallow/water edge) also lets you drink.
        let tx = px.floor() as i32;
        let ty = py.floor() as i32;
        tile_at(&self.world, &mut self.chunks, tx, ty).wadable()
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
        play_sfx("craft");
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
        self.net_dodge = true;
        let dir = if let Some((ax, ay)) = self.analog {
            let len = (ax * ax + ay * ay).sqrt();
            if len < 1e-4 {
                (0.0, 0.0)
            } else {
                (ax / len, ay / len)
            }
        } else {
            player::input_dir(self.keys[0], self.keys[1], self.keys[2], self.keys[3])
        };
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

    /// Attack with the equipped weapon. Melee weapons swing (hits everything in
    /// reach); ranged weapons (Bow) loose an arrow instead. Honors the weapon's
    /// cooldown so heavier weapons swing slower.
    pub fn attack(&mut self) {
        if self.swing_cd > 0.0 {
            return;
        }
        let w = self.player.weapon;
        self.swing_cd = w.cooldown();
        if w.ranged() {
            self.net_shoot = true;
            if !self.player.spend_stamina(4.0) {
                return;
            }
            self.debug_shots += 1;
            let mut a = Arrow::new(self.player.x, self.player.y, self.player.facing.0, self.player.facing.1);
            a.damage = self.player.weapon_damage();
            self.arrows.push(a);
            play_sfx("shoot");
            // small recoil puff
            let (fx, fy) = self.player.facing;
            let flen = (fx * fx + fy * fy).sqrt().max(0.01);
            let cx = self.player.x + fx / flen * 0.6;
            let cy = self.player.y + fy / flen * 0.6;
            self.spawn_particles(cx, cy, [1.0, 0.95, 0.6], 3, 26.0, 0.15, 2.0);
        } else {
            self.net_atk = true;
            if !self.player.spend_stamina(6.0) {
                return;
            }
            play_sfx("swing");
            self.debug_attacks += 1;
            let mut hits = swing_hits(&self.player, self.enemies.enemies_mut(), w.reach());
            self.debug_swing_hits += hits.len() as u32;
            let mut sparks = Vec::new();
            for e in &mut hits {
                // Weak-point bonus: the Bestiary tells you which weapon a foe fears.
                let dmg = self.player.weapon_damage() * e.kind.weakness_to(w);
                e.take_damage(dmg);
                // Knock the struck enemy back along the player->enemy vector.
                let dx = e.x - self.player.x;
                let dy = e.y - self.player.y;
                let len = (dx * dx + dy * dy).sqrt().max(0.01);
                e.x += dx / len * 0.35;
                e.y += dy / len * 0.35;
                sparks.push((e.x, e.y));
            }
            if !hits.is_empty() {
                play_sfx("hit");
                self.hitstop = 0.06;
            }
            drop(hits);
            let tint = w.color();
            let spark = [
                (tint[0] * 0.5 + 0.5).min(1.0),
                (tint[1] * 0.5 + 0.5).min(1.0),
                (tint[2] * 0.5 + 0.5).min(1.0),
            ];
            for (x, y) in sparks {
                self.spawn_particles(x, y, spark, 7, 55.0, 0.35, 3.5);
            }
            // Swept slash: a short arc of particles, tinted by the weapon, in front
            // of the player — so each weapon reads differently on screen.
            let (fx, fy) = self.player.facing;
            let flen = (fx * fx + fy * fy).sqrt().max(0.01);
            let cx = self.player.x + fx / flen * w.reach() * 0.5;
            let cy = self.player.y + fy / flen * w.reach() * 0.5;
            self.spawn_particles(cx, cy, tint, 6, 40.0, 0.18, 2.5);
            self.sweep_dead();
        }
    }

    /// Recompute the HUD compass target: the nearest still-hostile Crown
    /// Fragment guardian, else the altar once all five are in hand, else the
    /// general direction of the next fragment's biome. Throttled to ~1 Hz.
    fn update_objective(&mut self, dt: f32) {
        if self.ending.is_some() {
            self.objective = None;
            return;
        }
        self.obj_timer -= dt;
        if self.obj_timer > 0.0 {
            return;
        }
        self.obj_timer = 1.0;

        // 1) a fragment guardian that is currently alive takes priority
        let mut best: Option<(f32, (f32, f32))> = None;
        for e in self.enemies.enemies() {
            if e.kind.fragment_bit().is_some() && e.alive() {
                let d = (e.x - self.player.x).hypot(e.y - self.player.y);
                if best.map_or(true, |(bd, _)| d < bd) {
                    best = Some((d, (e.x, e.y)));
                }
            }
        }
        if let Some((_, p)) = best {
            self.objective = Some(p);
            return;
        }
        // 2) once all five are recovered, point home to the reforging altar
        if self.fragments == 0b11111 {
            if let Some((ax, ay)) = self.altar_tile {
                self.objective = Some((ax as f32 + 0.5, ay as f32 + 0.5));
            }
            return;
        }
        // 3) otherwise aim toward the next fragment's biome (sampled ring scan)
        if let Some(k) = next_fragment_boss(self.fragments) {
            if let Some(t) = self.biome_center(k.fragment_bit().unwrap()) {
                self.objective = Some(t);
                return;
            }
        }
        self.objective = None;
    }

    /// Find a tile of the given fragment's home biome by sampling outward rings
    /// from the player (cheap: a handful of probes per ring, capped radius).
    fn biome_center(&mut self, bit: u8) -> Option<(f32, f32)> {
        let tk = match bit {
            0 => TileKind::Forest,
            1 => TileKind::Desert,
            2 => TileKind::Snow,
            3 => TileKind::Swamp,
            4 => TileKind::Water,
            _ => return None,
        };
        let px = self.player.x;
        let py = self.player.y;
        for r in (8..400).step_by(4) {
            for (dx, dy) in [
                (r, 0),
                (-r, 0),
                (0, r),
                (0, -r),
                (r, r),
                (r, -r),
                (-r, r),
                (-r, -r),
            ] {
                let x = px + dx as f32;
                let y = py + dy as f32;
                if tile_at(&self.world, &mut self.chunks, x.floor() as i32, y.floor() as i32) == tk {
                    return Some((x, y));
                }
            }
        }
        None
    }

    /// Resolve kills: drop loot and start respawn timers.
    fn sweep_dead(&mut self) {
        let drops: Vec<((i32, i32), f32, f32, EnemyKind, Vec<ItemKind>, f32)> = self
            .enemies
            .iter_mut_with_key()
            .filter(|(_, e)| !e.alive())
            .map(|((tx, ty), e)| ((tx, ty), e.x, e.y, e.kind, e.drops(), e.elite))
            .collect();
        for ((tx, ty), ex, ey, kind, items, elite) in drops {
            play_sfx("enemydie");
            // Death puff: a quick scatter of loot-colored sparks so kills read
            // clearly. Raiders (the nocturnal raiders) burst red, with a second
            // gold "loot scatter" ring to suggest items flying out of a fallen foe.
            if matches!(kind, EnemyKind::Raider) {
                self.spawn_particles(ex, ey, [0.85, 0.2, 0.18], 10, 44.0, 0.32, 3.2);
            } else {
                self.spawn_particles(ex, ey, [1.0, 0.85, 0.4], 6, 38.0, 0.28, 3.0);
            }
            self.spawn_particles(ex, ey, [1.0, 0.9, 0.55], 5, 22.0, 0.45, 2.5);
            // Drop loot on the ground at the enemy's position; the player walks
            // over it to collect (see collect_loot). Spread multiple drops in a
            // small ring so they don't perfectly overlap.
            let n = items.len().max(1) as f32;
            for (i, it) in items.iter().enumerate() {
                let ang = (i as f32 / n) * std::f32::consts::TAU;
                self.loot.push(LootDrop {
                    kind: *it,
                    x: ex + ang.cos() * 0.25,
                    y: ey + ang.sin() * 0.25,
                    count: 1,
                    ttl: 60.0,
                    phase: (ex + ey).fract().abs() * std::f32::consts::TAU,
                });
            }
            match kind {
                EnemyKind::Slime => self.slimes_killed += 1,
                EnemyKind::Boss => {
                    self.boss_killed += 1;
                    play_sfx("victory");
                }
                EnemyKind::Colossus => {
                    self.colossus_killed += 1;
                    play_sfx("victory");
                }
                _ => {}
            }
            // Crown Fragment guardians each guard one of the five fragments. On
            // defeat, record which fragment was recovered (idempotent) and surface
            // a line of the lost empire's lore.
            if let Some(bit) = kind.fragment_bit() {
                if self.fragments & (1 << bit) == 0 {
                    self.fragments |= 1 << bit;
                    let got = self.fragments.count_ones();
                    play_sfx("victory");
                    toast(&format!(
                        "Crown Fragment recovered! ({}/5) — {}",
                        got,
                        fragment_lore(bit)
                    ));
                }
            }
            // Experience + level-up for the player (elite enemies pay out more).
            let lvl_before = self.player.level;
            self.player.add_xp((kind.xp() as f32 * elite) as u32);
            if self.player.level > lvl_before {
                play_sfx("levelup");
                toast(&format!("Level up! You are now level {}", self.player.level));
            }
            // bosses never respawn; slimes return after 15s
            let respawn = if matches!(kind, EnemyKind::Boss | EnemyKind::Colossus) { f32::MAX } else { 15.0 };
            self.enemies.kill(tx, ty, respawn);
            // Occasional weapon drop: a findable weapon on the ground. Heavier
            // weapons are rarer (handled in roll_drop_with).
            let roll = (((ex * 53.0 + ey * 31.0 + self.debug_attacks as f32 * 7.0) as i32) as u32) % 100;
            if let Some(wk) = game::weapons::WeaponKind::roll_drop_with(roll) {
                self.weapon_loot.push(WeaponDrop {
                    kind: wk,
                    x: ex,
                    y: ey,
                    ttl: 90.0,
                    phase: (ex + ey).fract().abs() * std::f32::consts::TAU,
                });
            }
        }
    }

    /// Auto-collect nearby ground loot and expire old drops.
    fn collect_loot(&mut self, dt: f32) {
        let px = self.player.x;
        let py = self.player.y;
        let mut i = 0;
        while i < self.loot.len() {
            let l = &mut self.loot[i];
            l.ttl -= dt;
            l.phase += dt * 3.0;
            if l.ttl <= 0.0 {
                self.loot.remove(i);
                continue;
            }
            let dx = l.x - px;
            let dy = l.y - py;
            if dx * dx + dy * dy < 1.1 * 1.1 {
                self.inventory.add(l.kind, l.count);
                play_sfx("pickup");
                self.loot.remove(i);
                continue;
            }
            i += 1;
        }
        // Weapons lying on the ground: bob, expire, and get equipped on contact.
        let px = self.player.x;
        let py = self.player.y;
        let mut j = 0;
        while j < self.weapon_loot.len() {
            let w = &mut self.weapon_loot[j];
            w.ttl -= dt;
            w.phase += dt * 3.0;
            if w.ttl <= 0.0 {
                self.weapon_loot.remove(j);
                continue;
            }
            let dx = w.x - px;
            let dy = w.y - py;
            if dx * dx + dy * dy < 1.1 * 1.1 {
                let k = w.kind;
                self.player.equip_weapon(k);
                toast(&format!("Found {}!", k.name()));
                play_sfx("pickup");
                self.weapon_loot.remove(j);
                continue;
            }
            j += 1;
        }
    }

    /// Begin the portal trip to the town: start the loading overlay; actual
    /// teleport + (first-time) build animation happen when the timer elapses.
    fn use_portal(&mut self) {
        if self.town_transition > 0.0 {
            return;
        }
        if self.towns.is_empty() {
            return;
        }
        self.town_transition = TOWN_LOAD_TIME;
        play_sfx("door");
        toast("The portal hums — travelling to the city…");
    }

    /// True if `(tx, ty)` lies within the walled town's footprint (so its
    /// buildings can be gated by the build-in animation).
    fn is_town_tile(&self, tx: i32, ty: i32) -> bool {
        self.towns
            .iter()
            .any(|&(cx, cy, _)| (tx - cx).abs() <= 14 && (ty - cy).abs() <= 14)
    }

    fn harvest(&mut self) {
        self.net_harvest = true;
        // Village portal: standing on the gate and pressing E whisks you to the
        // walled town, playing a "the city is being built" animation on arrival.
        if let Some((px, py)) = self.portal {
            let d = (self.player.x - px).hypot(self.player.y - py);
            if d < 1.8 {
                self.use_portal();
                return;
            }
        }
        // Fast travel: standing on a settlement signpost cycles to the next
        // settlement (villages + towns together), so you can traverse the world
        // without walking the long distances.
        let ptx = self.player.x.floor() as i32;
        let pty = self.player.y.floor() as i32;
        let mut dests: Vec<(i32, i32, String)> = self
            .villages
            .iter()
            .cloned()
            .chain(self.towns.iter().cloned())
            .collect();
        if let Some(idx) = dests.iter().position(|(x, y, _)| *x == ptx && *y == pty) {
            let nxt = dests[(idx + 1) % dests.len()].clone();
            self.player.x = nxt.0 as f32 + 0.5;
            self.player.y = nxt.1 as f32 + 0.5;
            self.spawn_point = (self.player.x, self.player.y);
            self.interior = None;
            toast(&format!("Fast-travelled to {}", nxt.2));
            play_sfx("door");
            return;
        }
        // Reforge at the altar when all five Crown Fragments are in hand: this
        // arms the choice; the HUD shows Reign/Shatter and forwards the pick to
        // reforge().
        if self.ending.is_none() {
            if let Some((ax, ay)) = self.altar_tile {
                let d = (self.player.x - (ax as f32 + 0.5))
                    .abs()
                    .max((self.player.y - (ay as f32 + 0.5)).abs());
                if d <= CHEST_RANGE {
                    if self.fragments == 0b11111 {
                        self.ending_pending = true;
                        return;
                    } else if !self.altar_hinted {
                        let left = 5 - self.fragments.count_ones();
                        toast(&format!(
                            "The altar is cold. {left} Crown Fragment{} still lost to the world.",
                            if left == 1 { "" } else { "s" }
                        ));
                        self.altar_hinted = true;
                    }
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
                        play_sfx("harvest");
                        *cd = 30.0;
                        return;
                    }
                }
            }
        }
        if let Some((tx, ty, kind)) = self.nearest_resource() {
            if let Some(item) = self.nodes.chop(tx, ty, kind) {
                play_sfx("harvest");
                // Honed Tools (crafted at an Anvil) yield bonus resources. Drop
                // the yield as ground loot (collected on proximity) rather than
                // crediting inventory directly, so harvesting reads like looting.
                let n = 1 + self.craft_harvest;
                let (lx, ly) = (tx as f32 + 0.5, ty as f32 + 0.5);
                self.loot.push(LootDrop {
                    kind: item,
                    x: lx,
                    y: ly,
                    count: n,
                    ttl: 60.0,
                    phase: (lx + ly).fract().abs() * std::f32::consts::TAU,
                });
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
            play_sfx("chest");
            self.inventory.add(ItemKind::Food, 2);
            self.inventory.add(ItemKind::Wood, 2);
            self.inventory.add(ItemKind::Stone, 1);
            // Chests often hide a weapon to find (and equip).
            let roll = ((tx as u32 * 31 + ty as u32 * 17) % 100) as u32;
            if let Some(wk) = game::weapons::WeaponKind::roll_drop_with(roll) {
                self.player.equip_weapon(wk);
                toast(&format!("Chest contained a {}!", wk.name()));
            }
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
        // Tech tree: advanced structures require prerequisites so progression
        // has a shape. Anvil unlocks support/turret tech; forging iron at the
        // anvil (any weapon/plate) unlocks the Turret specifically.
        if matches!(kind, StructureKind::HealingTotem | StructureKind::Turret) && !self.has_anvil() {
            toast("Build an Anvil before advanced structures");
            return;
        }
        if kind == StructureKind::Turret && !self.crafted_iron {
            toast("Forge iron at an anvil first (craft any weapon/plate)");
            return;
        }
        // Latch a networked build intent (server validates range + cost).
        let (bk, btx, bty) = if let Some((gk, gx, gy, valid)) = self.build_ghost {
            if gk == kind && valid {
                (kind, gx, gy)
            } else {
                (kind, self.player.x.floor() as i32, self.player.y.floor() as i32)
            }
        } else {
            (kind, self.player.x.floor() as i32, self.player.y.floor() as i32)
        };
        self.net_build = Some((bk, btx, bty));
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
        let mut rested = false;
        // Inside a building: rest at the bed furniture (left-top of the room).
        if let Some(int) = &self.interior {
            let bedx = int.bx - int.rw + 0.8;
            let bedy = int.by - int.rh + 0.8;
            let dx = bedx - (int.bx + int.px);
            let dy = bedy - (int.by + int.py);
            if dx * dx + dy * dy < 1.2 {
                rested = true;
            }
        }
        // Outside: rest at a placed bed structure.
        if !rested {
            rested = self.structures.iter().any(|s| {
                s.kind == StructureKind::Bed
                    && {
                        let dx = s.tx as f32 + 0.5 - self.player.x;
                        let dy = s.ty as f32 + 0.5 - self.player.y;
                        dx * dx + dy * dy < 4.0
                    }
            });
        }
        if rested {
            self.time_of_day = 0.32; // wake ~07:40 with daylight climbing
            self.player.hunger = (self.player.hunger + 30.0).min(100.0);
            self.player.thirst = (self.player.thirst + 30.0).min(100.0);
            self.player.hp = (self.player.hp + 40.0).min(self.player.max_hp());
            play_sfx("sleep");
        } else {
            toast("No bed nearby to rest");
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
        // Villages: a few named hamlets of houses (which double as shelters) plus
        // a sign, an anvil and a well so each is a functional safe haven.
        self.villages.clear();
        self.npcs.clear();
        let sites = village_sites(seed, 3, |tx, ty| {
            tile_at(&self.world, &mut self.chunks, tx, ty).walkable()
        });
        // Capture the first village so we can (re)spawn the player inside a
        // settlement even after `sites` is consumed by the generation loop below.
        let first_village = sites.first().copied();
        let house_kinds = [
            StructureKind::House,
            StructureKind::Cabin,
            StructureKind::Hut,
            StructureKind::Inn,
            StructureKind::Barn,
            StructureKind::Watchtower,
        ];
        let ring: [(i32, i32); 12] = [
            (3, 0),
            (-3, 0),
            (0, 3),
            (0, -3),
            (3, 3),
            (-3, -3),
            (3, -3),
            (-3, 3),
            (5, 0),
            (-5, 0),
            (0, 5),
            (0, -5),
        ];
        const FIRST_NAMES: &[&str] = &[
            "Bryn", "Cael", "Dora", "Edda", "Finn", "Greta", "Hale", "Ivo", "Jora", "Kell",
            "Lia", "Mira", "Nils", "Orin", "Petra", "Quill", "Rowan", "Sefa", "Tobias", "Ulla",
        ];
        for (vx, vy) in sites {
            let name = village_name(vx, vy);
            self.villages.push((vx, vy, name.clone()));
            structures.push(Structure { tx: vx, ty: vy, kind: StructureKind::Sign });
            for (i, (dx, dy)) in ring.iter().enumerate() {
                let hx = vx + dx;
                let hy = vy + dy;
                if tile_at(&self.world, &mut self.chunks, hx, hy).walkable() {
                    structures.push(Structure {
                        tx: hx,
                        ty: hy,
                        kind: house_kinds[i % 3],
                    });
                }
            }
            if tile_at(&self.world, &mut self.chunks, vx + 2, vy).walkable() {
                structures.push(Structure { tx: vx + 2, ty: vy, kind: StructureKind::Anvil });
            }
            if tile_at(&self.world, &mut self.chunks, vx - 2, vy).walkable() {
                structures.push(Structure { tx: vx - 2, ty: vy, kind: StructureKind::Well });
            }
            // Populate the hamlet: a guard by the sign, a stone golem defender,
            // a merchant by the well, and a few villagers among the houses.
            let c = (vx as f32 + 0.5, vy as f32 + 0.5);
            self.npcs.push(Npc::new(NpcKind::Guard, c.0, c.1, format!("Guard of {}", name), (vx as f32, vy as f32)));
            self.npcs.push(Npc::new(
                NpcKind::Golem,
                c.0 + 1.2,
                c.1 + 0.4,
                format!("{} the Golem", name),
                (vx as f32, vy as f32),
            ));
            self.npcs.push(Npc::new(
                NpcKind::Merchant,
                vx as f32 + 0.5,
                vy as f32 + 2.0 + 0.5,
                format!("{} the Merchant", FIRST_NAMES[(vx.wrapping_abs() as usize) % FIRST_NAMES.len()]),
                (vx as f32, vy as f32 + 2.0),
            ));
            for k in 0..4 {
                // Villagers mingle on the inner tiles (between the houses), not on
                // the house tiles themselves.
                let offs = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
                let hx = vx + offs[k].0;
                let hy = vy + offs[k].1;
                let nm = FIRST_NAMES[(vx.wrapping_abs() as usize + k as usize * 3) % FIRST_NAMES.len()].to_string();
                self.npcs.push(Npc::new(
                    NpcKind::Villager,
                    hx as f32 + 0.5,
                    hy as f32 + 0.5,
                    nm,
                    (hx as f32, hy as f32),
                ));
            }
        }

        // Village portal: a glowing arcane gate in the first hamlet that travels
        // to the walled town. Placed just south of the sign on walkable ground.
        if let Some((fvx, fvy)) = first_village {
            let spots = [(fvx, fvy - 2), (fvx, fvy + 2), (fvx + 2, fvy), (fvx - 2, fvy)];
            if let Some(&(px, py)) = spots
                .iter()
                .find(|&&(x, y)| tile_at(&self.world, &mut self.chunks, x, y).walkable())
            {
                structures.push(Structure { tx: px, ty: py, kind: StructureKind::Portal });
                self.portal = Some((px as f32 + 0.5, py as f32 + 0.5));
            }
        }

        // ---------------------------------------------------------------------
        // Town / city: a walled settlement far from spawn, crossed by an old
        // railway with a parked train and a few abandoned cars. Villages are left
        // open; only the town gets a wall boundary (with gated gaps).
        // ---------------------------------------------------------------------
        self.towns.clear();
        let (tx0, ty0) = town_site(seed, |tx, ty| tile_at(&self.world, &mut self.chunks, tx, ty).walkable());
        let town_nm = town_name(tx0, ty0);
        self.towns.push((tx0, ty0, town_nm.clone()));
        let r = 14;
        // Wall boundary ring with a 2-tile gate centered on each side.
        for x in (tx0 - r)..=(tx0 + r) {
            for y in (ty0 - r)..=(ty0 + r) {
                let edge = x == tx0 - r || x == tx0 + r || y == ty0 - r || y == ty0 + r;
                if !edge {
                    continue;
                }
                let gate = ((x == tx0 - r || x == tx0 + r) && (y - ty0).abs() <= 1)
                    || ((y == ty0 - r || y == ty0 + r) && (x - tx0).abs() <= 1);
                if gate {
                    continue;
                }
                if tile_at(&self.world, &mut self.chunks, x, y).walkable() {
                    structures.push(Structure { tx: x, ty: y, kind: StructureKind::Wall });
                }
            }
        }
        // Central plaza marker.
        structures.push(Structure { tx: tx0, ty: ty0, kind: StructureKind::Sign });
        // Railway: a horizontal run of rails across the town (train sits at center).
        let rail_y = ty0 + 4;
        for x in (tx0 - r + 1)..=(tx0 + r - 1) {
            if tile_at(&self.world, &mut self.chunks, x, rail_y).walkable() {
                structures.push(Structure { tx: x, ty: rail_y, kind: StructureKind::Rail });
            }
        }
        structures.push(Structure { tx: tx0, ty: rail_y, kind: StructureKind::Train });
        // Buildings on a grid, skipping the plaza and the rail row. Mix in inns,
        // barns and a watchtower so the town reads as a real settlement.
        let bkinds = [
            StructureKind::House,
            StructureKind::Cabin,
            StructureKind::Hut,
            StructureKind::Inn,
            StructureKind::Barn,
            StructureKind::Watchtower,
        ];
        let mut bi = 0usize;
        for gx in (tx0 - 10..=tx0 + 10).step_by(5) {
            for gy in (ty0 - 10..=ty0 + 10).step_by(5) {
                if (gx - tx0).abs() <= 2 && (gy - ty0).abs() <= 2 {
                    continue;
                }
                if gy == rail_y {
                    continue;
                }
                if tile_at(&self.world, &mut self.chunks, gx, gy).walkable() {
                    let k = bkinds[bi % bkinds.len()];
                    bi += 1;
                    structures.push(Structure { tx: gx, ty: gy, kind: k });
                }
            }
        }
        // A few parked old cars along the streets (deterministic scatter).
        let mut h = ((tx0 as u32) ^ (ty0 as u32)).wrapping_mul(2654435761);
        for _ in 0..8 {
            h = h.wrapping_mul(1664525).wrapping_add(1013904223);
            let cx = tx0 - 10 + (((h as i32) % 21i32).abs());
            let cy = ty0 - 10 + (((h >> 8) as i32) % 21i32).abs();
            if (cx - tx0).abs() <= 1 && (cy - ty0).abs() <= 1 {
                continue;
            }
            if cy == rail_y {
                continue;
            }
            if tile_at(&self.world, &mut self.chunks, cx, cy).walkable()
                && !structures.iter().any(|s| s.tx == cx && s.ty == cy)
            {
                structures.push(Structure { tx: cx, ty: cy, kind: StructureKind::Car });
            }
        }
        // Populate the town with citizens: guards at the gates, a merchant, villagers.
        let tc = (tx0 as f32 + 0.5, ty0 as f32 + 0.5);
        self.npcs.push(Npc::new(
            NpcKind::Guard,
            tx0 as f32 - r as f32 + 1.5,
            ty0 as f32,
            format!("Gate Guard of {}", town_nm),
            (tx0 as f32 - r as f32 + 1.0, ty0 as f32),
        ));
        self.npcs.push(Npc::new(
            NpcKind::Merchant,
            tx0 as f32 + 0.5,
            ty0 as f32 + 2.5,
            format!("{} the Trader", FIRST_NAMES[(tx0.unsigned_abs() as usize) % FIRST_NAMES.len()]),
            tc,
        ));
        for k in 0..6 {
            let hx = tx0 + ring[k % 8].0;
            let hy = ty0 + ring[k % 8].1;
            let nm = FIRST_NAMES[(tx0.unsigned_abs() as usize + k as usize * 5) % FIRST_NAMES.len()].to_string();
            self.npcs.push(Npc::new(
                NpcKind::Villager,
                hx as f32 + 0.5,
                hy as f32 + 0.5,
                nm,
                (hx as f32, hy as f32),
            ));
        }

        // Record the town's generated layout so it can be persisted and revealed
        // progressively (the "town is being built" animation) on first arrival.
        let r = 14;
        self.town_structures = structures
            .iter()
            .filter(|s| (s.tx - tx0).abs() <= r && (s.ty - ty0).abs() <= r)
            .map(|s| (s.tx, s.ty, s.kind))
            .collect();
        // A freshly rolled world hasn't been visited yet, so the creation animation
        // will play the first time the player steps through the portal.
        self.town_visited = false;
        self.town_build_t = 1.0;

        // Always start the player inside a settlement (a village first, else the town).
        let start = first_village.unwrap_or((tx0, ty0));
        let (spx, spy) = (start.0 as f32 + 0.5, start.1 as f32 + 0.5);
        self.spawn_point = (spx, spy);
        self.player = Player::new(spx, spy);

        self.structures = structures;
        // Scatter a few dungeon entrances across the world, away from the spawn
        // village and the ruins, so exploration has a deadly payoff.
        {
            let sp = (self.spawn_point.0 as i32, self.spawn_point.1 as i32);
            let mut h = (seed ^ 0x9e37_79b9).wrapping_mul(2654435761);
            let mut placed = 0;
            for _ in 0..240 {
                h = h.wrapping_mul(1664525).wrapping_add(1013904223);
                let tx = sp.0 + ((h as i32) % 160) - 80;
                let ty = sp.1 + (((h >> 11) as i32) % 160) - 80;
                if (tx - sp.0).abs() < 30 && (ty - sp.1).abs() < 30 {
                    continue;
                }
                if (tx - self.ruins.0).abs() < 12 && (ty - self.ruins.1).abs() < 12 {
                    continue;
                }
                if tile_at(&self.world, &mut self.chunks, tx, ty).walkable()
                    && !self.structures.iter().any(|s| s.tx == tx && s.ty == ty)
                {
                    self.structures.push(Structure { tx, ty, kind: StructureKind::Dungeon });
                    placed += 1;
                    if placed >= 5 {
                        break;
                    }
                }
            }
        }
        self.quest = QuestLog::new();
        self.boss_killed = 0;
        self.colossus_killed = 0;
        self.fragments = 0;
        self.discovered.clear();
        self.weather = 0;
        self.weather_timer = 25.0;
        self.elite_timer = 60.0;
        self.raider_timer = 75.0;
        self.boss_spawned = false;
        self.altar_placed = false;
        self.altar_tile = None;
        self.near_altar = false;
        self.altar_hinted = false;
        self.ending_pending = false;
        self.ending = None;
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
        if self.ending.is_some() || self.fragments != 0b11111 {
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
        self.turret_cd.clear();
        self.crafted_iron = false;
        self.loot.clear();
        self.weapon_loot.clear();
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
                xp: self.player.xp,
                level: self.player.level,
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
            fragments: self.fragments,
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
            weapon: self.player.weapon.as_u8(),
            weapon_unlocked: self.player.unlocked,
            enchant: self.player.enchant,
            town: if self.town_structures.is_empty() {
                None
            } else {
                Some(self.town_structures.clone())
            },
            town_visited: self.town_visited,
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
        self.player.xp = s.player.xp;
        self.player.level = s.player.level;

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
        self.fragments = s.fragments;
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
        self.player.weapon = game::weapons::WeaponKind::from_u8(s.weapon);
        self.player.unlocked = s.weapon_unlocked;
        self.player.enchant = s.enchant;
        // Restore the persisted town so it cannot be re-rolled into something new:
        // the captured layout (and the "already visited" flag) survive the reload.
        self.town_visited = s.town_visited;
        if let Some(town) = &s.town {
            self.town_structures = town.clone();
        }
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
        let user_cap = get_render_cap();
        // Highest-quality ladder level allowed by the user's render cap.
        self.max_res_level = RES_LEVELS
            .iter()
            .enumerate()
            .filter(|(_, (w, h))| *w <= user_cap.0 && *h <= user_cap.1)
            .map(|(i, _)| i)
            .next()
            .unwrap_or(0);
        if self.res_level < self.max_res_level {
            self.res_level = self.max_res_level;
        }
        // Effective internal resolution: the chosen ladder level, never above the
        // user's cap.
        let (lvl_w, lvl_h) = RES_LEVELS[self.res_level.min(RES_LEVELS.len() - 1)];
        let (cap_w, cap_h) = (lvl_w.min(user_cap.0), lvl_h.min(user_cap.1));
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
        // While inside a building we run a separate, lighter simulation: just the
        // room walk + stairs. The world keeps ticking for remote players via the
        // network step below, but local combat/survival is paused indoors.
        if self.interior.is_some() {
            self.update_interior(dt);
            return;
        }
        self.frames += 1;
        self.hurt_flash = (self.hurt_flash * 0.86).max(0.0);
        self.swing_cd = (self.swing_cd - dt).max(0.0);
        // Expose swing progress (0..1 over the weapon's cooldown) so the world
        // renderer can lunge the player's torso/arms on a strike.
        let cd = self.player.weapon.cooldown().max(0.05);
        self.player.swing_t = if self.swing_cd > 0.0 {
            1.0 - self.swing_cd / cd
        } else {
            0.0
        };
        // Decay any active screen-shake (set when a boss lands a hit).
        self.shake = (self.shake * 0.82).max(0.0);
        // Minimal screen-shake: when a boss lands a blow on the player, nudge the
        // camera a touch. Re-armed only on the frame the hit lands (hurt_timer
        // rising edge) and only if a boss is actually in melee range.
        let hurt_now = self.player.hurt_timer;
        if hurt_now > 0.0 && self.prev_hurt <= 0.0 {
            let px = self.player.x;
            let py = self.player.y;
            let boss_near = self
                .enemies
                .iter_mut_with_key()
                .any(|(_, e)| {
                    (e.kind == EnemyKind::Boss || e.kind == EnemyKind::Colossus)
                        && ((e.x - px).powi(2) + (e.y - py).powi(2)).sqrt() < 2.2
                });
            if boss_near {
                self.shake = (self.shake + 0.35).min(0.5);
            }
        }
        self.prev_hurt = hurt_now;
        // Hit-stop: briefly scale the simulation dt toward zero so landed blows
        // land with a satisfying snap. Rendering continues at full rate.
        let real_dt = dt;
        let dt = if self.hitstop > 0.0 {
            self.hitstop = (self.hitstop - real_dt).max(0.0);
            real_dt * 0.12
        } else {
            real_dt
        };

        // Adaptive resolution: keep fps high on backends with a slow present /
        // readback path (e.g. default Linux Chrome without Vulkan). We measure
        // the real step rate (EMA) and step the internal resolution down when
        // it sags, back up when there's headroom. Hysteresis avoids oscillation.
        // Disabled via the settings menu: resolution stays at the user's cap.
        let adaptive = *ADAPTIVE_RES.lock().unwrap();
        if adaptive {
            let inst_fps = 1.0 / dt.max(0.001);
            self.fps_est = self.fps_est * 0.9 + inst_fps * 0.1;
            self.res_timer += dt;
            if self.res_timer >= 1.5 {
                self.res_timer = 0.0;
                if self.fps_est < 50.0 && self.res_level < RES_LEVELS.len() - 1 {
                    self.res_level += 1;
                    self.resize();
                } else if self.fps_est > 75.0 && self.res_level > self.max_res_level {
                    self.res_level -= 1;
                    self.resize();
                }
            }
        } else if self.res_level != self.max_res_level {
            self.res_level = self.max_res_level;
            self.resize();
        }

        self.ensure_visible();
        self.time_of_day = (self.time_of_day + dt / DAY_LENGTH).rem_euclid(1.0);
        self.anim_clock = (self.anim_clock + dt).rem_euclid(3600.0);

        // Portal trip: count down the "city is being built" loading overlay, then
        // arrive (teleport) and, on the first visit, begin the in-world build-in.
        if self.town_transition > 0.0 {
            self.town_transition = (self.town_transition - dt).max(0.0);
            if self.town_transition == 0.0 {
                if let Some(&(tx0, ty0, ref nm)) = self.towns.first() {
                    self.player.x = tx0 as f32 + 0.5;
                    self.player.y = ty0 as f32 + 0.5;
                    self.spawn_point = (self.player.x, self.player.y);
                    self.interior = None;
                    if !self.town_visited {
                        self.town_build_t = 0.0;
                        self.town_visited = true;
                    }
                    play_sfx("door");
                    toast(&format!("Arrived at {}", nm));
                }
            }
        }
        // In-world town build-in animation (first arrival only).
        if self.town_build_t < 1.0 {
            self.town_build_t = (self.town_build_t + dt / TOWN_BUILD_TIME).min(1.0);
        }

        // Weather: periodically reconsider rain. Storms last ~20-40s; clear
        // spells ~25-45s. Cheap deterministic-ish roll from the clock.
        self.weather_timer -= dt;
        if self.weather_timer <= 0.0 {
            let r = (self.anim_clock * 7.0 + self.time_of_day * 311.0).fract();
            // Clear weather ends; otherwise roll a new condition: rain, snow,
            // a fierce storm, or a heat wave.
            if self.weather != 0 {
                self.weather = 0;
                self.weather_timer = 25.0 + r * 20.0;
            } else if r < 0.20 {
                self.weather = 1; // rain
                self.weather_timer = 20.0 + r * 20.0;
            } else if r < 0.30 {
                self.weather = 2; // snow
                self.weather_timer = 20.0 + r * 20.0;
            } else if r < 0.38 {
                self.weather = 3; // storm
                self.weather_timer = 16.0 + r * 12.0;
            } else if r < 0.46 {
                self.weather = 4; // heat wave
                self.weather_timer = 28.0 + r * 18.0;
            } else {
                self.weather_timer = 25.0 + r * 20.0;
            }
        }

        // Roaming elite (mini-boss): every few minutes a tougher-than-average
        // foe materializes in the wilds near the player and hunts them. A telegraphed
        // threat that scales with the world's danger.
        self.elite_timer -= dt;
        if self.elite_timer <= 0.0 && self.player.alive {
            self.elite_timer = 70.0 + (self.anim_clock * 13.0).fract() * 70.0;
            // Pick a tile 14-20 tiles away from the player on walkable ground.
            let ang = (self.anim_clock * 2.3 + self.player.x * 0.7).fract() * std::f32::consts::TAU;
            let dist = 14.0 + (self.anim_clock * 5.1).fract() * 6.0;
            let tx = (self.player.x + ang.cos() * dist).floor() as i32;
            let ty = (self.player.y + ang.sin() * dist).floor() as i32;
            if tile_at(&self.world, &mut self.chunks, tx, ty).walkable() {
                // Campaign: if a Crown Fragment guardian still roams, hunt it down.
                // Prefer the boss of the biome the player is currently in; failing
                // that, send the next unrecovered fragment's guardian so all five
                // are reachable no matter where the player wanders.
                let here = tile_at(&self.world, &mut self.chunks, self.player.x.floor() as i32, self.player.y.floor() as i32);
                let boss = boss_for_biome(here)
                    .filter(|k| k.fragment_bit().map_or(false, |b| self.fragments & (1 << b) == 0))
                    .or_else(|| next_fragment_boss(self.fragments));
                if let Some(kind) = boss {
                    self.enemies.spawn_elite(kind, tx as f32 + 0.5, ty as f32 + 0.5, 1.0);
                    toast(&format!("The {} emerges — a Crown Fragment is at stake!", kind.name()));
                    play_sfx("roar");
                } else {
                    let kind = if (self.anim_clock as i32) % 2 == 0 {
                        EnemyKind::Brute
                    } else {
                        EnemyKind::Ogre
                    };
                    let elite = 2.5 + (self.anim_clock * 3.0).fract() * 1.5;
                    self.enemies.spawn_elite(kind, tx as f32 + 0.5, ty as f32 + 0.5, elite);
                    toast(&format!(
                        "A roaming {} (elite x{:.1}) has appeared nearby!",
                        kind.name(),
                        elite
                    ));
                    play_sfx("levelup");
                }
            }
        }

        // Night raiders: after dark, bands of Raiders sweep in toward the player's
        // base to test the guards/Banners. They only appear once the player has
        // actually built something worth raiding.
        self.raider_timer -= dt;
        if self.raider_timer <= 0.0 && self.player.alive {
            self.raider_timer = 55.0 + (self.anim_clock * 7.0).fract() * 45.0;
            let night = daylight_at(self.time_of_day) < 0.3;
            if night && !self.structures.is_empty() {
                let idx = (self.anim_clock as usize) % self.structures.len();
                let s = self.structures[idx];
                for i in 0..2 {
                    let ang = (self.anim_clock * 1.7 + 3.1 * i as f32).fract() * std::f32::consts::TAU;
                    let tx = s.tx + (ang.cos() * 4.0) as i32;
                    let ty = s.ty + (ang.sin() * 4.0) as i32;
                    if tile_at(&self.world, &mut self.chunks, tx, ty).walkable() {
                        self.enemies
                            .spawn_elite(EnemyKind::Raider, tx as f32 + 0.5, ty as f32 + 0.5, 1.0);
                    }
                }
                toast("⚔ Raiders are attacking your base!");
                play_sfx("roar");
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
        // Shelter: standing in/next to a house (House/Cabin/Hut) is a safe zone —
        // it counts as warm and slowly mends wounds, so homes are worth defending.
        let sheltered = self.player.alive
            && self.structures.iter().any(|s| {
                matches!(
                    s.kind,
                    StructureKind::House | StructureKind::Cabin | StructureKind::Hut
                ) && {
                    let dx = s.tx as f32 + 0.5 - self.player.x;
                    let dy = s.ty as f32 + 0.5 - self.player.y;
                    dx * dx + dy * dy < 1.6 * 1.6
                }
            });
        let warm = warm || sheltered;
        // Tile under the player drives biome-specific survival + movement.
        let ptx = self.player.x.floor() as i32;
        let pty = self.player.y.floor() as i32;
        let biome = tile_at(&self.world, &mut self.chunks, ptx, pty);
        self.cur_biome = biome;
        // Village welcome: when the player first wanders into a hamlet, announce it.
        for (vx, vy, name) in &self.villages {
            if !self.visited_villages.contains(&(*vx, *vy)) {
                let dx = self.player.x - (*vx as f32 + 0.5);
                let dy = self.player.y - (*vy as f32 + 0.5);
                if dx * dx + dy * dy < 36.0 {
                    self.visited_villages.insert((*vx, *vy));
                    toast(&format!("Entered {} — a safe haven", name));
                }
            }
        }
        // Town welcome: a walled city with a railway — announced once on entry.
        for (vx, vy, name) in &self.towns {
            if !self.visited_towns.contains(&(*vx, *vy)) {
                let dx = self.player.x - (*vx as f32 + 0.5);
                let dy = self.player.y - (*vy as f32 + 0.5);
                if dx * dx + dy * dy < 256.0 {
                    self.visited_towns.insert((*vx, *vy));
                    toast(&format!("Entered {} — a walled old-world city", name));
                }
            }
        }
        if self.player.alive {
            let wet = self.weather == 1;
            self.player
                .tick(dt, temperature(self.time_of_day), warm, wet, biome, self.weather);
            // Resting by a fire (or inside a home) slowly mends wounds.
            if warm && self.player.hp < 100.0 {
                self.player.hp = (self.player.hp + dt * 3.0).min(100.0);
            }
            if sheltered && self.player.hp < 100.0 {
                self.player.hp = (self.player.hp + dt * 2.0).min(100.0);
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
                // Nocturnal enemies only emerge after dark.
                if kind.nocturnal() && daylight_at(self.time_of_day) > 0.5 {
                    continue;
                }
                self.enemies.get(tx, ty, kind, dt);
            }
        }
        let px = self.player.x;
        let py = self.player.y;
        let day = daylight_at(self.time_of_day);
        let mut contact: Option<(f32, f32, f32)> = None;
        // Townsfolk wander; village guards actively defend the hamlet by chasing
        // and striking any hostile enemy that comes near, then return to strolling.
        let enemy_spots: Vec<(f32, f32)> = self.enemies.enemies().map(|e| (e.x, e.y)).collect();
        let mut blocked = |tx: f32, ty: f32| -> bool {
            let (tx, ty) = (tx as i32, ty as i32);
            let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
            !tile.walkable()
                || self
                    .structures
                    .iter()
                    .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
                || resource_on(tx, ty, tile).is_some_and(|k| k.blocks_movement())
                    && !self.nodes.is_depleted(tx, ty)
        };
        for n in self.npcs.iter_mut() {
            if n.kind == NpcKind::Guard || n.kind == NpcKind::Golem {
                // Engage the nearest hostile enemy within the hamlet's perimeter,
                // but stay close to home so the watch doesn't wander off.
                let home_d = (n.home.0 - n.x).hypot(n.home.1 - n.y);
                let mut best: Option<(f32, (f32, f32))> = None;
                for &(ex, ey) in &enemy_spots {
                    let d = (ex - n.x).hypot(ey - n.y);
                    if d < 8.0 && best.map_or(true, |(bd, _)| d < bd) {
                        best = Some((d, (ex, ey)));
                    }
                }
                if let Some((d, (ex, ey))) = best.filter(|_| home_d < 11.0) {
                    let dx = ex - n.x;
                    let dy = ey - n.y;
                    let len = d.max(1e-3);
                    n.facing = (dx / len, dy / len);
                    let sp = 2.6 * dt; // guards move briskly when giving chase
                    if d > 1.2 {
                        let nx = n.x + dx / len * sp;
                        let ny = n.y + dy / len * sp;
                        if !blocked(nx, ny) {
                            n.x = nx;
                            n.y = ny;
                        }
                    }
                    n.walk = (n.walk + dt * 6.0) % 1000.0;
                } else {
                    n.update(dt, &mut blocked);
                }
            } else {
                n.update(dt, &mut blocked);
            }
        }
        let enemy_speed = game::enemy::Enemy::speed_scale_for_level(self.player.level);
        for e in self.enemies.enemies_mut() {
            self.discovered.insert(e.kind);
            // Keep enemy pace in step with the player's progression so they don't
            // become trivial to outrun as you level up.
            e.speed_mult = enemy_speed;
            if let Some(dmg) = e.update((px, py), dt, |tx, ty| {
                let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
                !tile.walkable()
                    || self
                        .structures
                        .iter()
                        .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
                    || resource_on(tx, ty, tile).is_some_and(|k| k.blocks_movement())
                        && !self.nodes.is_depleted(tx, ty)
            }) {
                contact = Some((e.x, e.y, dmg));
            }
            // Nocturnal undead burn in daylight and should not be prowling by day.
            e.daylight_burn(dt, day);
            // Village guards defend: any hostile enemy within a guard's reach takes
            // steady damage, so mobs that wander into a hamlet get driven off.
            let guarded = self.npcs.iter().any(|n| {
                (n.kind == NpcKind::Guard || n.kind == NpcKind::Golem)
                    && (n.x - e.x).hypot(n.y - e.y) < 7.0
            });
            if guarded {
                // A War Banner within 6 tiles of the guarding NPC empowers it,
                // reinforcing base defense (the Banner's reason for existing).
                let buffed = self.npcs.iter().any(|n| {
                    (n.kind == NpcKind::Guard || n.kind == NpcKind::Golem)
                        && (n.x - e.x).hypot(n.y - e.y) < 7.0
                        && self
                            .structures
                            .iter()
                            .any(|s| {
                                s.kind == StructureKind::Banner
                                    && ((s.tx as f32 + 0.5) - n.x).hypot((s.ty as f32 + 0.5) - n.y) < 6.0
                            })
                });
                e.take_damage(20.0 * dt * if buffed { 1.8 } else { 1.0 });
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
                .any(|s| s.tx == etx && s.ty == ety && (s.kind == StructureKind::Spike || s.kind == StructureKind::Trap))
            {
                e.take_damage(12.0 * dt);
            }
        }
        if let Some((ex, ey, dmg)) = contact {
            // Iron Plate (crafted at an Anvil) reduces incoming damage, and the
            // world bites harder at night.
            let night = 1.0 - daylight_at(self.time_of_day).clamp(0.25, 1.0);
            let dmg = dmg * (1.0 - self.craft_armor) * (1.0 + 0.6 * night);
            self.player.take_damage(dmg);
            if dmg > 0.5 {
                self.hitstop = 0.06;
                if self.player.blocking {
                    play_sfx("block");
                }
            }
            // Knock the player back away from the attacker.
            let dx = self.player.x - ex;
            let dy = self.player.y - ey;
            self.player.knockback(dx, dy, 0.4);
            play_sfx("hurt");
            self.hurt_flash = 1.0;
            if !self.player.alive {
                play_sfx("death");
            }
        }
        self.sweep_dead();
        self.collect_loot(dt);

        // Farm plots regrow their crops over time.
        const FARM_GROW: f32 = 30.0;
        for s in self.structures.iter() {
            if s.kind == StructureKind::FarmPlot {
                let cd = self.farm_cd.entry((s.tx, s.ty)).or_insert(0.0);
                *cd = (*cd - dt).max(0.0);
            }
        }

        // Turrets auto-fire at the nearest enemy in range; Healing Totems slowly
        // regenerate the player while they linger nearby.
        const TURRET_RANGE: f32 = 9.0;
        const TURRET_CD: f32 = 1.1;
        const HEAL_RADIUS: f32 = 4.0;
        const HEAL_RATE: f32 = 8.0;
        let px = self.player.x;
        let py = self.player.y;
        for s in &self.structures {
            if s.kind == StructureKind::Turret {
                let cx = s.tx as f32 + 0.5;
                let cy = s.ty as f32 + 0.5;
                let cd = self.turret_cd.entry((s.tx, s.ty)).or_insert(0.0);
                *cd = (*cd - dt).max(0.0);
                if *cd <= 0.0 {
                    let r2 = TURRET_RANGE * TURRET_RANGE;
                    let mut best: Option<(f32, f32, f32)> = None; // (dist2, ex, ey)
                    for e in self.enemies.enemies() {
                        let dx = e.x - cx;
                        let dy = e.y - cy;
                        let d2 = dx * dx + dy * dy;
                        if d2 <= r2 && best.map_or(true, |(bd, _, _)| d2 < bd) {
                            best = Some((d2, e.x, e.y));
                        }
                    }
                    if let Some((_, ex, ey)) = best {
                        let (dx, dy) = (ex - cx, ey - cy);
                        let len = (dx * dx + dy * dy).sqrt().max(1e-4);
                        self.arrows.push(Arrow::new(cx, cy, dx / len, dy / len));
                        *cd = TURRET_CD;
                    }
                }
            } else if s.kind == StructureKind::HealingTotem {
                let cx = s.tx as f32 + 0.5;
                let cy = s.ty as f32 + 0.5;
                let dx = px - cx;
                let dy = py - cy;
                if dx * dx + dy * dy <= HEAL_RADIUS * HEAL_RADIUS && self.player.alive {
                    self.player.hp = (self.player.hp + HEAL_RATE * dt).min(player::MAX_HP);
                }
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
            play_sfx("roar");
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

        // Recompute the HUD compass objective (throttled internally).
        self.update_objective(dt);

        // In co-op the room's server advances the campaign; the synced stage is
        // authoritative (see net_sync). Only drive it locally in single-player.
        if self.net.is_none() {
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
            self.fragments,
            self.ending.is_some(),
            self.colossus_killed >= 1,
        );
        }

        // arrows fly, hit, and expire (a hit removes the arrow)
        let mut hit_pos = Vec::new();
        self.arrows.retain_mut(|a| {
            if !a.step(dt) {
                return false;
            }
            if a.from_player {
                for (_key, e) in self.enemies.iter_mut_with_key() {
                    if arrow_hits(a, std::iter::once(&*e)).is_some() {
                        e.take_damage(a.damage);
                        hit_pos.push((e.x, e.y));
                        play_sfx("hit");
                        self.hitstop = 0.06;
                        return false;
                    }
                }
            } else {
                // enemy arrow: hits the player
                let dx = self.player.x - a.x;
                let dy = self.player.y - a.y;
                if dx * dx + dy * dy <= 0.8 * 0.8 {
                    let night = 1.0 - daylight_at(self.time_of_day).clamp(0.25, 1.0);
                    let dmg = a.damage * (1.0 - self.craft_armor) * (1.0 + 0.6 * night);
                    self.player.take_damage(dmg);
                    if dmg > 0.5 {
                        self.hitstop = 0.06;
                        if self.player.blocking {
                            play_sfx("block");
                        }
                    }
                    self.player.knockback(dx, dy, 0.25);
                    play_sfx("hurt");
                    self.hurt_flash = 1.0;
                    if !self.player.alive {
                        play_sfx("death");
                    }
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

        let dir = if let Some((ax, ay)) = self.analog {
            let len = (ax * ax + ay * ay).sqrt();
            if len < 1e-4 {
                (0.0, 0.0)
            } else {
                (ax / len, ay / len)
            }
        } else {
            player::input_dir(self.keys[0], self.keys[1], self.keys[2], self.keys[3])
        };
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
            TileKind::Volcanic => 0.82,
            _ => 1.0,
        };
        // During a dodge roll, move in the dodge direction at boosted speed.
        let (move_dir, move_dt2) = if self.player.dodge_timer > 0.0 {
            (self.player.dodge_dir, move_dt * player::DODGE_BOOST)
        } else {
            (dir, move_dt)
        };
        player::move_player(&mut self.player, move_dir, move_dt2, speed_mul, |tx, ty| {
            let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
            !tile.walkable()
                || self
                    .structures
                    .iter()
                    .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
                || resource_on(tx, ty, tile).is_some_and(|k| k.blocks_movement())
                    && !self.nodes.is_depleted(tx, ty)
        });
        let moved = ((self.player.x - bx).powi(2) + (self.player.y - by).powi(2)).sqrt();
        self.speed = if dt > 0.0 { moved / dt } else { 0.0 };

        // Footstep SFX: while actually moving, tick on a cadence scaled by speed
        // (faster = quicker steps). Suppressed during a dodge roll.
        self.step_timer -= dt;
        if self.speed > 0.6 && self.player.dodge_timer <= 0.0 && self.step_timer <= 0.0 {
            play_sfx("step");
            self.step_timer = (0.42 - (self.speed * 0.02).min(0.18)).max(0.18);
        }

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
        let focus = if let Some(int) = &self.interior {
            (int.bx + int.px, int.by + int.py)
        } else {
            let f = render::focus_target(&self.player, (self.viewport[0], self.viewport[1]));
            (f.0 + self.cam_lead.0, f.1 + self.cam_lead.1)
        };
        player::follow_camera(&mut self.camera, focus, dt);
        self.ensure_visible();
        // Multiplayer: send our input and overlay the authoritative server
        // world on top of this frame's local (predictive) simulation.
        self.net_sync();
        let sprites = self.sprites();
        // While inside a building we draw only the interior sprites (the room) and
        // suppress terrain by passing an empty tile list.
        let tiles: &[_] = if self.interior.is_some() {
            &self.visible_cache.4[..0]
        } else {
            &self.visible_cache.4
        };
        // Movement intensity drives the humanoid walk cycle (0 = standing, 1 = brisk walk).
        let player_walk = (self.speed / 8.0).clamp(0.0, 1.0);
        let mesh_player = if self.interior.is_some() {
            None
        } else {
            Some(&self.player)
        };
        self.quad_count = render::build_tile_mesh(
            &self.world,
            &mut self.chunks,
            self.camera,
            (self.viewport[0], self.viewport[1]),
            tiles,
            &sprites,
            mesh_player,
            &mut self.vertices,
            self.anim_clock,
            player_walk,
        );
        // The player quad is always emitted while `player` is Some (it is, in
        // the live loop), so scanning the whole vertex buffer every frame to
        // rediscover it is wasted work.
        self.player_in_mesh = mesh_player.is_some();
    }

    /// Synchronize with the multiplayer server: send this frame's input and
    /// overlay the authoritative snapshot onto our local fields. Single-player
    /// (no `net`) is a no-op. Remote players are rebuilt here and drawn as
    /// humanoid sprites in `sprites()`.
    fn net_sync(&mut self) {
        let client = match self.net.as_ref() {
            Some(c) => c,
            None => return,
        };

        let (mx, my) = match self.analog {
            Some((ax, ay)) => (ax, ay),
            None => player::input_dir(self.keys[0], self.keys[1], self.keys[2], self.keys[3]),
        };
        let input = PlayerInput {
            move_x: mx,
            move_y: my,
            dodge: self.net_dodge,
            attack: self.net_atk,
            harvest: self.net_harvest,
            eat: self.net_eat,
            shoot: self.net_shoot,
            build: self.net_build.take(),
            weapon: self.player.weapon.as_u8(),
            weapon_unlocked: self.player.unlocked,
            enchant: self.player.enchant,
            craft: None,
        };
        self.net_dodge = false;
        self.net_atk = false;
        self.net_harvest = false;
        self.net_eat = false;
        self.net_shoot = false;
        client.send_input(&input);

        // Play a one-shot "join" cue the first time the server welcomes us.
        let my_id = match client.id() {
            Some(id) => {
                if self.net_id.is_none() {
                    play_sfx("join");
                    glog("[net] connected to co-op server");
                }
                id
            }
            None => return,
        };
        self.net_id = Some(my_id);
        let snap = match client.sample() {
            Some(s) => s,
            None => return,
        };

        if let Some(lp) = snap.players.iter().find(|p| p.id == my_id) {
            self.player.x = lp.x;
            self.player.y = lp.y;
            self.player.hp = lp.hp;
            self.player.hunger = lp.hunger;
            self.player.stamina = lp.stamina;
            self.player.facing = lp.facing;
            self.player.alive = lp.alive;
        }

        // Adopt the room's authoritative campaign progress so every co-op player
        // sees the same quest objective and crafting milestone in the HUD.
        self.quest.stage = snap.quest_stage;
        self.crafted_iron = snap.iron_crafted;

        let mut enemies = Vec::with_capacity(snap.enemies.len());
        for es in &snap.enemies {
            let mut e = Enemy::new(es.x, es.y, es.kind);
            e.hp = es.hp;
            e.facing = es.facing;
            e.state = es.state;
            e.windup = es.windup;
            e.flash = es.flash;
            enemies.push(e);
        }
        self.enemies.render_sync(enemies);

        self.structures = snap
            .structures
            .iter()
            .map(|s| Structure { tx: s.tx, ty: s.ty, kind: s.kind })
            .collect();
        self.arrows = snap
            .arrows
            .iter()
            .map(|a| Arrow {
                x: a.x,
                y: a.y,
                dx: a.dx,
                dy: a.dy,
                life: 3.0,
                from_player: a.from_player,
                damage: ARROW_DAMAGE,
            })
            .collect();
        self.time_of_day = snap.time_of_day;

        self.remote_players = snap
            .players
            .iter()
            .filter(|p| p.id != my_id)
            .map(|p| {
                let mut pl = Player::new(p.x, p.y);
                pl.hp = p.hp;
                pl.hunger = p.hunger;
                pl.stamina = p.stamina;
                pl.facing = p.facing;
                pl.alive = p.alive;
                (p.id, pl)
            })
            .collect();
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
        if let Some(int) = &self.interior {
            return self.interior_sprites(int);
        }
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
            // Town build-in: while the in-world reveal is still ramping, hold back
            // (tx,ty) buildings whose staggered threshold hasn't been reached, so the
            // city appears to rise into place on first arrival.
            if self.town_build_t < 1.0 && self.is_town_tile(s.tx, s.ty) {
                let reveal = portal_reveal_at(s.tx, s.ty);
                if self.town_build_t < reveal {
                    continue;
                }
            }
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
            // Wind-up telegraph: as the strike nears, the figure flushes red so
            // the player gets a clear dodge cue before contact damage lands.
            if e.windup > 0.0 {
                let t = (e.windup / WINDUP).clamp(0.0, 1.0);
                sp.color = [
                    sp.color[0] + (1.0 - sp.color[0]) * 0.6 * t,
                    sp.color[1] * (1.0 - 0.45 * t),
                    sp.color[2] * (1.0 - 0.45 * t),
                ];
            }
            // Daylight scorch: nocturnal undead caught in the sun flush ember-orange.
            let b = e.burn.min(1.0);
            if b > 0.0 {
                sp.color = [
                    sp.color[0] * (1.0 - b) + 1.0 * b,
                    sp.color[1] * (1.0 - b) + 0.45 * b,
                    sp.color[2] * (1.0 - b) + 0.15 * b,
                ];
            }
            // Drive walk-cycle animation: idle enemies breathe; chasing/attacking
            // ones stride. Only the humanoid (Boss) element uses this today.
            sp.walk = match e.state {
                AiState::Idle => 0.15,
                _ => 0.9,
            };
            // Attack lunge: ramps up as the wind-up completes (the strike lands
            // when windup reaches 0), so melee foes visibly lunge on contact.
            sp.attack = if e.windup > 0.0 {
                (1.0 - (e.windup / WINDUP).clamp(0.0, 1.0))
            } else {
                0.0
            };
            // Flash is already baked into sp.color above; keep the generic path idle.
            sp.flash = 0.0;
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
                EnemyKind::ScorpionQueen => 42.0,
                EnemyKind::FrostGolem => 58.0,
                EnemyKind::ToadKing => 46.0,
                EnemyKind::OceanLeviathan => 44.0,
                EnemyKind::Brute => 30.0,
                EnemyKind::Stormcaller => 26.0,
                EnemyKind::Wolf => 16.0,
                EnemyKind::Archer => 24.0,
                EnemyKind::Raider => 24.0,
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
        // Remote multiplayer players: humanoid figures, each tinted a distinct
        // hue from their server id (and a small HP plate) so allies are easy to
        // tell apart at a glance.
        for (id, rp) in &self.remote_players {
            if !rp.alive {
                continue;
            }
            let hue = ((*id * 47) % 360) as f32;
            let col = hsv_to_rgb(hue, 0.6, 1.0);
            sprites.push(
                Sprite::new_center(rp.x, rp.y, col, 16.0, 22.0, 0.0)
                    .with_style(SpriteStyle::Humanoid)
                    .with_facing(rp.facing)
                    .with_walk(0.6),
            );
            sprites.push(
                Sprite::new_center(rp.x, rp.y, [0.0, 0.0, 0.0], 11.0, 2.5, 28.0)
                    .with_style(SpriteStyle::HpBack),
            );
            let hp01 = (rp.hp / 100.0).clamp(0.0, 1.0);
            sprites.push(
                Sprite::new_center(rp.x, rp.y, [1.0 - hp01, hp01, 0.1], 11.0 * hp01.max(0.05), 2.5, 28.0)
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
        // ground loot: small bobbing diamonds tinted by item kind
        for l in &self.loot {
            let bob = (l.phase.sin() * 2.0).max(0.0);
            let mut sp = Sprite::new_center(l.x, l.y, l.kind.color(), 7.0, 7.0, 6.0 + bob)
                .with_style(SpriteStyle::Generic);
            sp.alpha = 1.0;
            sprites.push(sp);
            // bright core
            let mut core = Sprite::new_center(l.x, l.y, [1.0, 1.0, 1.0], 2.5, 2.5, 7.0 + bob)
                .with_style(SpriteStyle::Generic);
            core.alpha = 0.8;
            sprites.push(core);
        }
        // weapon drops: bobbing diamonds tinted by weapon color
        for w in &self.weapon_loot {
            let bob = (w.phase.sin() * 2.0).max(0.0);
            let mut sp = Sprite::new_center(w.x, w.y, w.kind.color(), 9.0, 9.0, 7.0 + bob)
                .with_style(SpriteStyle::Generic);
            sp.alpha = 1.0;
            sprites.push(sp);
            let mut core = Sprite::new_center(w.x, w.y, [1.0, 1.0, 1.0], 3.0, 3.0, 8.0 + bob)
                .with_style(SpriteStyle::Generic);
            core.alpha = 0.85;
            sprites.push(core);
        }
        for p in &self.particles {
            let a = (p.life / p.max_life).clamp(0.0, 1.0);
            let mut ps = Sprite::new_center(p.x, p.y, p.color, p.size, p.size, 4.0)
                .with_style(SpriteStyle::Generic);
            ps.alpha = a;
            sprites.push(ps);
        }
        // Townsfolk: humanoid figures tinted by role, with a walk cycle driven by
        // their wander state. Guards wear helmet + sword + shield; golems are
        // bulky stone sentinels with a club — both clearly distinct from villagers.
        for n in &self.npcs {
            let (style, hw, hh) = match n.kind {
                NpcKind::Guard => (SpriteStyle::Guard, 9.0, 22.0),
                NpcKind::Golem => (SpriteStyle::Golem, 11.0, 28.0),
                _ => (SpriteStyle::Humanoid, 7.5, 17.0),
            };
            sprites.push(
                Sprite::new_center(n.x, n.y, n.kind.color(), hw, hh, 0.0)
                    .with_style(style)
                    .with_facing(n.facing)
                    .with_walk((n.walk * 0.15).min(1.0)),
            );
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
            let pflick = 0.9 + 0.1 * (self.anim_clock * 6.0).sin();
            data[0..4].copy_from_slice(&[sx, sy, 0.32 * pflick, 66.0]);
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
            let (mut intensity, radius, rgb) = match s.kind {
                StructureKind::Lantern => (0.40, 70.0, [1.0, 0.80, 0.40]),
                StructureKind::Brazier => (0.70, 120.0, [1.0, 0.50, 0.20]),
                _ => (0.55, 90.0, [1.0, 0.62, 0.28]),
            };
            // Warm, irregular candle flicker: two detuned sines per fixture so the
            // whole lit scene gently breathes instead of sitting at a flat brightness.
            let seed = s.tx as f32 * 0.7 + s.ty as f32 * 1.3;
            let flick = 0.82
                + 0.18
                    * ((self.anim_clock * 9.0 + seed).sin() * 0.6
                        + (self.anim_clock * 17.0 + seed * 2.1).sin() * 0.4)
                    .clamp(-1.0, 1.0);
            intensity *= flick;
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
        // Minimal screen-shake: offset the camera by a tiny, fast-flickering
        // amount while `shake` is active, then restore it before returning so the
        // next frame's follow-camera logic starts from a clean position.
        let (shake_x, shake_y) = if self.shake > 0.001 {
            (
                (self.anim_clock * 47.0).sin() * self.shake * 0.5,
                (self.anim_clock * 53.0 + 1.3).sin() * self.shake * 0.5,
            )
        } else {
            (0.0, 0.0)
        };
        self.camera.x += shake_x;
        self.camera.y += shake_y;
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
            // The WebGPU surface may not composite on this browser, but its
            // backing is still drawable — so we present (to recycle the
            // swapchain) and then copy it onto the 2D #blit canvas with a fast
            // GPU->GPU `drawImage`, avoiding the slow GPU->CPU readback entirely.
            let view = f.texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.record_pass(&view, &mut encoder);
            let cmd = encoder.finish();
            self.queue.submit([cmd]);
            self.queue.present(f);
            if FORCE_GPU.load(std::sync::atomic::Ordering::Relaxed) {
                // EXPERIMENT (?gpu): trust that the surface composites and show
                // it directly, skipping the per-frame readback that stalls fps.
                if self.backend_mode != 1 {
                    self.backend_mode = 1;
                    self.using_blit = false;
                    set_backend("gpu");
                }
            } else {
                blit_via_draw(
                    self.config.width,
                    self.config.height,
                    self.time_of_day,
                    self.anim_clock,
                    self.weather,
                    self.player.hp / 100.0,
                    self.hurt_flash,
                );
                if self.backend_mode != 2 {
                    self.backend_mode = 2;
                    self.using_blit = true;
                    set_backend("blit");
                }
            }
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
                let hp01 = self.player.hp / 100.0;
                let hurt01 = self.hurt_flash;
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
                                        hp01,
                                        hurt01,
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

        if self.frames % 120 == 0 {
            glog(&format!(
                "[gfx] heartbeat #{} quads={} backend={}",
                self.frames,
                self.quad_count,
                if self.using_blit { "blit" } else { "gpu" }
            ));
        }

        // Restore the camera so the shake offset doesn't leak into next frame.
        self.camera.x -= shake_x;
        self.camera.y -= shake_y;
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