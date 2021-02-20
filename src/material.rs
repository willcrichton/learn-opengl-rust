use crate::{prelude::*, texture::Texture};

#[derive(BindUniform, ShaderTypeDef, Clone)]
pub struct Material {
  pub diffuse: Vec<Texture>,
  pub specular: Vec<Texture>,
  pub shininess: f32,
}
