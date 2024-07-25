use std::num::NonZeroU64;
use std::ops::{Deref, DerefMut};

use bytemuck::NoUninit;
use wgpu::{Buffer, BufferUsages, Device, Queue, RenderPipeline};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub mod texture_pipeline;

#[derive(Debug)]
pub struct PipelineWrapper {
  pub pipeline: RenderPipeline,
  pub vertex_buffer: BufferWrapper,
  pub index_buffer: BufferWrapper,
}

impl PipelineWrapper {
  pub fn new<V, I>(
    pipeline: RenderPipeline,
    vertex_buffer_descriptor: V,
    index_buffer_descriptor: I,
  ) -> Self
  where
    V: Into<Option<BufferInfo>>,
    I: Into<Option<BufferInfo>>,
  {
    Self {
      pipeline,
      vertex_buffer: BufferWrapper::new(BufferDescriptor::from_info(
        vertex_buffer_descriptor.into().unwrap_or_default(),
        BufferUsages::VERTEX,
      )),
      index_buffer: BufferWrapper::new(BufferDescriptor::from_info(
        index_buffer_descriptor.into().unwrap_or_default(),
        BufferUsages::INDEX,
      )),
    }
  }
}

///A wrapper around a [`Buffer`] which keeps track of the number of elements inside the [`Buffer`]
/// and allocates a new one with more space if needed
#[derive(Debug)]
pub struct BufferWrapper {
  descriptor: BufferDescriptor,
  buffer: Option<Buffer>,
  data: Vec<u8>,
  len: u32,
  dirty: bool,
}

impl BufferWrapper {
  pub fn new(descriptor: BufferDescriptor) -> Self {
    Self {
      descriptor,
      buffer: None,
      data: vec![],
      len: 0,
      dirty: false,
    }
  }

  pub fn add<A>(&mut self, data: &[A])
  where
    A: NoUninit,
  {
    self.data.extend_from_slice(bytemuck::cast_slice(data));
    self.len += data.len() as u32;
    self.dirty = true;
  }

  pub fn clear(&mut self) {
    self.data.clear();
    self.len = 0;
    self.dirty = false;
  }

  pub fn get_buffer(&mut self, device: &Device, queue: &Queue) -> &Buffer {
    if let Some(buffer) = self
      .buffer
      .take()
      .filter(|buffer| buffer.size() as u32 >= self.data.len() as u32)
    {
      if let Some(data_len) = NonZeroU64::new(self.data.len() as u64) {
        if self.dirty {
          queue
            .write_buffer_with(&buffer, 0, data_len)
            .expect("not enough buffer space")
            .copy_from_slice(&self.data);
          self.dirty = false;
        }
      }
      self.buffer.insert(buffer)
    } else {
      let buffer = self
        .buffer
        .insert(device.create_buffer_init(&self.descriptor.to_init_descriptor(&self.data)));
      self.dirty = false;
      buffer
    }
  }

  pub fn len(&self) -> u32 {
    self.len
  }
}

#[derive(Debug, Clone)]
pub struct BufferDescriptor {
  pub info: BufferInfo,
  pub usage: BufferUsages,
}

impl BufferDescriptor {
  pub fn from_info(info: BufferInfo, usage: BufferUsages) -> Self {
    Self { info, usage }
  }

  pub fn new(usage: BufferUsages) -> Self {
    Self::from_info(BufferInfo::default(), usage)
  }

  pub fn to_init_descriptor<'a>(&'a self, data: &'a [u8]) -> BufferInitDescriptor<'a> {
    BufferInitDescriptor {
      label: self.label.as_deref(),
      contents: data,
      usage: self.usage,
    }
  }
}

impl Deref for BufferDescriptor {
  type Target = BufferInfo;

  fn deref(&self) -> &Self::Target {
    &self.info
  }
}

impl DerefMut for BufferDescriptor {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.info
  }
}

#[derive(Debug, Clone, Default)]
pub struct BufferInfo {
  pub label: Option<String>,
}

impl BufferInfo {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_label<S>(mut self, label: S) -> Self
  where
    S: ToString,
  {
    self.label = Some(label.to_string());
    self
  }
}

pub trait HasVertexBuffer {
  fn vertex_buffer(&mut self, device: &Device, queue: &Queue) -> &Buffer;
}

pub trait HasIndexBuffer {
  fn index_buffer(&mut self, device: &Device, queue: &Queue) -> &Buffer;
}
