use std::{
    f32::consts::{PI, TAU},
    mem::size_of,
    ops::{AddAssign, SubAssign},
    time::Duration,
};

use bytemuck::{offset_of, Pod, Zeroable};
use glam::{FloatExt, IVec3, Mat4, Quat, Vec2, Vec3, Vec4};
use log::error;
use std::{borrow::Cow, time::Instant};
use wgpu::{self, util::DeviceExt};

use crate::api::Identifier;

use super::{
    camera::Camera,
    elapsed_as_vec,
    floor::{Floor, LineSegment},
    projection::Projection,
    DepthTexture,
};

pub(super) struct Robot {
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: u32,
    time_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    matrix_buf: wgpu::Buffer,
    animation_position: Vec3,
    animation_angle: f32,
    position: IVec3,
    orientation: Orientation,
    current_animation: Option<Animation>,
}

impl Robot {
    // TODO partition this function into smaller parts
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_format: wgpu::TextureFormat,
    ) -> Self {
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

        let wireframe_pipeline = device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
            .then(|| {
                Self::create_wireframe_pipeline(
                    device,
                    &pipeline_layout,
                    &shader,
                    &vertex_buffers,
                    view_format,
                )
            });

        let orientation = Orientation::default();
        let position = IVec3::new(0, 0, 0);

        Self {
            vertex_buf,
            index_buf,
            index_count: u32::try_from(index_data.len()).unwrap(),
            time_buf,
            bind_group,
            matrix_buf,
            pipeline,
            pipeline_wire: wireframe_pipeline,
            animation_position: position.as_vec3() + Vec3::new(0.5, 0.5, 0.0),
            current_animation: None,
            animation_angle: orientation.angle(),
            orientation,
            position,
        }
    }

    pub(super) fn render<'pipeline>(
        &'pipeline mut self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'pipeline>,
        camera: &Camera,
        projection: &Projection,
        start_time: Instant,
    ) {
        if let Some(animation) = self.current_animation.as_ref() {
            if animation.animate(&mut self.animation_position, &mut self.animation_angle) {
                self.current_animation.take();
            }
        };

        let position = Mat4::from_rotation_translation(
            Quat::from_rotation_z(self.animation_angle),
            self.animation_position,
        );

        self.update_time(start_time, queue);
        self.update_matrix(projection, camera, queue, position);

        render_pass.push_debug_group("Prepare data for draw.");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        render_pass.pop_debug_group();
        render_pass.insert_debug_marker("Draw!");
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);

        if let Some(ref pipe) = self.pipeline_wire {
            render_pass.set_pipeline(pipe);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
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
        })
    }

    fn create_wireframe_pipeline(
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
            entry_point: "fs_wire",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: view_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        operation: wgpu::BlendOperation::Add,
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    },
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        };

        let primitive = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Line,
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

    #[allow(clippy::similar_names)] // those are code names
    fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
        let front = 0.4;
        let back = -0.4;
        let left = 0.2;
        let right = -0.2;
        let top = 0.25;
        let bottom = 0.0;
        let scale_front = 0.5;
        let scale_top = 0.8;

        //  xyz
        let flt = Vec3::new(
            front * scale_top,
            left * scale_top * scale_front,
            top * scale_front,
        );
        let blt = Vec3::new(back * scale_top, left * scale_top, top);
        let frt = Vec3::new(
            front * scale_top,
            right * scale_top * scale_front,
            top * scale_front,
        );
        let brt = Vec3::new(back * scale_top, right * scale_top, top);
        let flb = Vec3::new(front, left * scale_front, bottom * scale_front);
        let blb = Vec3::new(back, left, bottom);
        let frb = Vec3::new(front, right * scale_front, bottom * scale_front);
        let brb = Vec3::new(back, right, bottom);

        let vertex_data = [
            // front
            vertex(frb, Vec2::new(1.0, -1.0)),
            vertex(flb, Vec2::new(1.0, 1.0)),
            vertex(flt, Vec2::new(1.0, 1.0)),
            vertex(frt, Vec2::new(1.0, -1.0)),
            // back
            vertex(brt, Vec2::new(-1.0, 1.0)),
            vertex(blt, Vec2::new(-1.0, -1.0)),
            vertex(blb, Vec2::new(-1.0, -1.0)),
            vertex(brb, Vec2::new(-1.0, 1.0)),
            // left
            vertex(flb, Vec2::new(1.0, -1.0)),
            vertex(blb, Vec2::new(-1.0, -1.0)),
            vertex(blt, Vec2::new(-1.0, 1.0)),
            vertex(flt, Vec2::new(1.0, 1.0)),
            // right
            vertex(frt, Vec2::new(1.0, -1.0)),
            vertex(brt, Vec2::new(-1.0, -1.0)),
            vertex(brb, Vec2::new(-1.0, 1.0)),
            vertex(frb, Vec2::new(1.0, 1.0)),
            // top
            vertex(brt, Vec2::new(-1.0, -1.0)),
            vertex(frt, Vec2::new(1.0, -1.0)),
            vertex(flt, Vec2::new(1.0, 1.0)),
            vertex(blt, Vec2::new(-1.0, 1.0)),
            // bottom
            vertex(blb, Vec2::new(1.0, -1.0)),
            vertex(flb, Vec2::new(-1.0, -1.0)),
            vertex(frb, Vec2::new(-1.0, 1.0)),
            vertex(brb, Vec2::new(1.0, 1.0)),
        ];

        let index_data: &[u16] = &[
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        (vertex_data.to_vec(), index_data.to_vec())
    }

    fn create_texels(size: usize) -> Vec<u8> {
        // testure doesn't need to be precise
        #[allow(clippy::cast_precision_loss)]
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

    pub(super) fn is_idle(&self) -> bool {
        self.current_animation.is_none()
    }

    pub(super) fn process_command(&mut self, command: &Command, floor: &mut Floor) {
        if let Some(current_animation) = self.current_animation.take() {
            current_animation.complete(&mut self.animation_position, &mut self.animation_angle);
        }

        self.current_animation = match command.name.0.as_str() {
            "MoveForward" => {
                let segment = LineSegment::from(self.orientation);

                // TODO make this a safe function
                #[allow(clippy::cast_sign_loss)]
                let start_index = (self.position.y * 10 + self.position.x + 55) as usize;
                floor.tiles[start_index].line_pattern |= segment;

                let offset = self.orientation.as_ivec3();
                if offset.x != 0 && offset.y != 0 {
                    // TODO make this a safe function
                    #[allow(clippy::cast_sign_loss)]
                    let index0 =
                        (self.position.y * 10 + (self.position.x + offset.x) + 55) as usize;
                    floor.tiles[index0].line_pattern |= segment.get_x_corner().unwrap();

                    // TODO make this a safe function
                    #[allow(clippy::cast_sign_loss)]
                    let index1 =
                        ((self.position.y + offset.y) * 10 + self.position.x + 55) as usize;
                    floor.tiles[index1].line_pattern |= -segment.get_x_corner().unwrap();
                }

                self.position += offset;

                // TODO make this a safe function
                #[allow(clippy::cast_sign_loss)]
                let end_index = (self.position.y * 10 + self.position.x + 55) as usize;
                floor.tiles[end_index].line_pattern |= -segment;
                floor.tainted = true;

                Some(Animation::Move {
                    start: self.animation_position,
                    end: self.position.as_vec3() + Vec3::new(0.5, 0.5, 0.0),
                    start_time: Instant::now(),
                    duration: Duration::from_millis(1_000),
                })
            }
            "TurnLeft" => {
                self.orientation += 1;
                Some(Animation::Rotate {
                    start: self.animation_angle,
                    end: self.orientation.angle(),
                    start_time: Instant::now(),
                    duration: Duration::from_millis(1_000),
                })
            }
            "TurnRight" => {
                self.orientation -= 1;
                Some(Animation::Rotate {
                    start: self.animation_angle,
                    end: self.orientation.angle(),
                    start_time: Instant::now(),
                    duration: Duration::from_millis(1_000),
                })
            }
            other => {
                error!("Unknown Command: {other}");
                None
            }
        };
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct Vertex {
    pos: Vec4,
    tex_coord: Vec2,
    _padding: Vec2,
}

fn vertex(position: Vec3, texture_coord: Vec2) -> Vertex {
    Vertex {
        pos: Vec4::new(position.x, position.y, position.z, 1.0),
        tex_coord: texture_coord,
        _padding: Vec2::default(),
    }
}

#[derive(Debug)]
pub struct Command {
    pub name: Identifier,
}

enum Animation {
    Move {
        start: Vec3,
        end: Vec3,
        start_time: Instant,
        duration: Duration,
    },
    Rotate {
        start: f32,
        end: f32,
        start_time: Instant,
        duration: Duration,
    },
}

impl Animation {
    fn progress(&self) -> f32 {
        match *self {
            Animation::Move {
                start_time,
                duration,
                ..
            }
            | Animation::Rotate {
                start_time,
                duration,
                ..
            } => start_time.elapsed().as_secs_f32() / duration.as_secs_f32(),
        }
    }

    fn animate(&self, position: &mut Vec3, orientation: &mut f32) -> bool {
        let progress = self.progress();
        let animation_complete = progress >= 1.0;
        self.animate_progress(progress, position, orientation);
        animation_complete
    }

    fn complete(&self, position: &mut Vec3, orientation: &mut f32) {
        self.animate_progress(1.0, position, orientation);
    }

    fn animate_progress(&self, progress: f32, position: &mut Vec3, orientation: &mut f32) {
        let progress = progress.clamp(0.0, 1.0);
        match *self {
            Animation::Move { start, end, .. } => {
                *position = start.lerp(end, progress);
            }
            Animation::Rotate { start, end, .. } => {
                *orientation = if (start - end).abs() <= PI {
                    start.lerp(end, progress)
                } else if start < end {
                    (start + TAU).lerp(end, progress)
                } else {
                    (start - TAU).lerp(end, progress)
                };
            }
        }
    }
}

// TODO W.I.P.
// their meaning is clear from the context
#[allow(clippy::min_ident_chars)]
#[derive(Clone, Copy, Default)]
#[repr(u8)]
pub(super) enum Orientation {
    /// positive x
    #[default]
    E = 0,
    /// +x, +y
    NE = 1,
    /// positive y
    N = 2,
    /// -x +y
    NW = 3,
    /// negative x
    W = 4,
    /// -x -y
    SW = 5,
    /// negative y
    S = 6,
    /// +x -y
    SE = 7,
}

impl Orientation {
    fn as_ivec3(self) -> IVec3 {
        match self {
            Orientation::E => IVec3::new(1, 0, 0),
            Orientation::NE => IVec3::new(1, 1, 0),
            Orientation::N => IVec3::new(0, 1, 0),
            Orientation::NW => IVec3::new(-1, 1, 0),
            Orientation::W => IVec3::new(-1, 0, 0),
            Orientation::SW => IVec3::new(-1, -1, 0),
            Orientation::S => IVec3::new(0, -1, 0),
            Orientation::SE => IVec3::new(1, -1, 0),
        }
    }

    fn angle(self) -> f32 {
        f32::from(self as u8) / 8.0 * TAU
    }
}

impl From<u8> for Orientation {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::E,
            1 => Self::NE,
            2 => Self::N,
            3 => Self::NW,
            4 => Self::W,
            5 => Self::SW,
            6 => Self::S,
            7 => Self::SE,
            _ => unreachable!(),
        }
    }
}

impl AddAssign<u8> for Orientation {
    fn add_assign(&mut self, rhs: u8) {
        *self = (*self as u8).wrapping_add(rhs).into();
    }
}

impl SubAssign<u8> for Orientation {
    fn sub_assign(&mut self, rhs: u8) {
        *self = (*self as u8).wrapping_sub(rhs).into();
    }
}
