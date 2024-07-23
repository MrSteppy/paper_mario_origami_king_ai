use winit::event_loop::EventLoop;

use gui::run;

fn main() {
  let event_loop = EventLoop::with_user_event()
    .build()
    .expect("Failed to create event loop");
  run(event_loop);
}
