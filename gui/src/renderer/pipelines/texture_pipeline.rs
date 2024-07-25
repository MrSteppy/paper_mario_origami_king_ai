use wgpu::RenderPipeline;

use crate::renderer::pipelines::BufferWrapper;

#[derive(Debug)]
pub struct TexturePipeline {
  pub pipeline: RenderPipeline,
  pub instance_buffer: BufferWrapper,
}

impl TexturePipeline {

}