use std::mem::size_of;

use bytemuck::offset_of;
use std::{borrow::Cow, time::Instant};
use wgpu::{util::DeviceExt, PipelineCompilationOptions, Queue, RenderPass, TextureFormat};

use crate::{game_state::Tick, renderer::DepthTexture};

use crate::{camera::Camera, projection::Projection, renderer::elapsed_as_vec, tile::Tile};

use crate::RenderState;

pub(super) struct FloorRenderer {
    pipeline: wgpu::RenderPipeline,
    time_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    matrix_buf: wgpu::Buffer,
    tile_buf: wgpu::Buffer,
    tiles_tick: Tick,
}

impl FloorRenderer {
    #[expect(
        clippy::similar_names,
        reason = "`time` will be moved to global scope anyway"
    )]
    #[must_use]
    pub(super) fn new(
        device: &wgpu::Device,
        _queue: &Queue,
        view_format: TextureFormat,
        state: &RenderState,
    ) -> Self {
        let tile_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Buffer"),
            contents: bytemuck::cast_slice(&state.tiles),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = Self::create_bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let mx_ref = &[0_u8; size_of::<[f32; 16]>()];
        let matrix_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: mx_ref, //bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let elapsed_bytes = [0_u32; 2];
        let time_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Uniform Buffer"),
            contents: bytemuck::cast_slice(&elapsed_bytes),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/floor.wgsl"
            ))),
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: size_of::<Tile>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Tile, pos) as u64,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: offset_of!(Tile, line_pattern) as u64,
                    shader_location: 2,
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
            pipeline,
            time_buf,
            bind_group,
            matrix_buf,
            tile_buf,
            tiles_tick: state.tiles_tick,
        }
    }

    // fn tile_count(&self) -> u32 {
    //     u32::try_from(self.tiles.len()).unwrap()
    // }

    pub(super) fn render<'pipeline>(
        &'pipeline mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'pipeline>,
        state: &mut RenderState,
        projection: &Projection,
    ) {
        if state.tiles_tick > self.tiles_tick {
            queue.write_buffer(&self.tile_buf, 0, bytemuck::cast_slice(&state.tiles));
            self.tiles_tick = state.tiles_tick;
        }
        let tile_count = u32::try_from(state.tiles.len()).unwrap();

        self.update_time(state.start_time, queue);
        self.update_matrix(projection, &state.camera, queue);

        render_pass.push_debug_group("Prepare data for draw.");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.tile_buf.slice(..));
        render_pass.pop_debug_group();
        render_pass.insert_debug_marker("Draw!");
        render_pass.draw(0..4, 0..tile_count);
    }

    fn update_matrix(&self, projection: &Projection, camera: &Camera, queue: &Queue) {
        // TODO camera and projection should be shared across all shaders
        let matrix = projection.matrix() * camera.matrix();
        let mx_ref: &[f32; 16] = matrix.as_ref();
        queue.write_buffer(&self.matrix_buf, 0, bytemuck::cast_slice(mx_ref));
    }

    fn update_time(&self, start_time: Instant, queue: &Queue) {
        let bytes = elapsed_as_vec(start_time);
        queue.write_buffer(&self.time_buf, 0, bytemuck::cast_slice(&bytes));
    }

    fn create_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        vertex_buffers: &[wgpu::VertexBufferLayout<'_>],
        view_format: TextureFormat,
    ) -> wgpu::RenderPipeline {
        let vertex = wgpu::VertexState {
            module: shader,
            entry_point: "vs_floor",
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
            topology: wgpu::PrimitiveTopology::TriangleStrip,
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
            cache: None,
        })
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        })
    }
}
