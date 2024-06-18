use camera::Camera;
use cube::Cube;
use projection::Projection;
use std::time::Instant;

mod camera;
mod cube;
mod projection;

pub(crate) struct Scene {
    start_time: Instant,
    projection: Projection,
    camera: Camera,
    cube: Cube,
}

fn elapsed_as_vec(start_time: Instant) -> [u32; 2] {
    let elapsed = start_time.elapsed();
    let seconds = u32::try_from(elapsed.as_secs()).unwrap();
    let subsec_nanos = u64::from(elapsed.subsec_nanos());
    // map range of nanoseconds to value range of u32 with rounding
    let subseconds = ((subsec_nanos << u32::BITS) + 500_000_000) / 1_000_000_000;

    [seconds, u32::try_from(subseconds).unwrap()]
}

impl Scene {
    pub(crate) fn init(
        surface: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let cube = Cube::new(device, queue, surface.view_formats[0]);

        let projection = Projection::new_perspective(
            (surface.width, surface.height),
            45_f32.to_radians(),
            1.0..10.0,
        );

        let camera = Camera::new(glam::Vec3::new(1.5f32, -5.0, 3.0), glam::Vec3::ZERO);

        let start_time = Instant::now();

        // Done
        Scene {
            start_time,
            projection,
            camera,
            cube,
        }
    }

    // fn update_rotation(&mut self, queue: &wgpu::Queue) {
    //     self.cube.rotation = Self::generate_rotation();
    //     self.update_matrix(queue);
    // }

    // fn generate_matrix(
    //     projection: glam::Mat4,
    //     view: glam::Mat4,
    //     rotation: glam::Mat4,
    // ) -> glam::Mat4 {
    //     projection * view * rotation
    // }

    // fn update_matrix(&mut self, queue: &wgpu::Queue) {
    //     let matrix = Self::generate_matrix(
    //         self.projection.matrix(),
    //         self.camera.matrix(),
    //         self.cube.rotation,
    //     );

    //     let mx_ref: &[f32; 16] = matrix.as_ref();
    //     queue.write_buffer(&self.cube.matrix_buf, 0, bytemuck::cast_slice(mx_ref));
    // }

    pub(crate) fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.projection
            .set_surface_dimensions((config.width, config.height));
        // self.update_matrix(queue);
    }

    pub(crate) fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        //self.update_rotation(queue);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.push_debug_group("Prepare data for draw.");

            self.cube.render(
                queue,
                &mut rpass,
                &self.camera,
                &self.projection,
                self.start_time,
            );
        }

        queue.submit(Some(encoder.finish()));
    }
}
