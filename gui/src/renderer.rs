use std::iter::once;
use std::sync::Arc;

use glam::{Vec3, Vec4};
use wgpu::{BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Face, FrontFace, Instance, LoadOp, MultisampleState, Operations, PolygonMode, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, VertexStepMode};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::app_state::AppState;
use crate::shader::shader;
use crate::shader::shader::VertexInput;

const BACKGROUND_COLOR: Color = Color {
  r: 0.0,
  g: 0.2,
  b: 0.0,
  a: 1.0,
};

//vertices in counter-clockwise order: top, bottom left, bottom right
const VERTICES: &[VertexInput] = &[
  VertexInput {
    position: Vec3::new(0.0, 0.5, 0.0),
    color: Vec4::new(0.0, 1.0, 0.0, 1.0),
    _padding: 0.0,
  },
  VertexInput {
    position: Vec3::new(-0.5, -0.5, 0.0),
    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
    _padding: 0.0,
  },
  VertexInput {
    position: Vec3::new(0.5, -0.5, 0.0),
    color: Vec4::new(0.0, 0.0, 1.0, 1.0),
    _padding: 0.0,
  },
];

#[derive(Debug)]
pub struct Renderer {
  surface: Surface<'static>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  window: Arc<Window>,
  size: PhysicalSize<u32>,
  render_pipeline: RenderPipeline,
  vertex_buffer: Buffer,
}

/*
TODO
 pipelines + shader:
  thin ring
  wide ring
  lines
  textures
*/

impl Renderer {
  pub async fn new(window: Window) -> Self {
    let window = Arc::new(window);
    let size = window.inner_size();

    let instance = Instance::default();

    let surface = instance
      .create_surface(window.clone())
      .expect("failed to create surface");

    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: Default::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
      })
      .await
      .expect("can't find appropriate adapter");

    let (device, queue) = adapter
      .request_device(&DeviceDescriptor::default(), None)
      .await
      .expect("failed to create logical device");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
      .formats
      .iter()
      .find(|f| f.is_srgb())
      .copied()
      .unwrap_or(surface_caps.formats[0]);

    let config = SurfaceConfiguration {
      usage: TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: PresentMode::default(),
      desired_maximum_frame_latency: 2,
      alpha_mode: CompositeAlphaMode::Auto,
      view_formats: vec![],
    };

    surface.configure(&device, &config);

    let shader = shader::create_shader_module(&device);
    let render_pipeline_layout = shader::create_pipeline_layout(&device);

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: shader::vertex_state(&shader, &shader::vs_main_entry(VertexStepMode::Vertex)),
      fragment: Some(shader::fragment_state(
        &shader,
        &shader::fs_main_entry([Some(ColorTargetState {
          format: config.format,
          blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        })]),
      )),
      primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: FrontFace::Ccw,
        cull_mode: Some(Face::Back),
        polygon_mode: PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      depth_stencil: None,
      multisample: MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
      cache: None, //TODO might be interesting to improve performance on android
    });

    //create vertex buffer
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(VERTICES),
      usage: BufferUsages::VERTEX,
    });

    Self {
      surface,
      device,
      queue,
      config,
      window,
      size,
      render_pipeline,
      vertex_buffer,
    }
  }

  pub fn size(&self) -> PhysicalSize<u32> {
    self.size
  }

  pub fn resize(&mut self, size: PhysicalSize<u32>) {
    if size.width > 0 && size.height > 0 {
      self.size = size;
      self.config.width = size.width;
      self.config.height = size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  pub fn render(&self, _app_state: &AppState) -> Result<(), SurfaceError> {
    let canvas = self.surface.get_current_texture()?;
    let view = canvas
      .texture
      .create_view(&TextureViewDescriptor::default());
    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });
    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
      label: Some("Render Pass"),
      color_attachments: &[Some(RenderPassColorAttachment {
        view: &view,
        resolve_target: None,
        ops: Operations {
          load: LoadOp::Clear(BACKGROUND_COLOR),
          store: StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
    });

    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.draw(0..VERTICES.len() as u32, 0..1);

    drop(render_pass); //must be dropped before the encoder can be finished

    self.queue.submit(once(encoder.finish()));
    canvas.present();

    Ok(())
  }

  pub fn window(&self) -> &Window {
    &self.window
  }
}
