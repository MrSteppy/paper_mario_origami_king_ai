use std::iter::once;
use std::sync::Arc;

use wgpu::{
  BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, CompositeAlphaMode,
  Device, DeviceDescriptor, Face, FragmentState, FrontFace, Instance, LoadOp, MultisampleState,
  Operations, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState,
  PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
  RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp,
  Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, VertexState,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::app_state::AppState;
use crate::resources::include_resource_str;

const BACKGROUND_COLOR: Color = Color {
  r: 0.0,
  g: 1.0,
  b: 0.0,
  a: 1.0,
};

#[derive(Debug)]
pub struct Renderer {
  surface: Surface<'static>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  window: Arc<Window>,
  size: PhysicalSize<u32>,
  render_pipeline: RenderPipeline,
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

    let shader = device.create_shader_module(ShaderModuleDescriptor {
      label: Some("Shader"),
      source: ShaderSource::Wgsl(include_resource_str!(shader/shader.wgsl).into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[],
      push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: VertexState {
        module: &shader,
        entry_point: "vs_main",
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(FragmentState {
        module: &shader,
        entry_point: "fs_main",
        compilation_options: Default::default(),
        targets: &[Some(ColorTargetState {
          format: config.format,
          blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        })],
      }),
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

    Self {
      surface,
      device,
      queue,
      config,
      window,
      size,
      render_pipeline,
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
    render_pass.draw(0..3, 0..1);

    drop(render_pass); //must be dropped before the encoder can be finished

    self.queue.submit(once(encoder.finish()));
    canvas.present();

    Ok(())
  }

  pub fn window(&self) -> &Window {
    &self.window
  }
}
