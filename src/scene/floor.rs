use std::{mem::size_of, ops};

use bytemuck::{offset_of, Pod, Zeroable};
use rand::{thread_rng, Rng};
use std::{borrow::Cow, time::Instant};
use wgpu::{util::DeviceExt, PipelineCompilationOptions, Queue, RenderPass, TextureFormat};

use super::{camera::Camera, elapsed_as_vec, projection::Projection, DepthTexture};

pub(super) struct Floor {
    pipeline: wgpu::RenderPipeline,
    time_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    matrix_buf: wgpu::Buffer,
    tiles: Vec<Tile>,
    tile_buf: wgpu::Buffer,
}

impl Floor {
    // `time` will be moved to global scope anyway
    #[allow(clippy::similar_names)]
    #[must_use]
    pub(super) fn new(device: &wgpu::Device, _queue: &Queue, view_format: TextureFormat) -> Self {
        let tiles = Self::create_vertices();

        let tile_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Buffer"),
            contents: bytemuck::cast_slice(&tiles),
            usage: wgpu::BufferUsages::VERTEX,
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
            tiles,
            tile_buf,
        }
    }

    fn tile_count(&self) -> u32 {
        u32::try_from(self.tiles.len()).unwrap()
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
        render_pass.set_vertex_buffer(0, self.tile_buf.slice(..));
        render_pass.pop_debug_group();
        render_pass.insert_debug_marker("Draw!");
        render_pass.draw(0..4, 0..self.tile_count());
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
        vertex_buffers: &[wgpu::VertexBufferLayout],
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
        })
    }

    fn create_vertices() -> Vec<Tile> {
        let mut vertex_data = Vec::new();
        let thread_rng = &mut thread_rng();
        for y in -5_i16..5 {
            let bottom = f32::from(y);
            for x in -5_i16..5 {
                let left = f32::from(x);
                let line_pattern = thread_rng.gen_range(0..0x100);
                vertex_data.push(tile([left, bottom, 0.0], LinePattern(line_pattern)));
            }
        }

        vertex_data
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct Tile {
    pos: [f32; 4],
    line_pattern: LinePattern,
}

fn tile(pos: [f32; 3], line_pattern: LinePattern) -> Tile {
    Tile {
        pos: [pos[0], pos[1], pos[2], 1.0],
        line_pattern,
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, Default)]
struct LinePattern(u32);

impl ops::BitOrAssign<LineSegment> for LinePattern {
    fn bitor_assign(&mut self, rhs: LineSegment) {
        self.0 |= 1 << rhs as u32;
    }
}

// TODO W.I.P.
#[allow(dead_code)]
enum LineSegment {
    /// positive x
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
    /// +x, +y
    NECorner = 9,
    /// -x +y
    NWCorner = 11,
    /// -x -y
    SWCorner = 13,
    /// +x -y
    SECorner = 15,
}
