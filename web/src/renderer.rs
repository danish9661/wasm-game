use game::building::{CHEST_RANGE, Structure, StructureKind, try_build};
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
use game::render::{self, Camera, Sprite, VERTEX_STRIDE_BYTES};
use game::resources::{NodeRegistry, ResourceKind, resource_on, HARVEST_RANGE};
use game::world::{ChunkCache, WorldGen, tile_at};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;

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
    }
}

fn struct_name(kind: StructureKind) -> &'static str {
    match kind {
        StructureKind::Campfire => "F",
        StructureKind::Wall => "W",
        StructureKind::Chest => "C",
        StructureKind::Altar => "A",
    }
}

fn enemy_name(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::Slime => "Slime",
        EnemyKind::Boss => "Warden",
    }
}

const SHADER: &str = r#"
struct Uniforms {
    viewport: vec2<f32>,
    daylight: f32,
    _pad: f32,
    lights: array<vec4<f32>, 16>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) color: vec3<f32>) -> VsOut {
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
    // global day/night: blend toward a dim blue night palette
    let night = vec3<f32>(0.16, 0.18, 0.32);
    col = mix(night, col, u.daylight);
    // campfire point lights: warm additive glow with soft falloff
    let sp = (in.pos.xy * 0.5 + 0.5) * u.viewport;
    for (var i = 0u; i < 8u; i++) {
        let lp = u.lights[i * 2u];
        if (lp.w <= 0.0) { continue; }
        let d = distance(sp, lp.xy);
        let fall = lp.z * exp(-d * d / (lp.w * lp.w));
        col += u.lights[i * 2u + 1u].rgb * fall;
    }
    return vec4<f32>(col, 1.0);
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
static RENDER_CAP: std::sync::Mutex<(u32, u32)> = std::sync::Mutex::new((960, 540));

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
fn blit_to_2d_canvas(data: &[u8], width: u32, height: u32, bytes_per_row: u32) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let doc = match window.document() {
        Some(d) => d,
        None => return,
    };
    let canvas = match doc
        .get_element_by_id("blit")
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
    {
        Some(c) => c,
        None => {
            glog("[gfx] blit: #blit canvas not found");
            return;
        }
    };
    if canvas.width() != width || canvas.height() != height {
        canvas.set_width(width);
        canvas.set_height(height);
    }
    let ctx = match canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
    {
        Some(c) => c,
        None => {
            glog("[gfx] blit: 2d context unavailable");
            return;
        }
    };
    let w = width as usize;
    let h = height as usize;
    let bpr = bytes_per_row as usize;
    let mut rgba: Vec<u8> = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        let src = y * bpr;
        let take = (w * 4).min(data.len().saturating_sub(src));
        rgba.extend_from_slice(&data[src..src + take]);
        if take < w * 4 {
            rgba.resize(rgba.len() + (w * 4 - take), 0);
        }
    }
    let clamped = Clamped(rgba.as_slice());
    match ImageData::new_with_u8_clamped_array_and_sh(clamped, width, height) {
        Ok(img) => {
            let _ = ctx.put_image_data(&img, 0.0, 0.0);
        }
        Err(e) => {
            glog(&format!("[gfx] blit: ImageData error {e:?}"));
        }
    }
}

struct VertexBuffer {
    buffer: wgpu::Buffer,
    capacity: u32,
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
    keys: [bool; 4],
    world: WorldGen,
    world_seed: u32,
    chunks: ChunkCache,
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
    respawn_timer: f32,
    debug_swing_hits: u32,
    debug_attacks: u32,
    debug_shots: u32,
    vertices: Vec<f32>,
    quad_count: u32,
    frames: u64,
    player_in_mesh: bool,
    readback_buffer: Option<wgpu::Buffer>,
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
            Some(_) => "unknown",
        };
        format!(
            "quads={} frames={} player=({:.1},{:.1}) hp={:.0} hunger={:.0} stamina={:.0} alive={} inv=(w{},s{},f{}) structures={} structs={} mobs={} mob={} pack={} swings={} atk={} shots={} quest=S{} ruins=({},{}) chest={} time={} near={} boss={} frag={} altar={} nearaltar={} ending={} ng={} bosshp={} altartile={}",
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
            self.inventory.count(ItemKind::Fragment),
            self.altar_placed as u8,
            self.near_altar as u8,
            ending_str,
            self.ng_plus,
            boss_hp,
            self.altar_tile
                .map(|(ax, ay)| format!("({ax},{ay})"))
                .unwrap_or_else(|| "none".to_string()),
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
            keys: [false; 4],
            world,
            world_seed: 1337,
            chunks,
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
            boss_spawned: false,
            altar_placed: false,
            altar_tile: None,
            near_altar: false,
            ending_pending: false,
            ending: None,
            ng_plus: 0,
            spawn_point: (px, py),
            time_of_day: START_TIME,
            respawn_timer: 0.0,
            debug_swing_hits: 0,
            debug_attacks: 0,
            debug_shots: 0,
            vertices: Vec::with_capacity(64 * 1024 * 6),
            quad_count: 0,
            frames: 0,
            player_in_mesh: false,
            readback_buffer: None,
        })
    }

    pub fn set_key(&mut self, code: &str, down: bool) {
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
                "KeyJ" => self.swing(),
                "KeyK" => self.shoot_arrow(),
                "KeyC" => {
                    self.player.eat(&mut self.inventory);
                }
                _ => {}
            }
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
        for e in &mut hits {
            e.take_damage(SWING_DAMAGE);
        }
        drop(hits);
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
            for it in items {
                self.inventory.add(it, 1);
            }
            match kind {
                EnemyKind::Slime => self.slimes_killed += 1,
                EnemyKind::Boss => self.boss_killed += 1,
            }
            // bosses never respawn; slimes return after 15s
            let respawn = if matches!(kind, EnemyKind::Boss) { f32::MAX } else { 15.0 };
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
        if let Some((tx, ty, kind)) = self.nearest_resource() {
            if let Some(item) = self.nodes.chop(tx, ty, kind) {
                self.inventory.add(item, 1);
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

    fn build(&mut self, kind: StructureKind) {
        let tx = self.player.x.floor() as i32;
        let ty = self.player.y.floor() as i32;
        if self.structures.iter().any(|s| s.tx == tx && s.ty == ty) {
            return;
        }
        if !tile_at(&self.world, &mut self.chunks, tx, ty).walkable() {
            return;
        }
        if resource_on(tx, ty, tile_at(&self.world, &mut self.chunks, tx, ty)).is_some()
            && !self.nodes.is_depleted(tx, ty)
        {
            return;
        }
        if let Ok(s) = try_build(kind, tx, ty, &mut self.inventory) {
            self.structures.push(s);
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
        let mut structures = Vec::new();
        structures.push(Structure { tx: self.ruins.0, ty: self.ruins.1, kind: StructureKind::Chest });
        for (wx, wy) in ruins_walls(self.ruins.0, self.ruins.1) {
            structures.push(Structure { tx: wx, ty: wy, kind: StructureKind::Wall });
        }
        self.structures = structures;
        self.quest = QuestLog::new();
        self.boss_killed = 0;
        self.boss_spawned = false;
        self.altar_placed = false;
        self.altar_tile = None;
        self.near_altar = false;
        self.ending_pending = false;
        self.time_of_day = START_TIME;
        self.respawn_timer = 0.0;
    }

    /// Reforge the Crown (campaign finale). `choice`: 0 = Reign (victory),
    /// 1 = Shatter (New Game+ with a harder, reseeded world). Both start a
    /// fresh run with `ng_plus` incremented so the HUD can show the streak.
    pub fn reforge(&mut self, choice: u8) {
        if self.ending.is_some() || self.inventory.count(ItemKind::Fragment) == 0 {
            return;
        }
        self.ending = Some(choice % 2);
        self.ng_plus += 1;
        let seed = 1338 + (self.ng_plus - 1);
        self.reset_world(seed);
    }

    /// Start a brand-new run at the base seed (Save/Load "New Game").
    pub fn new_game(&mut self) {
        self.ng_plus = 0;
        self.ending = None;
        self.reset_world(1337);
    }

    // ---- Save / Load ------------------------------------------------------

    pub fn to_save(&self) -> crate::save::SaveState {
        use game::items::ItemKind;
        let inv = [ItemKind::Wood, ItemKind::Stone, ItemKind::Food, ItemKind::Fragment]
            .iter()
            .map(|k| (*k, self.inventory.count(*k)))
            .collect();
        crate::save::SaveState {
            version: 1,
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
            enemies: self.enemies.enemies().map(|e| (e.kind, e.x, e.y, e.hp)).collect(),
            quest_stage: self.quest.stage,
            slimes_killed: self.slimes_killed,
            boss_killed: self.boss_killed,
            boss_spawned: self.boss_spawned,
            altar_placed: self.altar_placed,
            altar_tile: self.altar_tile,
            ending_pending: self.ending_pending,
            ending: self.ending,
            ng_plus: self.ng_plus,
            time_of_day: self.time_of_day,
            spawn_point: self.spawn_point,
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
        self.boss_spawned = s.boss_spawned;
        self.altar_placed = s.altar_placed;
        self.altar_tile = s.altar_tile;
        self.near_altar = false;
        self.ending_pending = s.ending_pending;
        self.ending = s.ending;
        self.ng_plus = s.ng_plus;
        self.time_of_day = s.time_of_day;
        self.spawn_point = s.spawn_point;
        self.respawn_timer = 0.0;
        self.arrows = Vec::new();
        self.nodes = NodeRegistry::new();
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
        let mut data = [0.0f32; 4 + LIGHT_FLOATS];
        data[0] = self.viewport[0];
        data[1] = self.viewport[1];
        data[2] = daylight_at(self.time_of_day);
        let lights = self.light_data();
        data[4..].copy_from_slice(&lights);
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck_cast(&data));
    }

    pub fn update(&mut self, dt: f32) {
        self.frames += 1;
        self.time_of_day = (self.time_of_day + dt / DAY_LENGTH).rem_euclid(1.0);

        // survival: hunger/stamina/temperature
        if self.player.alive {
            self.player.tick(dt, temperature(self.time_of_day));
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
        let vp = (self.viewport[0], self.viewport[1]);
        for (tx, ty) in render::visible_tiles(self.camera, vp) {
            let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
            if let Some(kind) = spawner_on(tx, ty, tile) {
                self.enemies.get(tx, ty, kind, dt);
            }
        }
        let px = self.player.x;
        let py = self.player.y;
        let mut contact: Option<f32> = None;
        for e in self.enemies.enemies_mut() {
            if let Some(dmg) = e.update((px, py), dt, |tx, ty| {
                !tile_at(&self.world, &mut self.chunks, tx, ty).walkable()
                    || self
                        .structures
                        .iter()
                        .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
            }) {
                contact = Some(dmg);
            }
        }
        if let Some(dmg) = contact {
            self.player.take_damage(dmg);
        }
        self.sweep_dead();

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
            self.slimes_killed,
            near_ruins,
            self.opened_chests.contains(&self.ruins),
            self.boss_killed >= 1,
            self.inventory.count(ItemKind::Fragment) > 0,
            self.ending.is_some(),
        );

        // arrows fly, hit, and expire (a hit removes the arrow)
        self.arrows.retain_mut(|a| {
            if !a.step(dt) {
                return false;
            }
            for (_key, e) in self.enemies.iter_mut_with_key() {
                if arrow_hits(a, std::iter::once(&*e)).is_some() {
                    e.take_damage(ARROW_DAMAGE);
                    return false;
                }
            }
            true
        });
        self.sweep_dead();

        let dir = player::input_dir(self.keys[0], self.keys[1], self.keys[2], self.keys[3]);
        player::move_player(&mut self.player, dir, dt, |tx, ty| {
            !tile_at(&self.world, &mut self.chunks, tx, ty).walkable()
                || self
                    .structures
                    .iter()
                    .any(|s| s.tx == tx && s.ty == ty && s.kind.blocks_movement())
        });
        let focus = render::focus_target(&self.player, (self.viewport[0], self.viewport[1]));
        player::follow_camera(&mut self.camera, focus, dt);
        let sprites = self.sprites();
        self.quad_count = render::build_tile_mesh(
            &self.world,
            &mut self.chunks,
            self.camera,
            (self.viewport[0], self.viewport[1]),
            &sprites,
            Some(&self.player),
            &mut self.vertices,
        );
        self.player_in_mesh = self
            .vertices
            .chunks_exact(render::VERTEX_FLOATS)
            .any(|v| v[2] == render::PLAYER_COLOR[0] && v[3] == render::PLAYER_COLOR[1] && v[4] == render::PLAYER_COLOR[2]);
    }

    /// Resource nodes + structures + enemies visible in the current view.
    fn sprites(&mut self) -> Vec<Sprite> {
        let mut sprites = Vec::new();
        for (tx, ty) in render::visible_tiles(self.camera, (self.viewport[0], self.viewport[1])) {
            let tile = tile_at(&self.world, &mut self.chunks, tx, ty);
            if let Some(kind) = resource_on(tx, ty, tile) {
                if !self.nodes.is_depleted(tx, ty) {
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
            sprites.push(e.kind.sprite(e.x, e.y, hp_frac));
            // hp bar: dark back + green->red fill, just above the diamond
            sprites.push(Sprite::new_center(e.x, e.y, [0.12, 0.12, 0.12], 10.0, 1.5, 18.0));
            sprites.push(Sprite::new_center(
                e.x,
                e.y,
                [1.0 - hp_frac, hp_frac, 0.1],
                10.0 * hp_frac.max(0.05),
                1.5,
                18.0,
            ));
        }
        for a in &self.arrows {
            sprites.push(Sprite::new_center(a.x, a.y, [0.95, 0.90, 0.85], 5.0, 2.0, 0.0));
        }
        sprites
    }

    /// Campfire point lights in screen pixels: [x, y, intensity, radius, r, g, b, 0] per slot.
    fn light_data(&self) -> [f32; LIGHT_FLOATS] {
        let mut data = [0.0f32; LIGHT_FLOATS];
        let mut n = 0;
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
            let slot = n * 8;
            data[slot..slot + 4].copy_from_slice(&[sx, sy, 0.55, 90.0]);
            data[slot + 4..slot + 8].copy_from_slice(&[1.0, 0.62, 0.28, 0.0]);
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
        // Display fallback: SwiftShader-Vulkan can't composite the WebGPU canvas
        // to screen in headed Chrome, so read the frame back and blit to #blit.
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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        // Always render the scene to the offscreen target (the WebGPU canvas
        // can't be composited in headed SwiftShader; #blit shows the read-back
        // frame).
        let off_view = self
            .offscreen
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.record_pass(&off_view, &mut encoder);

        // Read back + blit, but never start a new readback while one is still
        // pending — otherwise we'd allocate a buffer every frame and SwiftShader
        // would never catch up, leaking GPU memory until the tab crashes.
        let can_readback = !READBACK_INFLIGHT.load(std::sync::atomic::Ordering::Relaxed)
            && self.readback_buffer.is_some();
        if can_readback {
            let buffer = self.readback_buffer.clone().unwrap();
            let width = self.config.width;
            let height = self.config.height;
            let bytes_per_row = ((width * 4 + 255) / 256) * 256;
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.offscreen,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
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
            let gen = READBACK_GEN.load(std::sync::atomic::Ordering::Relaxed);
            READBACK_INFLIGHT.store(true, std::sync::atomic::Ordering::Relaxed);
            *READBACK.lock().unwrap() = String::from("queued");
            let buf = buffer.clone();
            let cmd = encoder.finish();
            cmd.map_buffer_on_submit(
                &buffer,
                wgpu::MapMode::Read,
                ..,
                move |res| {
                    match res {
                        Ok(()) => {
                            if let Ok(data) = buf.slice(..).get_mapped_range() {
                                blit_to_2d_canvas(&data, width, height, bytes_per_row);
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
                    // Only clear INFLIGHT if this callback belongs to the current
                    // buffer generation (a resize may have swapped the buffer).
                    if READBACK_GEN.load(std::sync::atomic::Ordering::Relaxed) == gen {
                        READBACK_INFLIGHT.store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                },
            );
            self.queue.submit([cmd]);
        } else {
            self.queue.submit([encoder.finish()]);
        }
        if self.frames % 600 == 0 {
            glog(&format!("[gfx] heartbeat #{} quads={}", self.frames, self.quad_count));
        }
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
    const UNIFORM_BYTES: u64 = (4 + LIGHT_FLOATS) as u64 * 4;
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
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
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
                blend: Some(wgpu::BlendState::REPLACE),
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
                min_binding_size: wgpu::BufferSize::new((4 + LIGHT_FLOATS) as u64 * 4),
            },
            count: None,
        }],
    })
}