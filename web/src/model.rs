//! GLB -> GPU-ready mesh data for the additive 3D render path.
//!
//! Art contract (docs/ART_DIRECTION.md): Y-up, base at y = 0, one flat material
//! per colour, < 500 triangles. We bake each glTF node transform into the vertex
//! positions/normals and copy the material base colour into a per-vertex colour
//! attribute, so every model ends up as one interleaved vertex buffer + one u32
//! index buffer that the simple flat/lit pipeline in `pipeline3d` can draw with
//! instancing. No PBR, no textures, no heavy math dependency.

/// Tiny column-major 4x4 matrix helpers. Shared with `pipeline3d` so the 3D
/// path stays self-contained and does not need `glam`.
pub(crate) mod math4 {
    pub type Mat4 = [[f32; 4]; 4];

    pub fn identity() -> Mat4 {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// `a * b` (both column-major: `m[column][row]`).
    pub fn multiply(a: &Mat4, b: &Mat4) -> Mat4 {
        let mut out = [[0.0f32; 4]; 4];
        for c in 0..4 {
            for r in 0..4 {
                out[c][r] = a[0][r] * b[c][0]
                    + a[1][r] * b[c][1]
                    + a[2][r] * b[c][2]
                    + a[3][r] * b[c][3];
            }
        }
        out
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Mat4 {
        let mut m = identity();
        m[3][0] = x;
        m[3][1] = y;
        m[3][2] = z;
        m
    }

    pub fn scaling(x: f32, y: f32, z: f32) -> Mat4 {
        let mut m = identity();
        m[0][0] = x;
        m[1][1] = y;
        m[2][2] = z;
        m
    }

    /// Right-handed rotation around +Y (matches glTF convention).
    pub fn rotation_y(rad: f32) -> Mat4 {
        let (c, s) = (rad.cos(), rad.sin());
        [
            [c, 0.0, -s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [s, 0.0, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// Transform a point (applies translation + w divide).
    pub fn transform_point(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
        let x = m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2] + m[3][0];
        let y = m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2] + m[3][1];
        let z = m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2] + m[3][2];
        let w = m[0][3] * p[0] + m[1][3] * p[1] + m[2][3] * p[2] + m[3][3];
        if w.abs() > 1e-8 && (w - 1.0).abs() > 1e-8 {
            [x / w, y / w, z / w]
        } else {
            [x, y, z]
        }
    }

    /// Transform a direction/normal with the upper 3x3 and renormalise.
    /// (The assets use rigid transforms and uniform scale, so an inverse
    /// transpose is unnecessary here.)
    pub fn transform_direction(m: &Mat4, v: [f32; 3]) -> [f32; 3] {
        let x = m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2];
        let y = m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2];
        let z = m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2];
        let len = (x * x + y * y + z * z).sqrt();
        if len > 1e-8 {
            [x / len, y / len, z / len]
        } else {
            [0.0, 1.0, 0.0]
        }
    }

    /// Orthographic projection with WebGPU's 0..1 depth range.
    /// `n`/`f` are positive distances in front of the camera; view-space z is
    /// negative there.
    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, n: f32, f: f32) -> Mat4 {
        let mut m = identity();
        m[0][0] = 2.0 / (right - left);
        m[1][1] = 2.0 / (top - bottom);
        m[2][2] = 1.0 / (n - f);
        m[3][0] = -(right + left) / (right - left);
        m[3][1] = -(top + bottom) / (top - bottom);
        m[3][2] = n / (n - f);
        m
    }

    /// Right-handed look-at: camera looks down view-space -Z.
    pub fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Mat4 {
        let sub = |a: [f32; 3], b: [f32; 3]| -> [f32; 3] {
            [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
        };
        let cross = |a: [f32; 3], b: [f32; 3]| -> [f32; 3] {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let dot = |a: [f32; 3], b: [f32; 3]| -> f32 {
            a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
        };
        let norm = |v: [f32; 3]| -> [f32; 3] {
            let l = (dot(v, v)).sqrt();
            if l > 1e-8 {
                [v[0] / l, v[1] / l, v[2] / l]
            } else {
                [0.0, 1.0, 0.0]
            }
        };
        let z = norm(sub(eye, target)); // "backward"
        let x = norm(cross(up, z)); // "right"
        let y = cross(z, x); // "up"
        [
            [x[0], y[0], z[0], 0.0],
            [x[1], y[1], z[1], 0.0],
            [x[2], y[2], z[2], 0.0],
            [-dot(x, eye), -dot(y, eye), -dot(z, eye), 1.0],
        ]
    }
}

/// Floats per vertex: position xyz, normal xyz, colour rgb.
pub const MODEL_VERTEX_FLOATS: usize = 9;
pub const MODEL_VERTEX_STRIDE: u64 = (MODEL_VERTEX_FLOATS * 4) as u64;

/// One GPU-ready GLB model. Geometries from every mesh primitive and every
/// node are merged into a single vertex/index pair; node transforms and
/// material colours are already baked into the vertices.
pub struct Model {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
    /// Material base colours seen while parsing (diagnostics / future use).
    pub base_colours: Vec<[f32; 3]>,
}

/// Reinterpret an f32 slice as raw bytes for `queue.write_buffer`.
/// (Avoids pulling `bytemuck` in just for a cast.)
fn f32_bytes(data: &[f32]) -> &[u8] {
    // SAFETY: f32 is 4 bytes with no padding, and every bit pattern is valid u8.
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
}

fn u32_bytes(data: &[u32]) -> &[u8] {
    // SAFETY: u32 is 4 bytes with no padding.
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
}

/// Upload bytes to a freshly-created vertex/index buffer.
fn upload_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    bytes: &[u8],
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        // wgpu requires a non-zero size that is a multiple of 4.
        size: bytes.len().max(4) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    if !bytes.is_empty() {
        queue.write_buffer(&buffer, 0, bytes);
    }
    buffer
}

/// Recursively walk a glTF node tree, appending transformed geometry.
fn walk_node(
    node: gltf::Node<'_>,
    parent: &math4::Mat4,
    buffers: &[gltf::buffer::Data],
    vertices: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    base_colours: &mut Vec<[f32; 3]>,
) {
    // glTF matrices are column-major, matching our Mat4 layout directly.
    let world = math4::multiply(parent, &node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            // We only draw triangles; points/lines are not part of the art spec.
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
            let positions: Vec<[f32; 3]> = match reader.read_positions() {
                Some(iter) => iter.collect(),
                None => continue,
            };
            if positions.is_empty() {
                continue;
            }
            let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(|it| it.collect());
            let primitive_indices: Vec<u32> = match reader.read_indices() {
                Some(iter) => iter.into_u32().collect(),
                None => (0..positions.len() as u32).collect(),
            };

            // Flat-shaded low-poly: use the material base colour as the vertex
            // colour, falling back to white if a primitive has no material.
            let factor = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();
            let colour = [factor[0], factor[1], factor[2]];
            base_colours.push(colour);

            let base = (vertices.len() / MODEL_VERTEX_FLOATS) as u32;
            for (i, p) in positions.iter().enumerate() {
                let wp = math4::transform_point(&world, *p);
                let n = match normals.as_ref() {
                    Some(ns) => math4::transform_direction(&world, ns[i]),
                    None => [0.0, 1.0, 0.0],
                };
                vertices.extend_from_slice(&[
                    wp[0], wp[1], wp[2], // position
                    n[0], n[1], n[2], // normal
                    colour[0], colour[1], colour[2], // vertex colour
                ]);
            }
            for index in primitive_indices {
                indices.push(base + index);
            }
        }
    }

    for child in node.children() {
        walk_node(child, &world, buffers, vertices, indices, base_colours);
    }
}

/// Parsed geometry, before GPU upload. Kept separate so the glTF conversion
/// can be tested on the host without a WebGPU device.
pub struct ParsedGeometry {
    pub name: String,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub base_colours: Vec<[f32; 3]>,
}

/// Parse a GLB into interleaved vertices + u32 indices, baking node transforms
/// and material base colours. No GPU resources involved.
pub fn parse_glb(bytes: &[u8]) -> Result<ParsedGeometry, String> {
    let (document, buffers, _images) =
        gltf::import_slice(bytes).map_err(|e| format!("gltf import failed: {e}"))?;

    let mut vertices: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut base_colours: Vec<[f32; 3]> = Vec::new();

    // Prefer the default scene; fall back to walking all root nodes when a
    // document has no scene set (still produces geometry for such assets).
    if let Some(scene) = document.scenes().next() {
        for root in scene.nodes() {
            walk_node(
                root,
                &math4::identity(),
                &buffers,
                &mut vertices,
                &mut indices,
                &mut base_colours,
            );
        }
    } else {
        for root in document.nodes() {
            walk_node(
                root,
                &math4::identity(),
                &buffers,
                &mut vertices,
                &mut indices,
                &mut base_colours,
            );
        }
    }

    if indices.is_empty() {
        return Err("GLB contained no triangle geometry".to_string());
    }

    let name = document
        .scenes()
        .next()
        .and_then(|s| s.name().map(str::to_owned))
        .unwrap_or_else(|| "model".to_string());
    Ok(ParsedGeometry {
        name,
        vertices,
        indices,
        base_colours,
    })
}

/// Upload already-parsed `[pos(3), normal(3), colour(3)]` geometry.
///
/// Shared by the GLB loader and the generated terrain heightfield so both go
/// through exactly one vertex/index layout for the 3D pipeline.
pub fn upload_geometry(
    parsed: ParsedGeometry,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Model {
    let ParsedGeometry {
        name,
        vertices,
        indices,
        base_colours,
    } = parsed;
    let vertex_buffer = upload_buffer(
        device,
        queue,
        "glb_vertices",
        f32_bytes(&vertices),
    );
    // Index buffers need INDEX usage, not VERTEX — build it separately.
    let index_bytes = u32_bytes(&indices);
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("glb_indices"),
        size: index_bytes.len().max(4) as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    if !index_bytes.is_empty() {
        queue.write_buffer(&index_buffer, 0, index_bytes);
    }

    Model {
        name,
        vertex_buffer,
        index_buffer,
        vertex_count: (vertices.len() / MODEL_VERTEX_FLOATS) as u32,
        index_count: indices.len() as u32,
        base_colours,
    }
}

/// Parse a GLB and upload its geometry to the GPU.
///
/// Uses the `gltf` crate (with its default `import`/`utils` features) for
/// parsing, then bakes node transforms + material colours into interleaved
/// `[pos(3), normal(3), colour(3)]` vertices and u32 indices.
pub fn load_glb(
    bytes: &[u8],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<Model, String> {
    Ok(upload_geometry(parse_glb(bytes)?, device, queue))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn asset(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("assets")
            .join("models")
            .join(format!("{name}.glb"))
    }

    #[test]
    fn parses_real_demo_assets() {
        for name in ["player", "tree", "house", "campfire", "rock", "crystal", "banner"] {
            let bytes = std::fs::read(asset(name)).expect("demo GLB missing");
            let parsed = parse_glb(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(parsed.vertices.len() % MODEL_VERTEX_FLOATS, 0, "{name}");
            assert!(!parsed.indices.is_empty(), "{name} has no indices");
            assert!(!parsed.base_colours.is_empty(), "{name} has no material");
            let vertex_count = parsed.vertices.len() / MODEL_VERTEX_FLOATS;
            for i in &parsed.indices {
                assert!((*i as usize) < vertex_count, "{name}: index out of range");
            }
            for v in &parsed.vertices {
                assert!(v.is_finite(), "{name}: non-finite vertex component");
            }
            eprintln!(
                "{name}: {} verts, {} indices, {} colours",
                vertex_count,
                parsed.indices.len(),
                parsed.base_colours.len()
            );
        }
    }
}
