mod platform;

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

#[derive(Default)]
struct RookAndroid {
    window: Option<Window>,
}

impl ApplicationHandler for RookAndroid {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.window = Some(
                event_loop
                    .create_window(Window::default_attributes())
                    .expect("Failed to create Android window"),
            );
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn android_main(app: AndroidApp) {
    platform::install().expect("Failed to install Android platform");

    let mut builder = EventLoop::builder();
    builder.with_android_app(app);

    let event_loop = builder.build().expect("Failed to create event loop");
    let mut player = RookAndroid::default();

    event_loop
        .run_app(&mut player)
        .expect("Android event loop failed");
}