#![expect(dead_code, clippy::allow_attributes_without_reason, reason = "WIP")]
use std::{
    io::{BufReader, Cursor},
    iter::ExactSizeIterator,
    mem::offset_of,
    path::Path,
};

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};
use gltf::{buffer, mesh::util::ReadTexCoords, Gltf};
use log::debug;
use wgpu::util::DeviceExt;

pub(crate) struct Mesh {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct Vertex {
    // Geometric properties
    pub position: Vec4,
    // ---- 16 byte alignment
    pub normal: Vec4,
    // Material properties
    // ---- 16 byte alignment
    pub base_color_factor: Vec4,
    // // ---- 16 byte alignment
    pub base_color_texture_coordinates: Vec2,
    pub _padding: Vec2,
    // pub(crate) metallic_factor: f32,
    // pub(crate) roughness_factor: f32,
    // // ---- 16 byte alignment
    // pub(crate) metallic_roughness_texture_coordinates: Vec2,
    // ---- 16 byte alignment
    // position: Vec3,
    // normal: Vec3,
    // tex_coords: Vec2,

    // "materials": [
    //     {
    //         "pbrMetallicRoughness": {
    //             "baseColorTexture": {
    //                 "index": 1,
    //                  "texCoord": 1
    //             },
    //             "baseColorFactor":
    //                 [ 1.0, 0.75, 0.35, 1.0 ],
    //             "metallicRoughnessTexture": {
    //                 "index": 5,
    //                 "texCoord": 1
    //             },
    //             "metallicFactor": 1.0,
    //             "roughnessFactor": 0.0,
    //         }
    //         "normalTexture": {
    //             "scale": 0.8,
    //             "index": 2,
    //             "texCoord": 1
    //         },
    //         "occlusionTexture": {
    //             "strength": 0.9,
    //             "index": 4,
    //             "texCoord": 1
    //         },
    //         "emissiveTexture": {
    //             "index": 3,
    //             "texCoord": 1
    //         },
    //         "emissiveFactor":
    //             [0.4, 0.8, 0.6]
    //     }
    // ],
}

impl Vertex {
    pub(crate) fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Vertex, position) as wgpu::BufferAddress,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Vertex, normal) as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Vertex, base_color_factor) as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: offset_of!(Vertex, base_color_texture_coordinates)
                        as wgpu::BufferAddress,
                    shader_location: 3,
                },
            ],
        }
    }
}

// #[repr(C)]
// #[derive(Clone, Copy, Pod, Zeroable, Default)]
// struct Vertex {
//     pos: Vec4,
//     tex_coord: Vec2,
//     _padding: Vec2,
// }

fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = Path::new("assets").join(file_name);
    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = Path::new("assets").join(file_name);
    let data = std::fs::read(path)?;

    Ok(data)
}

pub(crate) fn load_model(model_path: &Path, device: &wgpu::Device) -> anyhow::Result<Vec<Mesh>> {
    let gltf_text = std::fs::read_to_string(model_path)?;
    let gltf_cursor = Cursor::new(gltf_text);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = Gltf::from_reader(gltf_reader)?;

    // Load buffers
    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            buffer::Source::Bin => {
                // if let Some(blob) = gltf.blob.as_deref() {
                //     buffer_data.push(blob.into());
                //     println!("Found a bin, saving");
                // };
            }
            buffer::Source::Uri(uri) => {
                let path = model_path.with_file_name(uri);
                let bin = std::fs::read(path)?;
                buffer_data.push(bin);
            }
        }
    }

    let mut meshes = Vec::new();

    debug!("{gltf:#?}");

    for mesh in gltf.meshes() {
        // for scene in gltf.scenes() {
        //     for node in scene.nodes() {
        //         let mesh = node.mesh().expect("Got mesh");
        let primitives = mesh.primitives();
        primitives.for_each(|primitive| {
            let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

            let mut positions = reader.read_positions();
            let mut normals = reader.read_normals();
            let mut tex_coords = reader.read_tex_coords(0).map(ReadTexCoords::into_f32);

            let vertex_count = [
                positions.as_ref().map(ExactSizeIterator::len),
                normals.as_ref().map(ExactSizeIterator::len),
                tex_coords.as_ref().map(ExactSizeIterator::len),
            ]
            .into_iter()
            .flatten()
            .max()
            .unwrap_or_default();

            let vertices = (0..vertex_count)
                .map(|_| {
                    let position = positions
                        .as_mut()
                        .and_then(Iterator::next)
                        .unwrap_or_default();
                    let normal = normals
                        .as_mut()
                        .and_then(Iterator::next)
                        .unwrap_or_default();
                    let tex_coord = tex_coords
                        .as_mut()
                        .and_then(Iterator::next)
                        .unwrap_or_default();

                    Vertex {
                        position: (Vec3::from(position), 1.0).into(),
                        normal: (Vec3::from(normal), 1.0).into(),
                        base_color_factor: Vec4::new(1.0, 0.0, 0.0, 1.0),
                        base_color_texture_coordinates: tex_coord.into(),
                        _padding: Vec2::default(),
                    }
                })
                .collect::<Vec<_>>();

            let mut indices = Vec::new();
            if let Some(indices_raw) = reader.read_indices() {
                indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{model_path:?} Vertex Buffer")),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{model_path:?} Index Buffer")),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            meshes.push(Mesh {
                vertex_buffer,
                index_buffer,
                index_count: indices.len().try_into().unwrap(),
            });
        });
        // }
    }

    Ok(meshes)
}
