use egui_aurora_app::app::{GlowApp, UserEvent};

fn main() {
    let event_loop = winit::event_loop::EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("failed to build event loop");
    let proxy = event_loop.create_proxy();

    let mut app = GlowApp::new(proxy);
    event_loop.run_app(&mut app).expect("failed to run app");
}
