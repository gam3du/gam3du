#![allow(missing_docs, reason = "TODO remove before release")]

pub trait RendererBuilder {
    type Renderer: Renderer;

    fn build(
        self,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::SurfaceConfiguration,
    ) -> Self::Renderer;
}

pub trait Renderer {
    fn update(&mut self);
    fn try_update(&mut self) -> bool;

    fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::SurfaceConfiguration,
    );

    fn render(
        &mut self,
        texture_view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
}
