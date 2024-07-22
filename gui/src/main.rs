use winit::event_loop::EventLoop;

use gui::run;

fn main() {
  let event_loop = EventLoop::new().expect("Failed to create event loop");
  run(event_loop);
}