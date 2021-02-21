use crate::{prelude::*, texture::Texture};

#[derive(BindUniform, ShaderTypeDef, Clone)]
pub struct Material {
  pub diffuse: Texture,
  pub specular: Texture,
  pub shininess: f32,
}
