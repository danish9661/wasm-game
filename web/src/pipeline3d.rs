//! Additive orthographic-isometric glTF render path.
//!
//! This is intentionally *separate* from the working 2.5D quad renderer in
//! `renderer.rs`: it owns its own canvas (`#demo3d`), device, surface, depth
//! buffer and pipeline, so none of the existing gameplay exports or the
//! `step()` game loop are touched. It draws GLB models from `model.rs` with
//! instanced draws, flat/lit shading and simple distance fog — no PBR stack.

use crate::model::math4::{self, Mat4};
use crate::model::{Model, MODEL_VERTEX_STRIDE};
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

/// Floats per instance: 4 mat4 columns + tint rgba.
const INSTANCE_FLOATS: usize = 20;
const INSTANCE_STRIDE: u64 = (INSTANCE_FLOATS * 4) as u64;

/// Uniform block: view_proj(64) + 7 vec4s = 176 bytes, std140 friendly.
const GLOBALS_FLOATS: usize = 16 + 7 * 4;

/// One instance = model transform (column-major mat4) + tint multiplier.
#[derive(Clone, Copy)]
pub struct Instance {
    pub transform: Mat4,
    pub tint: [f32; 4],
}

impl Instance {
    fn floats(&self) -> [f32; INSTANCE_FLOATS] {
        let mut out = [0.0f32; INSTANCE_FLOATS];
        for (c, col) in self.transform.iter().enumerate() {
            out[c * 4..c * 4 + 4].copy_from_slice(col);
        }
        out[16..20].copy_from_slice(&self.tint);
        out
    }
}

/// A group of instances drawn from one model with a single `draw_indexed`.
struct Group {
    /// Index into `Demo3D::models`.
    model: usize,
    /// Byte offset into the shared instance buffer.
    instance_offset: u64,
    count: u32,
}

thread_local! {
    static DEMO: RefCell<Option<Demo3D>> = const { RefCell::new(None) };
}

const SHADER_3D: &str = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,        // xyz = camera position
    sun_dir: vec4<f32>,    // xyz = direction TO sun, w = intensity
    sun_color: vec4<f32>,
    ambient: vec4<f32>,    // cool hemisphere fill
    point_pos: vec4<f32>,  // xyz = position, w = radius
    point_color: vec4<f32>,// rgb, w = intensity
    fog: vec4<f32>,        // rgb = fog colour, w = density
};
@group(0) @binding(0) var<uniform> g: Globals;

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) dist: f32,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) c0: vec4<f32>,
    @location(4) c1: vec4<f32>,
    @location(5) c2: vec4<f32>,
    @location(6) c3: vec4<f32>,
    @location(7) tint: vec4<f32>,
) -> VOut {
    let model = mat4x4<f32>(c0, c1, c2, c3);
    let wp = model * vec4<f32>(pos, 1.0);
    var out: VOut;
    out.clip = g.view_proj * wp;
    out.world = wp.xyz;
    // Uniform scale only in this demo, so the upper 3x3 is enough for normals.
    out.normal = normalize((model * vec4<f32>(normal, 0.0)).xyz);
    out.color = color * tint.rgb;
    out.dist = length(wp.xyz - g.eye.xyz);
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    // Flat-shaded look: quantise the sun term into a few discrete bands.
    let raw = max(dot(n, normalize(g.sun_dir.xyz)), 0.0);
   let lit = floor(raw * 4.0 + 0.5) / 4.0;
    var col = in.color * (g.ambient.rgb + g.sun_color.rgb * g.sun_dir.w * lit);
    // Single warm point light with a linear falloff.
    let to_light = g.point_pos.xyz - in.world;
    let att = max(0.0, 1.0 - length(to_light) / max(g.point_pos.w, 0.001));
    col += in.color * g.point_color.rgb * (g.point_color.w * att * att);
    // Simple distance fog toward a dark diorama backdrop.
    let f = clamp((in.dist - 20.0) * g.fog.w, 0.0, 1.0);
    col = mix(col, g.fog.rgb, f);
    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#;

fn f32_bytes(data: &[f32]) -> &[u8] {
    // SAFETY: f32 is 4 bytes with no padding; every bit pattern is valid u8.
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
}

struct Demo3D {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    globals_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    depth: wgpu::Texture,
    models: Vec<Model>,
    instance_buffer: wgpu::Buffer,
    groups: Vec<Group>,
    total_instances: u32,
    frame: u64,
    width: u32,
    height: u32,
}

impl Demo3D {
    /// Build a dedicated WebGPU context on a `#demo3d` canvas.
    async fn new() -> Result<Self, String> {
        let canvas = ensure_canvas()?;
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| format!("create_surface(demo3d): {e}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| format!("request_adapter(demo3d): {e}"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("demo3d_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .map_err(|e| format!("request_device(demo3d): {e}"))?;

        let format = surface
            .get_capabilities(&adapter)
            .formats
            .first()
            .copied()
            .ok_or("demo3d: no surface formats")?;
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

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("demo3d_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_3D.into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("demo3d_globals_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new((GLOBALS_FLOATS * 4) as u64),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("demo3d_pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("demo3d_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
                        array_stride: MODEL_VERTEX_STRIDE,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3, // position
                            1 => Float32x3, // normal
                            2 => Float32x3, // colour
                        ],
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: INSTANCE_STRIDE,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            3 => Float32x4, // instance columns 0..3
                            4 => Float32x4,
                            5 => Float32x4,
                            6 => Float32x4,
                            7 => Float32x4, // tint
                        ],
                    }),
                ],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                // Low-poly assets may not all agree on winding; keep both faces.
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("demo3d_globals"),
            size: (GLOBALS_FLOATS * 4) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("demo3d_globals_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });
        let depth = create_depth(&device, width, height);
        // Placeholder instance buffer; replaced by `set_scene`.
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("demo3d_instances"),
            size: INSTANCE_STRIDE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            device,
            queue,
            surface,
            config,
            pipeline,
            globals_buffer,
            bind_group,
            depth,
            models: Vec::new(),
            instance_buffer,
            groups: Vec::new(),
            total_instances: 0,
            frame: 0,
            width,
            height,
        })
    }

    /// Replace the demo scene. `placements` is parallel to `models`: each
    /// entry holds the instances of that model. All instances are packed into
    /// one buffer; one `draw_indexed` per model group keeps it instanced.
    fn set_scene(&mut self, models: Vec<Model>, placements: Vec<Vec<Instance>>) {
        let mut packed: Vec<f32> = Vec::new();
        let mut groups = Vec::new();
        let mut total = 0u32;
        for (model_index, instances) in placements.into_iter().enumerate() {
            if instances.is_empty() || model_index >= models.len() {
                continue;
            }
            let offset = (packed.len() * 4) as u64;
            for inst in &instances {
                packed.extend_from_slice(&inst.floats());
            }
            groups.push(Group {
                model: model_index,
                instance_offset: offset,
                count: instances.len() as u32,
            });
            total += instances.len() as u32;
        }
        let bytes = f32_bytes(&packed);
        self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("demo3d_instances"),
            size: bytes.len().max(INSTANCE_STRIDE as usize) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if !bytes.is_empty() {
            self.queue.write_buffer(&self.instance_buffer, 0, bytes);
        }
        self.models = models;
        self.groups = groups;
        self.total_instances = total;
    }

    fn write_globals(&mut self) {
        // Orthographic isometric camera: yaw 45°, pitch 30° (2:1-ish look).
        let yaw = 45.0_f32.to_radians();
        let pitch = 30.0_f32.to_radians();
        let dist = 24.0f32;
        let target = [0.0f32, 0.8, 0.0];
        let eye = [
            target[0] + pitch.cos() * yaw.sin() * dist,
            target[1] + pitch.sin() * dist,
            target[2] + pitch.cos() * yaw.cos() * dist,
        ];

        let half_h = 5.0f32;
        let aspect = self.width as f32 / self.height as f32;
        let proj = math4::ortho(
            -half_h * aspect,
            half_h * aspect,
            -half_h,
            half_h,
            0.1,
            100.0,
        );
        let view = math4::look_at(eye, target, [0.0, 1.0, 0.0]);
        let view_proj = math4::multiply(&proj, &view);

        let mut data = [0.0f32; GLOBALS_FLOATS];
        for (c, col) in view_proj.iter().enumerate() {
            data[c * 4..c * 4 + 4].copy_from_slice(col);
        }
        data[16..20].copy_from_slice(&[eye[0], eye[1], eye[2], 0.0]);
        // Warm sun, high in the sky.
        data[20..24].copy_from_slice(&[0.45, 0.82, 0.35, 0.78]);
        data[24..28].copy_from_slice(&[1.0, 0.93, 0.82, 0.0]);
        // Cool hemisphere fill.
        data[28..32].copy_from_slice(&[0.42, 0.47, 0.56, 0.0]);
        // Campfire-ish point light near the middle of the diorama.
        data[32..36].copy_from_slice(&[1.6, 1.2, 1.4, 6.0]);
        data[36..40].copy_from_slice(&[1.0, 0.55, 0.25, 0.55]);
        // Fog colour + density.
        data[40..44].copy_from_slice(&[0.05, 0.07, 0.11, 0.012]);
        self.queue
            .write_buffer(&self.globals_buffer, 0, f32_bytes(&data));
    }

    fn render_frame(&mut self) -> Result<(), String> {
        self.write_globals();
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) => f,
            wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            _ => return Ok(()), // Timeout / Occluded / Outdated / Lost: skip frame
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = self.depth.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("demo3d_encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("demo3d_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if !self.groups.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                for group in &self.groups {
                    let model = &self.models[group.model];
                    pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                    pass.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.set_vertex_buffer(
                        1,
                        self.instance_buffer.slice(
                            group.instance_offset
                                ..group.instance_offset + group.count as u64 * INSTANCE_STRIDE,
                        ),
                    );
                    pass.draw_indexed(0..model.index_count, 0, 0..group.count);
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        self.frame += 1;
        Ok(())
    }

    fn stats(&self) -> String {
        let vertices: u32 = self.models.iter().map(|m| m.vertex_count).sum();
        let colours: usize = self.models.iter().map(|m| m.base_colours.len()).sum();
        format!(
            "demo3d: {} models ({} verts, {} materials), {} instances, {} draw groups, {} frames, {}x{}",
            self.models.len(),
            vertices,
            colours,
            self.total_instances,
            self.groups.len(),
            self.frame,
            self.config.width,
            self.config.height
        )
    }
}

fn create_depth(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("demo3d_depth"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

/// Create (or reuse) a visible canvas dedicated to the 3D demo. Using a
/// separate canvas means the existing #game WebGPU surface is never touched.
fn ensure_canvas() -> Result<HtmlCanvasElement, String> {
    let window = web_sys::window().ok_or("no window")?;
    let doc = window.document().ok_or("no document")?;
    let canvas = match doc.get_element_by_id("demo3d") {
        Some(el) => el
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| "#demo3d exists but is not a <canvas>".to_string())?,
        None => {
            let el = doc
                .create_element("canvas")
                .map_err(|e| format!("create_element(canvas): {e:?}"))?;
            let canvas: HtmlCanvasElement = el
                .dyn_into()
                .map_err(|_| "failed to create demo canvas".to_string())?;
            canvas.set_id("demo3d");
            canvas.set_attribute(
                "style",
                "position:fixed;left:10px;bottom:10px;width:320px;height:240px;\
                 z-index:9999;border:2px solid #d9a62e;background:#0c111b",
            )
            .map_err(|e| format!("set style: {e:?}"))?;
            doc.body()
                .ok_or("no <body>")?
                .append_child(&canvas)
                .map_err(|e| format!("append demo canvas: {e:?}"))?;
            canvas
        }
    };
    // Must size BEFORE requesting the WebGPU context (see renderer.rs).
    canvas.set_width(640);
    canvas.set_height(480);
    Ok(canvas)
}

/// Download a GLB from the dev server into raw bytes.
async fn fetch_glb(url: &str) -> Result<Vec<u8>, String> {
    let window = web_sys::window().ok_or("no window")?;
    let response = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("fetch {url}: {e:?}"))?;
    let response: web_sys::Response = response
        .dyn_into()
        .map_err(|_| format!("fetch {url}: not a Response"))?;
    if !response.ok() {
        return Err(format!("fetch {url}: HTTP status {}", response.status()));
    }
    let array_buffer = wasm_bindgen_futures::JsFuture::from(
        response
            .array_buffer()
            .map_err(|e| format!("array_buffer {url}: {e:?}"))?,
    )
    .await
    .map_err(|e| format!("array_buffer {url}: {e:?}"))?;
    Ok(js_sys::Uint8Array::new(&array_buffer).to_vec())
}

/// Model names used by the demo scene (all exist in `assets/models/`).
const DEMO_MODELS: &[&str] = &[
    "player", "tree", "house", "campfire", "rock", "crystal", "banner",
];

fn make_instance(x: f32, z: f32, yaw: f32, scale: f32, tint: [f32; 4]) -> Instance {
    // translation * rotationY * uniform scale
    let mut transform = math4::multiply(
        &math4::translation(x, 0.0, z),
        &math4::rotation_y(yaw),
    );
    transform = math4::multiply(&transform, &math4::scaling(scale, scale, scale));
    Instance { transform, tint }
}

/// Build a small diorama layout. The repeated trees / rocks / crystals are
/// deliberately instanced to exercise the pipeline's instance step mode.
fn demo_placements(count: usize) -> Vec<Vec<Instance>> {
    let white = [1.0, 1.0, 1.0, 1.0];
    let mut placements = vec![Vec::new(); count];
    let mut push = |idx: usize, inst: Instance| {
        if idx < placements.len() {
            placements[idx].push(inst);
        }
    };
    if count > 0 {
        push(0, make_instance(0.0, 0.0, 0.6, 1.0, white)); // player
    }
    if count > 1 {
        for (x, z, yaw, s) in [
            (-3.2, -1.4, 0.2, 1.0),
            (-4.4, 1.9, 1.1, 0.85),
            (-2.0, 3.0, 2.4, 1.1),
            (3.8, -2.6, 0.7, 0.95),
        ] {
            push(1, make_instance(x, z, yaw, s, white)); // trees
        }
    }
    if count > 2 {
        push(2, make_instance(2.8, -0.6, -0.5, 1.0, white)); // house
    }
    if count > 3 {
        push(3, make_instance(0.9, 2.0, 0.0, 1.0, white)); // campfire
    }
    if count > 4 {
        for (x, z, s) in [(-1.2, 2.6, 0.7), (3.2, 2.4, 1.0), (4.4, 0.4, 0.8)] {
            push(4, make_instance(x, z, 0.3, s, white)); // rocks
        }
    }
    if count > 5 {
        for (x, z) in [(-0.7, -2.2), (1.9, -2.9)] {
            push(
                5,
                make_instance(x, z, 0.0, 0.9, [0.85, 0.95, 1.2, 1.0]),
            ); // crystals, faintly cool tint
        }
    }
    if count > 6 {
        push(6, make_instance(2.8, -0.6, 0.0, 1.0, [1.15, 1.05, 0.8, 1.0])); // banner
    }
    placements
}

/// Demo scene entry points (re-exported as wasm-bindgen functions in lib.rs)
/// -------------------------------------------------------------------------

/// Fetch several GLBs, upload them, and build the instanced demo scene.
/// Resolves to a status string; the caller can then call `render_3d_demo()`
/// once per frame. Safe to call repeatedly (it replaces the old scene).
pub async fn load_demo_models() -> Result<String, String> {
    let mut demo = Demo3D::new().await?;
    let mut models = Vec::new();
    let mut loaded_names = Vec::new();
    for name in DEMO_MODELS {
        let url = format!("/assets/models/{name}.glb");
        let bytes = fetch_glb(&url).await?;
        match crate::model::load_glb(&bytes, &demo.device, &demo.queue) {
            Ok(model) => {
                loaded_names.push(model.name.clone());
                models.push(model);
            }
            Err(e) => return Err(format!("load {url}: {e}")),
        }
    }
    let placements = demo_placements(models.len());
    demo.set_scene(models, placements);
    let stats = demo.stats();
    DEMO.with(|cell| *cell.borrow_mut() = Some(demo));
    Ok(format!("{stats}; loaded: {}", loaded_names.join(", ")))
}

/// Draw one frame of the loaded 3D demo on `#demo3d`.
/// No-op (returns a status string) if `load_demo_models()` has not run yet.
pub fn render_3d_demo() -> String {
    DEMO.with(|cell| {
        let mut slot = cell.borrow_mut();
        match slot.as_mut() {
            Some(demo) => match demo.render_frame() {
                Ok(()) => demo.stats(),
                Err(e) => format!("demo3d render error: {e}"),
            },
            None => String::from(
                "demo3d: not loaded — call `await load_demo_models()` first",
            ),
        }
    })
}
