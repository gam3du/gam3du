// use crate::{
//     camera::Camera,
//     model::{load_model, Mesh, Vertex},
//     projection::Projection,
//     renderer::{elapsed_as_vec, DepthTexture},
//     RenderState,
// };

use core::f32;
use glam::{Mat4, Vec4};
use lib_geometry::{Camera, Projection};
use lib_time::elapsed_as_vec;
use std::{borrow::Cow, mem::size_of, path::Path};
use web_time::Instant;
use wgpu::{self, util::DeviceExt};

use crate::model::{load_model, Mesh, Vertex};

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: u32,
    time_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    world_matrix_buf: wgpu::Buffer,
    camera_matrix_buf: wgpu::Buffer,
    projection_matrix_buf: wgpu::Buffer,
    color_buf: wgpu::Buffer,
}

impl Renderer {
    /// Creates a new [`Renderer`].
    ///
    /// # Panics
    ///
    /// Panics if the model cannot be loaded from the source file.
    #[expect(
        clippy::too_many_lines,
        reason = "TODO partition this function into smaller parts"
    )]
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_format: wgpu::TextureFormat,
        shader_source: Cow<'_, str>,
        depth_stencil_state: wgpu::DepthStencilState,
        model_path: &Path,
    ) -> Self {
        let Mesh {
            vertex_buffer,
            index_buffer,
            index_count,
        } = {
            let mut model = load_model(model_path, device).unwrap();

            model.pop().unwrap()
        };

        // Create pipeline layout
        let matrix_binding_type = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(64),
        };

        let bind_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: matrix_binding_type,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: matrix_binding_type,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: matrix_binding_type,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(8),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4 * 4),
                    },
                    count: None,
                },
            ],
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let texture_view = Self::create_texture_view(device, queue);

        let uniform = |label, contents| wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };

        let uniform_matrix = |label| uniform(label, &[0_u8; size_of::<[f32; 16]>()]);
        let uniform_time = |label| uniform(label, &[0_u8; size_of::<[u32; 2]>()]);
        let uniform_color = |label| uniform(label, &[0_u8; size_of::<[f32; 4]>()]);

        let world_matrix_buf =
            device.create_buffer_init(&uniform_matrix("world matrix uniform buffer"));
        let camera_matrix_buf =
            device.create_buffer_init(&uniform_matrix("camera matrix uniform buffer"));
        let projection_matrix_buf =
            device.create_buffer_init(&uniform_matrix("projection matrix uniform buffer"));
        let time_buf = device.create_buffer_init(&uniform_time("time uniform buffer"));
        let color_buf = device.create_buffer_init(&uniform_color("color uniform buffer"));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: world_matrix_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_matrix_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: projection_matrix_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: time_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: color_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader_source),
        });

        let pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            &[Vertex::buffer_layout()],
            view_format,
            depth_stencil_state,
        );

        Self {
            vertex_buf: vertex_buffer,
            index_buf: index_buffer,
            index_count,
            time_buf,
            color_buf,
            bind_group,
            world_matrix_buf,
            camera_matrix_buf,
            projection_matrix_buf,
            pipeline,
        }
    }

    #[expect(clippy::too_many_arguments, reason = "TODO")]
    pub fn render<'pipeline>(
        &'pipeline mut self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'pipeline>,
        world_matrix: Mat4,
        camera: &Camera,
        projection: &Projection,
        color: Vec4,
        time_reference: Instant,
    ) {
        self.update_time(time_reference, queue);
        self.update_color(color, queue);
        self.update_matrices(projection, camera, queue, world_matrix);

        render_pass.push_debug_group("Prepare data for draw.");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        render_pass.pop_debug_group();
        render_pass.insert_debug_marker("Draw!");
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    fn update_matrices(
        &self,
        projection: &Projection,
        camera: &Camera,
        queue: &wgpu::Queue,
        position: Mat4,
    ) {
        queue.write_buffer(
            &self.world_matrix_buf,
            0,
            bytemuck::cast_slice(position.as_ref()),
        );

        let camera_matrix = camera.matrix();
        queue.write_buffer(
            &self.camera_matrix_buf,
            0,
            bytemuck::cast_slice(camera_matrix.as_ref()),
        );

        let projection_matrix = projection.matrix();
        queue.write_buffer(
            &self.projection_matrix_buf,
            0,
            bytemuck::cast_slice(projection_matrix.as_ref()),
        );
    }

    fn update_time(&self, time_reference: Instant, queue: &wgpu::Queue) {
        let bytes = elapsed_as_vec(time_reference);
        queue.write_buffer(&self.time_buf, 0, bytemuck::cast_slice(&bytes));
    }

    fn update_color(&self, color: Vec4, queue: &wgpu::Queue) {
        queue.write_buffer(&self.color_buf, 0, bytemuck::cast_slice(color.as_ref()));
    }

    fn create_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        vertex_buffers: &[wgpu::VertexBufferLayout<'_>; 1],
        view_format: wgpu::TextureFormat,
        depth_stencil_state: wgpu::DepthStencilState,
    ) -> wgpu::RenderPipeline {
        let vertex = wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: vertex_buffers,
        };

        let fragment_state = wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
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
            depth_stencil: Some(depth_stencil_state),
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
