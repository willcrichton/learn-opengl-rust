use crate::{prelude::*, texture::Texture};

use crate::shader::{BindUniform, Shader};

pub struct Material {
  pub diffuse: Texture,
  pub specular: Texture,
  pub shininess: f32,
}

impl BindUniform for Material {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    self
      .diffuse
      .bind_uniform(gl, shader, &format!("{}.diffuse", name));
    self
      .specular
      .bind_uniform(gl, shader, &format!("{}.specular", name));
    self
      .shininess
      .bind_uniform(gl, shader, &format!("{}.shininess", name));
  }
}
