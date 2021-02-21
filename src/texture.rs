use std::{io::Cursor, path::Path};

use crate::{
  io,
  prelude::*,
  shader::{ActiveShader, BindUniform},
};
use image::{io::Reader as ImageReader, DynamicImage};

#[derive(Clone)]
pub struct Texture {
  texture: GlTexture,
}

fn read_image(bytes: &[u8]) -> Result<DynamicImage> {
  let format = image::guess_format(&bytes)?;
  let mut img_reader = ImageReader::new(Cursor::new(bytes));
  img_reader.set_format(format);
  Ok(img_reader.decode()?)
}

impl Texture {
  pub unsafe fn new(gl: &Context, bytes: &[u8], flip: bool) -> Result<Self> {
    let image = read_image(bytes)?;
    let image = if flip { image.flipv() } else { image };
    let image = image.into_rgba8();

    // Make new texture into TEXTURE_2D global slot
    let texture = gl.create_texture().map_err(Error::msg)?;
    gl.bind_texture(glow::TEXTURE_2D, Some(texture));

    // Bind raw image data to the texture and make mipmap
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

    // Set wrapping parameters
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

    Ok(Texture { texture })
  }

  pub async unsafe fn load(gl: &Context, path: impl AsRef<Path>, flip: bool) -> Result<Self> {
    // Load image from disk
    let bytes = io::load_file(path).await?;
    Self::new(gl, &bytes, flip)
  }
}

impl BindUniform for Texture {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    let unit = shader.new_texture_slot();
    shader.bind_uniform(gl, name, &(unit as i32));
    let gl_unit = glow::TEXTURE0 + unit;
    gl.active_texture(gl_unit);
    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
  }
}
