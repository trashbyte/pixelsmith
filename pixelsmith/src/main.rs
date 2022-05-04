mod app;
mod canvas;
mod geometry;
mod pipeline;
mod registry;
mod palette;
mod scene;
mod recent;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    app::App::new(&event_loop).run(event_loop);
}
