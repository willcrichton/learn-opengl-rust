use crate::prelude::*;
use crate::shader::{BindUniform, Shader};

pub struct Light {
  pub position: Vec3,
  pub ambient: Vec3,
  pub diffuse: Vec3,
  pub specular: Vec3,
}

impl BindUniform for Light {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    self
      .position
      .bind_uniform(gl, shader, &format!("{}.position", name));
    self
      .ambient
      .bind_uniform(gl, shader, &format!("{}.ambient", name));
    self
      .diffuse
      .bind_uniform(gl, shader, &format!("{}.diffuse", name));
    self
      .specular
      .bind_uniform(gl, shader, &format!("{}.specular", name));
  }
}
