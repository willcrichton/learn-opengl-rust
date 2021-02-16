use std::path::Path;

use crate::io;
use anyhow::{Context as AnyhowContext, Error, Result};
use glow::{Context, HasContext};
use image::{ImageFormat, RgbaImage};

pub struct Texture {
  image: Option<RgbaImage>,
  texture: Option<<Context as HasContext>::Texture>,
}

impl Texture {
  pub async fn load(path: impl AsRef<Path>, format: ImageFormat) -> Result<Self> {
    let image = Some(
      io::load_image(path, format)
        .await?
        .flipv() // GL expects (0, 0) is bottom-left of image so flip vertically
        .into_rgba8(),
    ); // Keep all images as RGBA8 for simplicity

    Ok(Texture {
      image,
      texture: None,
    })
  }

  pub unsafe fn build_texture(&mut self, gl: &Context) -> Result<()> {
    let image = self.image.take().context("Texture has been built twice")?;
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
    self.texture = Some(texture);
    Ok(())
    // NOTE: image is implicitly deallocated here
  }

  pub unsafe fn bind(&self, gl: &Context, slot: Option<u32>) {
    if let Some(unit) = slot {
      gl.active_texture(unit);
    }

    gl.bind_texture(glow::TEXTURE_2D, self.texture);
  }
}
