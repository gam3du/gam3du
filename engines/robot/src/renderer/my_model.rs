use crate::{
    camera::Camera,
    model::{load_model, Mesh, Vertex},
    projection::Projection,
    renderer::{elapsed_as_vec, DepthTexture},
    RenderState,
};
use bytemuck::offset_of;
use core::f32;
use glam::{Mat4, Quat, Vec3, Vec4};
use std::{borrow::Cow, mem::size_of, path::PathBuf, time::Instant};
use wgpu::{self, util::DeviceExt};

pub(super) struct MyModelRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: u32,
    time_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    matrix_buf: wgpu::Buffer,
    robot_color_buf: wgpu::Buffer,
}

impl MyModelRenderer {
    #[expect(
        clippy::too_many_lines,
        reason = "TODO partition this function into smaller parts"
    )]
    #[must_use]
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_format: wgpu::TextureFormat,
    ) -> Self {
        let Mesh {
            vertex_buffer,
            index_buffer,
            index_count,
        } = Self::create_vertices(device);

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
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(8),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4 * 4),
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

        let texture_view = Self::create_texture_view(device, queue);

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

        let robot_color_bytes = [0_f32; 4];
        let robot_color_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Robot Color Uniform Buffer"),
            contents: bytemuck::cast_slice(&robot_color_bytes),
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
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: time_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: robot_color_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/robot.wgsl"
            ))),
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Vertex, pos) as u64,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: offset_of!(Vertex, tex_coord) as u64,
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
            vertex_buf: vertex_buffer,
            index_buf: index_buffer,
            index_count,
            time_buf,
            robot_color_buf,
            bind_group,
            matrix_buf,
            pipeline,
        }
    }

    pub(super) fn render<'pipeline>(
        &'pipeline mut self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'pipeline>,
        state: &RenderState,
        projection: &Projection,
    ) {
        let position = Mat4::from_scale_rotation_translation(
            Vec3::new(0.5, 0.5, 0.5),
            Quat::from_rotation_z(state.animation_angle + f32::consts::FRAC_PI_2),
            state.animation_position + Vec3::new(0.0, 0.0, 0.5),
        );

        self.update_time(state.start_time, queue);
        self.update_robot_color(state.robot_color, queue);
        self.update_matrix(projection, &state.camera, queue, position);

        render_pass.push_debug_group("Prepare data for draw.");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        render_pass.pop_debug_group();
        render_pass.insert_debug_marker("Draw!");
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    fn update_matrix(
        &self,
        projection: &Projection,
        camera: &Camera,
        queue: &wgpu::Queue,
        position: Mat4,
    ) {
        let matrix = projection.matrix() * camera.matrix() * position;
        let mx_ref: &[f32; 16] = matrix.as_ref();
        queue.write_buffer(&self.matrix_buf, 0, bytemuck::cast_slice(mx_ref));
    }

    fn update_time(&self, start_time: Instant, queue: &wgpu::Queue) {
        let bytes = elapsed_as_vec(start_time);
        queue.write_buffer(&self.time_buf, 0, bytemuck::cast_slice(&bytes));
    }

    fn update_robot_color(&self, color: Vec4, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.robot_color_buf,
            0,
            bytemuck::cast_slice(color.as_ref()),
        );
    }

    fn create_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        vertex_buffers: &[wgpu::VertexBufferLayout<'_>; 1],
        view_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let vertex = wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: vertex_buffers,
        };

        let fragment_state = wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(view_format.into())],
        };

        let primitive = wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
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

    fn create_texture_view(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
        // Create the texture
        let size = 256;
        let texels = Self::create_texels(size as usize);
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

    fn create_vertices(device: &wgpu::Device) -> Mesh {
        let mut model =
            load_model(&PathBuf::from("engines/robot/assets/monkey.gltf"), device).unwrap();

        model.pop().unwrap()
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
}
