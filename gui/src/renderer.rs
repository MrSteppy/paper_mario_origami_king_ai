use std::iter::once;
use std::ops::Deref;
use std::sync::Arc;

use glam::{Vec3, Vec4};
use wgpu::{BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Extent3d, Face, FilterMode, IndexFormat, Instance, LoadOp, Operations, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, SamplerDescriptor, StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, VertexStepMode};
use wgpu::util::{BufferInitDescriptor, DeviceExt, TextureDataOrder};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::app_state::AppState;
use crate::resources::include_resource_bytes;
use crate::shader::{shader, texture_shader};
use crate::shader::shader::VertexInput;

mod pipelines;
mod coordinates;

const BACKGROUND_COLOR: Color = Color {
  r: 0.0,
  g: 0.2,
  b: 0.0,
  a: 1.0,
};

//vertices in counter-clockwise order: top, bottom left, bottom right
const VERTICES: &[VertexInput] = &[
  //top
  VertexInput {
    position: Vec3::new(0.0, 0.5, 0.0),
    color: Vec4::new(0.0, 1.0, 0.0, 1.0),
    _padding: 0.0,
  },
  //bottom left
  VertexInput {
    position: Vec3::new(-0.5, -0.5, 0.0),
    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
    _padding: 0.0,
  },
  //bottom right
  VertexInput {
    position: Vec3::new(0.5, -0.5, 0.0),
    color: Vec4::new(0.0, 0.0, 1.0, 1.0),
    _padding: 0.0,
  },
  //left
  VertexInput {
    position: Vec3::new(-0.75, 0.15, 0.0),
    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
    _padding: 0.0,
  },
  //right
  VertexInput {
    position: Vec3::new(0.75, 0.15, 0.0),
    color: Vec4::new(0.0, 0.0, 0.0, 1.0),
    _padding: 0.0,
  },
];

#[rustfmt::skip]
const INDICES: &[u16] = &[
  0, 3, 1,
  0, 1, 2,
  0, 2, 4,
];

#[derive(Debug)]
pub struct Renderer {
  surface: Surface<'static>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  window: Arc<Window>,
  size: PhysicalSize<u32>,
  texture_bind_group: texture_shader::bind_groups::BindGroup0,
  texture_pipeline: RenderPipeline,
  tutorial_pipeline: RenderPipeline,
  vertex_buffer: Buffer,
  index_buffer: Buffer,
}

/*
TODO
 pipelines + shader:
  circle
  ring
  line
  texture
  pixel (for text rendering)
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

    //load texture
    let sprite = image::load_from_memory(include_resource_bytes!(texture / steptech_logo.png))
      .expect("failed to load sprite")
      .to_rgba8();
    let (width, height) = sprite.dimensions();
    let texture = device.create_texture_with_data(
      &queue,
      &TextureDescriptor {
        label: Some("StepTechLogo"),
        size: Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      },
      TextureDataOrder::default(),
      &sprite,
    );
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
      label: Some("Sampler"),
      mag_filter: FilterMode::Linear,
      ..Default::default()
    });
    let texture_bind_group = texture_shader::bind_groups::BindGroup0::from_bindings(
      &device,
      texture_shader::bind_groups::BindGroupLayout0 {
        texture: &texture_view,
        t_sampler: &sampler,
      },
    );

    //texture pipeline
    let texture_shader = texture_shader::create_shader_module(&device);
    let color_target_state = [Some(ColorTargetState {
      format: config.format,
      blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
      write_mask: ColorWrites::ALL,
    })];
    let texture_pipeline_layout = texture_shader::create_pipeline_layout(&device);
    let texture_vertex_entry = texture_shader::vs_main_entry(VertexStepMode::Vertex);
    let texture_fragment_entry = texture_shader::fs_main_entry(color_target_state.clone());
    let texture_pipeline_descriptor = RenderPipelineDescriptor {
      layout: Some(&texture_pipeline_layout),
      vertex: texture_shader::vertex_state(&texture_shader, &texture_vertex_entry),
      fragment: Some(texture_shader::fragment_state(
        &texture_shader,
        &texture_fragment_entry,
      )),
      primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleStrip,
        cull_mode: Some(Face::Back),
        ..Default::default()
      },
      label: Some("Render Pipeline"),
      depth_stencil: None,
      multisample: Default::default(),
      multiview: None,
      cache: None, //TODO might be interesting to improve performance on android
    };
    let texture_pipeline = device.create_render_pipeline(&texture_pipeline_descriptor);

    //tutorial render pipeline
    let shader = shader::create_shader_module(&device);
    let tutorial_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      layout: Some(&shader::create_pipeline_layout(&device)),
      vertex: shader::vertex_state(&shader, &shader::vs_main_entry(VertexStepMode::Vertex)),
      fragment: Some(shader::fragment_state(
        &shader,
        &shader::fs_main_entry(color_target_state.clone()),
      )),
      ..texture_pipeline_descriptor.clone()
    });

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(VERTICES),
      usage: BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(INDICES),
      usage: BufferUsages::INDEX,
    });

    Self {
      surface,
      device,
      queue,
      config,
      window,
      size,
      texture_bind_group,
      texture_pipeline,
      tutorial_pipeline,
      vertex_buffer,
      index_buffer,
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
      ..Default::default()
    });

    render_pass.set_pipeline(&self.tutorial_pipeline);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
    render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

    drop(render_pass); //must be dropped before the encoder can be finished


    self.queue.submit(once(encoder.finish()));
    canvas.present();

    Ok(())
  }

  pub fn window(&self) -> &Window {
    &self.window
  }
}
