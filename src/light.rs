use crate::prelude::*;

#[derive(BindUniform, ShaderTypeDef)]
pub struct DirLight {
  pub direction: Vec3,

  pub ambient: Vec3,
  pub diffuse: Vec3,
  pub specular: Vec3,
}

#[derive(BindUniform, ShaderTypeDef)]
pub struct PointLight {
  pub position: Vec3,

  pub ambient: Vec3,
  pub diffuse: Vec3,
  pub specular: Vec3,

  pub constant: f32,
  pub linear: f32,
  pub quadratic: f32,
}

#[derive(BindUniform, ShaderTypeDef)]
pub struct SpotLight {
  pub position: Vec3,
  pub direction: Vec3,
  pub inner_cut_off: f32,
  pub outer_cut_off: f32,

  pub diffuse: Vec3,
  pub specular: Vec3,

  pub constant: f32,
  pub linear: f32,
  pub quadratic: f32,
}
