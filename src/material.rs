use crate::prelude::*;

use crate::shader::{BindUniform, Shader};

pub struct Material {
  pub ambient: Vec3,
  pub diffuse: Vec3,
  pub specular: Vec3,
  pub shininess: f32,
}

impl BindUniform for Material {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    self
      .ambient
      .bind_uniform(gl, shader, &format!("{}.ambient", name));
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
