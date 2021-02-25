use std::{collections::HashMap, io::Cursor, path::Path};

use crate::{
  io,
  prelude::*,
  shader::{ActiveShader, BindUniform},
};
use image::{io::Reader as ImageReader, DynamicImage};

fn read_image(bytes: &[u8]) -> Result<DynamicImage> {
  let format = image::guess_format(&bytes)?;
  let mut img_reader = ImageReader::new(Cursor::new(bytes));
  img_reader.set_format(format);
  Ok(img_reader.decode()?)
}

pub struct TextureBuilder<'a> {
  gl: &'a Context,
  tex_parameters: HashMap<u32, u32>,
  flip: bool,
  format: u32,
  alignment: u32,
}

impl<'a> TextureBuilder<'a> {
  pub fn new(gl: &'a Context) -> Self {
    TextureBuilder {
      tex_parameters: hashmap! {
        glow::TEXTURE_WRAP_S => glow::REPEAT,
        glow::TEXTURE_WRAP_T => glow::REPEAT,
        glow::TEXTURE_MIN_FILTER => glow::LINEAR_MIPMAP_LINEAR,
        glow::TEXTURE_MAG_FILTER => glow::LINEAR
      },
      flip: true,
      format: glow::RGBA,
      alignment: 4,
      gl,
    }
  }

  pub fn with_tex_parameter(mut self, parameter: u32, value: u32) -> Self {
    self.tex_parameters.insert(parameter, value);
    self
  }

  pub fn with_flip(mut self, flip: bool) -> Self {
    self.flip = flip;
    self
  }

  pub fn with_format(mut self, format: u32) -> Self {
    self.format = format;
    self
  }
  
  pub fn with_alignment(mut self, alignment: u32) -> Self {
    self.alignment = alignment;
    self
  }

  pub unsafe fn render_texture(self, width: u32, height: u32) -> Result<Texture> {
    let gl = self.gl;

    gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, self.alignment as i32);
    let texture = gl.create_texture().map_err(Error::msg)?;
    gl.bind_texture(glow::TEXTURE_2D, Some(texture));

    let internal_format = match self.format {
      glow::RGB | glow::RGBA => self.format,
      glow::RED => glow::R8,
      _ => unimplemented!()
    };

    gl.tex_image_2d(
      glow::TEXTURE_2D,
      0,
      internal_format as i32,
      width as i32,
      height as i32,
      0,
      self.format,
      glow::UNSIGNED_BYTE,
      None,
    );

    for (key, value) in self.tex_parameters.into_iter() {
      gl.tex_parameter_i32(glow::TEXTURE_2D, key, value as i32);
    }

    gl.bind_texture(glow::TEXTURE_2D, None);

    Ok(Texture {
      texture,
      format: self.format,
    })
  }

  pub unsafe fn from_bytes(self, bytes: &[u8]) -> Result<Texture> {
    let gl = self.gl;
    let image = read_image(bytes)?;
    let image = if self.flip { image.flipv() } else { image };
    let image = image.into_rgba8();

    // Make new texture into TEXTURE_2D global slot
    gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, self.alignment as i32);
    let texture = gl.create_texture().map_err(Error::msg)?;
    gl.bind_texture(glow::TEXTURE_2D, Some(texture));

    // Bind raw image data to the texture and make mipmap
    gl.tex_image_2d(
      glow::TEXTURE_2D,
      0,
      self.format as i32,
      image.width() as i32,
      image.height() as i32,
      0,
      self.format,
      glow::UNSIGNED_BYTE,
      Some(image.as_raw()),
    );
    gl.generate_mipmap(glow::TEXTURE_2D);

    // Set wrapping parameters
    for (key, value) in self.tex_parameters.into_iter() {
      gl.tex_parameter_i32(glow::TEXTURE_2D, key, value as i32);
    }

    gl.bind_texture(glow::TEXTURE_2D, None);

    Ok(Texture {
      texture,
      format: self.format,
    })
  }

  pub async unsafe fn from_file(self, path: impl AsRef<Path>) -> Result<Texture> {
    let bytes = io::load_file(path).await?;
    self.from_bytes(&bytes)
  }
}

#[derive(Clone)]
pub struct Texture {
  pub texture: GlTexture,
  format: u32,
}

impl Texture {
  pub unsafe fn sub_image(
    &self,
    gl: &Context,
    x_offset: u32,
    y_offset: u32,
    width: u32,
    height: u32,
    pixels: &[u8],
  ) {
    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
    gl.tex_sub_image_2d(
      glow::TEXTURE_2D,
      0,
      x_offset as i32,
      y_offset as i32,
      width as i32,
      height as i32,
      self.format,
      glow::UNSIGNED_BYTE,
      glow::PixelUnpackData::Slice(pixels),
    );
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
