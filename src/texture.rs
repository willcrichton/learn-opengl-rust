use std::path::Path;

use crate::{
  io,
  prelude::*,
  shader::{BindUniform, Shader},
};
use image::ImageFormat;

pub struct Texture {
  texture: GlTexture,
  unit: u32,
}

impl Texture {
  pub async fn load(
    gl: &Context,
    path: impl AsRef<Path>,
    format: ImageFormat,
    unit: u32,
  ) -> Result<Self> {
    let image = io::load_image(path, format)
      .await?
      .flipv() // GL expects (0, 0) is bottom-left of image so flip vertically
      .into_rgba8();

    unsafe {
      let texture = gl.create_texture().map_err(Error::msg)?;
      gl.bind_texture(glow::TEXTURE_2D, Some(texture));
      gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA as i32,
        image.width() as i32,
        image.height() as i32,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        Some(image.as_raw()),
      );
      gl.generate_mipmap(glow::TEXTURE_2D);

      gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
      gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
      gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR_MIPMAP_LINEAR as i32,
      );
      gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
      );

      Ok(Texture { texture, unit })
    }
  }
}

impl BindUniform for Texture {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    shader.bind_uniform(gl, name, &(self.unit as i32));
    let unit = match self.unit {
      0 => glow::TEXTURE0,
      1 => glow::TEXTURE1,
      _ => unimplemented!(),
    };
    gl.active_texture(unit);
    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
  }
}
