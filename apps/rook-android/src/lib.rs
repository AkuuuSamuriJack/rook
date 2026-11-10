mod platform;

use std::sync::Arc;

use glam::Vec3;
use rook_render::{Camera, Renderer, Scene};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::android::{
        activity::AndroidApp,
        EventLoopBuilderExtAndroid,
    },
    window::{Window, WindowId},
};

struct Drawing {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    camera: Camera,
}

#[derive(Default)]
struct RookAndroid {
    window: Option<Arc<Window>>,
    drawing: Option<Drawing>,
}

impl RookAndroid {
    fn create_renderer(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("Failed to create Android window"),
        );

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        );

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create GPU surface");

        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            },
        ))
        .expect("No compatible GPU adapter found");

        let (device, queue) = pollster::block_on(
            adapter.request_device(&Renderer::device_descriptor(&adapter)),
        )
        .expect("Failed to create GPU device");

        let size = window.inner_size();

        let mut config = surface
            .get_default_config(
                &adapter,
                size.width.max(1),
                size.height.max(1),
            )
            .expect("GPU surface is not supported");

        let capabilities = surface.get_capabilities(&adapter);

        if let Some(format) = capabilities.formats.iter().find(|f| f.is_srgb()) {
            config.format = *format;
        }

        config.present_mode = wgpu::PresentMode::AutoVsync;
        config.desired_maximum_frame_latency = 1;

        surface.configure(&device, &config);

        let mut renderer =
            Renderer::new(&adapter, device, queue, config.format);

        renderer.set_scene(&Scene::default());

        let camera = Camera::look_at(
            Vec3::new(0.0, 20.0, 40.0),
            Vec3::ZERO,
        );

        self.drawing = Some(Drawing {
            surface,
            config,
            renderer,
            camera,
        });

        self.window = Some(window);
    }

    fn draw_frame(&mut self) {
        let Some(window) = &self.window else {
            return;
        };

        let Some(drawing) = &mut self.drawing else {
            return;
        };

        let frame = match drawing.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => {
                drawing.surface.configure(
                    &drawing.renderer.device,
                    &drawing.config,
                );
                return;
            }
        };

        let view = frame.texture.create_view(&Default::default());

        drawing.renderer.set_time(0.0);

        drawing.renderer.render(
            &view,
            drawing.config.width,
            drawing.config.height,
            &drawing.camera,
        );

        window.pre_present_notify();
        drawing.renderer.queue.present(frame);
    }
}

impl ApplicationHandler for RookAndroid {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.create_renderer(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(drawing) = &mut self.drawing {
                        drawing.config.width = size.width;
                        drawing.config.height = size.height;

                        drawing.surface.configure(
                            &drawing.renderer.device,
                            &drawing.config,
                        );
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                self.draw_frame();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn android_main(app: AndroidApp) {
    platform::install().expect("Failed to install Android platform");

    let mut builder = EventLoop::builder();
    builder.with_android_app(app);

    let event_loop = builder
        .build()
        .expect("Failed to create event loop");

    let mut player = RookAndroid::default();

    event_loop
        .run_app(&mut player)
        .expect("Android event loop failed");
}