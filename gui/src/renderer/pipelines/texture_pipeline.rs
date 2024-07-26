use wgpu::{Device, Queue, RenderPass, RenderPipeline};
use wgpu::util::RenderEncoder;

use crate::renderer::coordinates::{Square, TexRect};
use crate::renderer::pipelines::BufferWrapper;
use crate::shader::texture_shader::bind_groups::BindGroup0;

///A pipeline optimized for rendering images
#[derive(Debug)]
pub struct TexturePipeline {
  pub pipeline: RenderPipeline,
  pub instance_buffer: BufferWrapper,
}

impl TexturePipeline {
  pub fn add<T, S>(&mut self, src: T, dest: S) where T: Into<TexRect>, S: Into<Square> {
    todo!("put instruction into buffer")
  }

  pub fn render<'a>(&mut self, render_pass: &mut RenderPass<'a>, device: &Device, queue: &Queue, bind_group: &'a BindGroup0) {
    render_pass.set_pipeline(&self.pipeline);
    bind_group.set(render_pass);
    render_pass.set_vertex_buffer(1, self.instance_buffer.get_buffer(device, queue).slice(..));
    render_pass.draw(0..4, 0..self.instance_buffer.len);
  }
}