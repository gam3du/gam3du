use crate::{game_state::Tick, renderer::DepthTexture, tile::Tile, RenderState};
use bytemuck::offset_of;
use glam::Vec4;
use lib_geometry::{Camera, Projection, SIZE_OF_MAT4, SIZE_OF_UVEC4, SIZE_OF_VEC4};
use lib_time::elapsed_as_vec;
use std::{borrow::Cow, mem::size_of};
use web_time::Instant;
use wgpu::util::DeviceExt;

pub(super) struct FloorRenderer {
    pipeline: wgpu::RenderPipeline,
    // vertex_buf: wgpu::Buffer,
    // index_buf: wgpu::Buffer,
    // index_count: u32,
    time_buf: wgpu::Buffer,
    camera_pos_buf: wgpu::Buffer,
    light_pos_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    matrix_buf: wgpu::Buffer,
    tile_buf: wgpu::Buffer,
    tiles_tick: Tick,
}

impl FloorRenderer {
    #[expect(
        clippy::similar_names,
        clippy::too_many_lines,
        reason = "`time` will be moved to global scope anyway; TODO split this up"
    )]
    #[must_use]
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_format: wgpu::TextureFormat,
        state: &RenderState,
    ) -> Self {
        // let (vertex_data, index_data) = Self::create_vertices();

        // let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(&vertex_data),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        // let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(&index_data),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

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
            label: Some("Matrix Uniform Buffer"),
            contents: mx_ref, //bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let elapsed_bytes = [0_u32; 4];
        let time_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Uniform Buffer"),
            contents: bytemuck::cast_slice(&elapsed_bytes),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let floor_size_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Floor size Uniform Buffer"),
            contents: bytemuck::bytes_of(&state.floor_size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_pos_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::bytes_of(&state.camera.position),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_pos_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Uniform Buffer"),
            contents: bytemuck::bytes_of(&state.camera.position),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_view = create_texture_view(device, queue);
        // device.create_sampler(&wgpu::SamplerDescriptor {
        //     label: todo!(),
        //     address_mode_u: todo!(),
        //     address_mode_v: todo!(),
        //     address_mode_w: todo!(),
        //     mag_filter: todo!(),
        //     min_filter: todo!(),
        //     mipmap_filter: todo!(),
        //     lod_min_clamp: todo!(),
        //     lod_max_clamp: todo!(),
        //     compare: todo!(),
        //     anisotropy_clamp: todo!(),
        //     border_color: todo!(),
        // });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: time_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: floor_size_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: camera_pos_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: light_pos_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../../../applications/robot/shaders/floor.wgsl"
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
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Tile, color) as u64,
                    shader_location: 3,
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
            camera_pos_buf,
            light_pos_buf,
        }
    }

    // fn tile_count(&self) -> u32 {
    //     u32::try_from(self.tiles.len()).unwrap()
    // }

    pub(super) fn render<'pipeline>(
        &'pipeline mut self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'pipeline>,
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
        self.update_camera(state.camera.position, queue);
        self.update_light(state.camera.position, queue);

        render_pass.push_debug_group("Prepare data for draw.");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.tile_buf.slice(..));
        render_pass.pop_debug_group();
        render_pass.insert_debug_marker("Draw!");
        render_pass.draw(0..48, 0..tile_count);
    }

    fn update_matrix(&self, projection: &Projection, camera: &Camera, queue: &wgpu::Queue) {
        // TODO camera and projection should be shared across all shaders
        let matrix = projection.matrix() * camera.matrix();
        let mx_ref: &[f32; 16] = matrix.as_ref();
        queue.write_buffer(&self.matrix_buf, 0, bytemuck::cast_slice(mx_ref));
    }

    fn update_time(&self, start_time: Instant, queue: &wgpu::Queue) {
        let bytes = elapsed_as_vec(start_time);
        queue.write_buffer(&self.time_buf, 0, bytemuck::cast_slice(&bytes));
    }

    fn update_camera(&self, camera_pos: Vec4, queue: &wgpu::Queue) {
        queue.write_buffer(&self.camera_pos_buf, 0, bytemuck::bytes_of(&camera_pos));
    }

    fn update_light(&self, light_pos: Vec4, queue: &wgpu::Queue) {
        queue.write_buffer(&self.light_pos_buf, 0, bytemuck::bytes_of(&light_pos));
    }

    fn create_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        vertex_buffers: &[wgpu::VertexBufferLayout<'_>],
        view_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let vertex = wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_floor"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: vertex_buffers,
        };

        let fragment_state = wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_floor_tile"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(view_format.into())],
        };

        let primitive = wgpu::PrimitiveState {
            cull_mode: None, //Some(wgpu::Face::Back),
            topology: wgpu::PrimitiveTopology::TriangleList,
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
                        min_binding_size: wgpu::BufferSize::new(SIZE_OF_MAT4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(SIZE_OF_UVEC4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(SIZE_OF_UVEC4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(SIZE_OF_VEC4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(SIZE_OF_VEC4),
                    },
                    count: None,
                },
            ],
        })
    }
}

fn create_texture_view(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    // Create the texture
    let size = 256;
    let texels = create_texels(size as usize);
    let texture_extent = wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    queue.write_texture(
        texture.as_image_copy(),
        &texels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(size),
            rows_per_image: None,
        },
        texture_extent,
    );
    texture_view
}

fn create_texels(size: usize) -> Vec<u8> {
    #[expect(
        clippy::cast_precision_loss,
        reason = "texture doesn't need to be precise"
    )]
    (0..size * size)
        .map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}
