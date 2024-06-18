use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, time::Instant};
use wgpu::{util::DeviceExt, PipelineCompilationOptions, Queue, RenderPass, TextureFormat};

use super::{camera::Camera, elapsed_as_vec, projection::Projection, DepthTexture};

pub(super) struct Floor {
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    time_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    matrix_buf: wgpu::Buffer,
}

impl Floor {
    // TODO partition this function into smaller parts
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub(super) fn new(device: &wgpu::Device, _queue: &Queue, view_format: TextureFormat) -> Self {
        let (vertex_data, index_data) = Self::create_vertices();

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(8),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create other resources
        // let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);
        //let mx_ref: &[f32; 16] = matrix.as_ref();
        let mx_ref = &[0_u8; size_of::<[f32; 16]>()];
        let matrix_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: mx_ref, //bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let elapsed_bytes = [0; 64];
        let time_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Uniform Buffer"),
            contents: bytemuck::cast_slice(&elapsed_bytes),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: time_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shader.wgsl"))),
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            &vertex_buffers,
            view_format,
        );

        Self {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            time_buf,
            bind_group,
            matrix_buf,
            pipeline,
        }
    }

    pub(super) fn render<'pipeline>(
        &'pipeline self,
        queue: &Queue,
        render_pass: &mut RenderPass<'pipeline>,
        camera: &Camera,
        projection: &Projection,
        start_time: Instant,
    ) {
        self.update_time(start_time, queue);
        self.update_matrix(projection, camera, queue);

        render_pass.push_debug_group("Prepare data for draw.");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        render_pass.pop_debug_group();
        render_pass.insert_debug_marker("Draw!");
        render_pass.draw_indexed(0..u32::try_from(self.index_count).unwrap(), 0, 0..1);
    }

    fn update_matrix(&self, projection: &Projection, camera: &Camera, queue: &Queue) {
        let matrix = projection.matrix() * camera.matrix();
        let mx_ref: &[f32; 16] = matrix.as_ref();
        queue.write_buffer(&self.matrix_buf, 0, bytemuck::cast_slice(mx_ref));
    }

    fn update_time(&self, start_time: Instant, queue: &Queue) {
        let mut elapsed_bytes = [0; 64];
        elapsed_bytes
            .iter_mut()
            .zip(elapsed_as_vec(start_time))
            .for_each(|(target, source)| *target = source);
        queue.write_buffer(&self.time_buf, 0, bytemuck::cast_slice(&elapsed_bytes));
    }

    fn create_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        vertex_buffers: &[wgpu::VertexBufferLayout; 1],
        view_format: TextureFormat,
    ) -> wgpu::RenderPipeline {
        let vertex = wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            compilation_options: PipelineCompilationOptions::default(),
            buffers: vertex_buffers,
        };

        let fragment_state = wgpu::FragmentState {
            module: shader,
            entry_point: "fs_floor_tile",
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(view_format.into())],
        };

        let primitive = wgpu::PrimitiveState {
            cull_mode: None, //Some(wgpu::Face::Back),
            ..Default::default()
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout),
            vertex,
            fragment: Some(fragment_state),
            primitive,
            depth_stencil: Some(DepthTexture::depth_stencil_state()),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
        let mut vertex_data = Vec::new();
        let mut index_data = Vec::new();
        for y in -5_i16..5 {
            let bottom = f32::from(y) + 0.1;
            let top = f32::from(y + 1) - 0.1;
            for x in -5_i16..5 {
                let left = f32::from(x) + 0.1;
                let right = f32::from(x + 1) - 0.1;
                let index = u16::try_from(vertex_data.len()).unwrap();
                vertex_data.push(vertex([left, bottom, 0.0], [0, 0]));
                vertex_data.push(vertex([right, bottom, 0.0], [1, 0]));
                vertex_data.push(vertex([left, top, 0.0], [0, 1]));
                vertex_data.push(vertex([right, top, 0.0], [1, 1]));

                // 2         3
                // +---------+
                // | \       |
                // |   \     |
                // |     \   |
                // |       \ |
                // +---------+
                // 0         1

                // for symmetry
                #[allow(clippy::identity_op)]
                index_data.extend([index + 0, index + 1, index + 2]);
                index_data.extend([index + 3, index + 2, index + 1]);
            }
        }

        (vertex_data, index_data)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

fn vertex(pos: [f32; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0], pos[1], pos[2], 1.0],
        _tex_coord: [f32::from(tc[0]), f32::from(tc[1])],
    }
}