mod floor;
// mod robot;

use crate::{RenderState, SharedGameState};
use core::f32;
use floor::FloorRenderer;
use gam3du_framework::renderer;
use glam::{Mat4, Quat, Vec3};
use lib_geometry::Projection;
use lib_gltf_model::GltfModelRenderer;
// use robot::RobotRenderer;
use std::{
    borrow::Cow,
    fs::read_to_string,
    sync::{Arc, TryLockError},
};

pub struct RendererBuilder {
    game_state: SharedGameState,
    shader_source: Cow<'static, str>,
}

impl RendererBuilder {
    #[must_use]
    pub fn new(game_state: SharedGameState) -> Self {
        let shader_source = read_to_string("engines/robot/shaders/robot.wgsl")
            .unwrap()
            .into();

        Self {
            game_state,
            shader_source,
        }
    }
}

impl renderer::RendererBuilder for RendererBuilder {
    type Renderer = Renderer;

    #[must_use]
    fn build(
        self,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::SurfaceConfiguration,
    ) -> Renderer {
        let game_state = Arc::clone(&self.game_state);
        let state = RenderState::new(&game_state.read().unwrap());

        let projection = Projection::new_perspective(
            (surface.width, surface.height),
            65_f32.to_radians(),
            1.0..15.0,
        );

        let depth_map = DepthTexture::create_depth_texture(device, surface, "depth_map");

        // let robot_renderer = RobotRenderer::new(device, queue, surface.view_formats[0]);
        let floor_renderer = FloorRenderer::new(device, queue, surface.view_formats[0], &state);

        let depth_stencil_state = DepthTexture::depth_stencil_state();
        let gltf_model_renderer = GltfModelRenderer::new(
            device,
            queue,
            surface.view_formats[0],
            self.shader_source,
            depth_stencil_state,
        );

        Renderer {
            game_state,
            projection,
            depth_map,
            state,
            // robot_renderer,
            floor_renderer,
            gltf_model_renderer,
        }
    }
}

pub struct Renderer {
    game_state: SharedGameState,
    // TODO check whether `projection` should be moved into `RenderState`
    projection: Projection,
    depth_map: DepthTexture,
    state: RenderState,
    // robot_renderer: RobotRenderer,
    floor_renderer: FloorRenderer,
    gltf_model_renderer: GltfModelRenderer,
}

impl Renderer {
    fn render_gltf_model(
        &mut self,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        let render_pass_color_attachment = wgpu::RenderPassColorAttachment {
            view: texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        };
        let color_attachments = [Some(render_pass_color_attachment)];
        let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_map.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        };
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: Some(render_pass_depth_stencil_attachment.clone()),
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);

            let world_matrix = Mat4::from_scale_rotation_translation(
                Vec3::new(0.5, 0.5, 0.5),
                Quat::from_rotation_z(self.state.animation_angle + f32::consts::FRAC_PI_2),
                self.state.animation_position + Vec3::new(0.0, 0.0, 0.5),
            );

            self.gltf_model_renderer.render(
                queue,
                &mut render_pass,
                world_matrix,
                &self.state.camera,
                &self.projection,
                self.state.robot_color,
                self.state.start_time,
            );
        }
    }

    fn render_robot(
        &mut self,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        _queue: &wgpu::Queue,
    ) {
        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        let render_pass_color_attachment = wgpu::RenderPassColorAttachment {
            view: texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
        };
        let color_attachments = [Some(render_pass_color_attachment)];
        let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_map.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        };
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: Some(render_pass_depth_stencil_attachment.clone()),
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        {
            let mut _render_pass = encoder.begin_render_pass(&render_pass_descriptor);

            // self.robot_renderer
            //     .render(queue, &mut render_pass, &self.state, &self.projection);
        }
    }

    fn render_floor(
        &mut self,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        let render_pass_color_attachment = wgpu::RenderPassColorAttachment {
            view: texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        };
        let color_attachments = [Some(render_pass_color_attachment)];

        let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_map.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: Some(render_pass_depth_stencil_attachment),
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);

            self.floor_renderer
                .render(queue, &mut render_pass, &mut self.state, &self.projection);
        }
    }
}

impl renderer::Renderer for Renderer {
    fn update(&mut self) {
        self.state.update(&self.game_state.read().unwrap());
    }

    fn try_update(&mut self) -> bool {
        let game_state = &self.game_state.try_read();
        match game_state {
            Ok(game_state) => {
                self.state.update(game_state);
                true
            }
            Err(TryLockError::WouldBlock) => false,
            Err(TryLockError::Poisoned(err)) => panic!("lock poisoned: {err}"),
        }
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        surface: &wgpu::SurfaceConfiguration,
    ) {
        self.projection
            .set_surface_dimensions((surface.width, surface.height));
        self.depth_map = DepthTexture::create_depth_texture(device, surface, "depth_map");
    }

    fn render(
        &mut self,
        texture_view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.render_robot(texture_view, &mut encoder, queue);
        self.render_gltf_model(texture_view, &mut encoder, queue);
        self.render_floor(texture_view, &mut encoder, queue);

        queue.submit(Some(encoder.finish()));
    }
}

pub(super) struct DepthTexture {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
}

impl DepthTexture {
    pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    pub(crate) fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            _texture: texture,
            view,
            _sampler: sampler,
        }
    }

    pub(super) fn depth_stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: DepthTexture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // 1.
            stencil: wgpu::StencilState::default(),     // 2.
            bias: wgpu::DepthBiasState::default(),
        }
    }
}
