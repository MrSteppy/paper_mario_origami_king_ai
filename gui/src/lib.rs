use std::default::Default;
use std::thread;
use std::time::Duration;

use pollster::FutureExt;
use wgpu::SurfaceError;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Icon, WindowAttributes, WindowId};

use crate::app_state::AppState;
use crate::renderer::Renderer;
use crate::resources::{include_resource_bytes, load_icon};

mod app_state;
mod renderer;
mod resources;
mod shader;

pub fn run(event_loop: EventLoop<AppEvent>) {
  env_logger::init();
  let proxy = event_loop.create_proxy();
  event_loop.set_control_flow(ControlFlow::Wait);

  let app_icon =
    load_icon(include_resource_bytes!(icon/icon.png)).expect("failed to load app icon");

  event_loop
    .run_app(&mut App::new(app_icon))
    .expect("failed to run app");

  //send animation tick every 50ms (20tps)
  thread::spawn(move || {
    while proxy.send_event(AppEvent::AnimationTick).is_ok() {
      thread::sleep(Duration::from_millis(50));
    }
  });
}

#[derive(Debug)]
struct App {
  state: AppState,
  render_state: Option<Renderer>,
  app_icon: Option<Icon>,
}

impl App {
  pub fn new<I>(app_icon: I) -> Self
  where
    I: Into<Option<Icon>>,
  {
    Self {
      state: AppState::default(),
      render_state: None,
      app_icon: app_icon.into(),
    }
  }
}

impl ApplicationHandler<AppEvent> for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    #[allow(unused_mut)]
    let mut window_attributes = WindowAttributes::default()
      .with_title("Paper Mario: The Origami King AI")
      .with_inner_size(PhysicalSize::new(600, 800))
      .with_window_icon(self.app_icon.clone());
    #[cfg(target_os = "windows")]
    {
      use winit::platform::windows::WindowAttributesExtWindows;
      window_attributes = window_attributes.with_taskbar_icon(self.app_icon.clone());
    }

    let window = event_loop
      .create_window(window_attributes)
      .expect("failed to create window");
    self.render_state = Some(Renderer::new(window).block_on());
  }

  fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
    if let Some(render_state) = &self.render_state {
      match event {
        AppEvent::AnimationTick => {
          self.state.height -= 1;
        }
      }

      render_state.window().request_redraw();
    }
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::Resized(size) => {
        if let Some(render_state) = &mut self.render_state {
          render_state.resize(size);
        }
      }
      WindowEvent::RedrawRequested => {
        if let Some(render_state) = &mut self.render_state {
          if let Err(e) = render_state.render(&self.state) {
            match e {
              SurfaceError::Lost => render_state.resize(render_state.size()),
              SurfaceError::OutOfMemory => event_loop.exit(),
              _ => eprintln!("render error: {}", e),
            }
          }
        }
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      _ => {}
    }
  }

  fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
    self.render_state = None;
  }
}

#[derive(Debug)]
pub enum AppEvent {
  ///Will be sent every 50ms (20 tps)
  AnimationTick,
}
