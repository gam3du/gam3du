#![expect(dead_code, clippy::allow_attributes_without_reason, reason = "WIP")]
use std::{
    io::{BufReader, Cursor},
    iter::ExactSizeIterator,
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
pub(crate) struct Vertex {
    pub(crate) pos: Vec4,
    pub(crate) tex_coord: Vec2,
    _padding: Vec2,
    // position: Vec3,
    // normal: Vec3,
    // tex_coords: Vec2,
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
                    let _normal = normals
                        .as_mut()
                        .and_then(Iterator::next)
                        .unwrap_or_default();
                    let tex_coord = tex_coords
                        .as_mut()
                        .and_then(Iterator::next)
                        .unwrap_or_default();

                    Vertex {
                        pos: (Vec3::from(position), 1.0).into(),
                        tex_coord: tex_coord.into(),
                        _padding: Vec2::default(),
                        // normal: normal.into(),
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
