use std::ops::Deref;

use wgpu::{Buffer, Device, Queue, RenderPipeline};

use crate::renderer::pipelines::{HasIndexBuffer, HasVertexBuffer, PipelineWrapper};

#[derive(Debug)]
pub struct TexturePipeline {
  pub wrapper: PipelineWrapper,
}

impl TexturePipeline {
  //TODO add fn draw()
}

impl Deref for TexturePipeline {
  type Target = RenderPipeline;

  fn deref(&self) -> &Self::Target {
    &self.wrapper.pipeline
  }
}

impl HasVertexBuffer for TexturePipeline {
  fn vertex_buffer(&mut self, device: &Device, queue: &Queue) -> &Buffer {
    self.wrapper.vertex_buffer.get_buffer(device, queue)
  }
}

impl HasIndexBuffer for TexturePipeline {
  fn index_buffer(&mut self, device: &Device, queue: &Queue) -> &Buffer {
    self.wrapper.index_buffer.get_buffer(device, queue)
  }
}