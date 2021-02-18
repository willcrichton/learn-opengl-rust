use crate::prelude::*;

#[derive(BindUniform, ShaderTypeDef)]
pub struct PointLight {
  pub position: Vec3,
  pub ambient: Vec3,
  pub diffuse: Vec3,
  pub specular: Vec3,
}

#[derive(BindUniform, ShaderTypeDef)]
pub struct DirLight {
  pub direction: Vec3,
  pub ambient: Vec3,
  pub diffuse: Vec3,
  pub specular: Vec3,
}
