use std::{collections::HashMap, marker::PhantomData, path::Path};

use crate::{
  io,
  prelude::*,
  shader::{ActiveShader, BindUniform},
};
use futures::future::try_join_all;
use image::{DynamicImage, GenericImageView};

// Each marker struct represents a different texture target (e.g. TEXTURE_2D)
#[derive(Clone)]
pub struct T2d;

#[derive(Clone)]
pub struct TCubemap;

pub trait TextureTarget {
  const TARGET: u32;
}

// Associate the corresponding GL constant with each type
impl TextureTarget for T2d {
  const TARGET: u32 = glow::TEXTURE_2D;
}

impl TextureTarget for TCubemap {
  const TARGET: u32 = glow::TEXTURE_CUBE_MAP;
}

pub struct TextureBuilder<'a, Target> {
  gl: &'a Context,
  tex_parameters: HashMap<u32, u32>,
  flip: bool,
  format: u32,
  alignment: u32,
  _marker: PhantomData<Target>,
}

impl<'a> TextureBuilder<'a, T2d> {
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
      _marker: PhantomData,
      gl,
    }
  }

  pub unsafe fn build(self, image: DynamicImage) -> Result<Texture<T2d>> {
    let target = Self::target();
    let internal_format = self.internal_format();
    let (image, (width, height)) = self.convert_image(image);

    // Make new texture into TEXTURE_2D global slot
    let gl = self.gl;
    gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, self.alignment as i32);
    let texture = gl.create_texture().map_err(Error::msg)?;
    gl.bind_texture(target, Some(texture));

    // Bind raw image data to the texture and make mipmap
    gl.tex_image_2d(
      target,
      0,
      internal_format as i32,
      width as i32,
      height as i32,
      0,
      self.format,
      glow::UNSIGNED_BYTE,
      Some(&image),
    );
    gl.generate_mipmap(target);

    // Set wrapping parameters
    Self::apply_texture_parameters(gl, self.tex_parameters);

    gl.bind_texture(target, None);

    Ok(Texture {
      texture,
      format: self.format,
      _marker: PhantomData,
    })
  }

  pub async unsafe fn load(self, path: impl AsRef<Path>) -> Result<Texture<T2d>> {
    let image = io::load_image(path).await?;
    self.build(image)
  }
}

impl<'a> TextureBuilder<'a, TCubemap> {
  pub unsafe fn build(self, images: Vec<DynamicImage>) -> Result<Texture<TCubemap>> {
    let target = Self::target();
    let internal_format = self.internal_format();

    // Parse every image
    let images = images
      .into_iter()
      .map(|image| self.convert_image(image))
      .collect::<Vec<_>>();

    let gl = self.gl;
    let texture = gl.create_texture().map_err(Error::msg)?;
    gl.bind_texture(target, Some(texture));

    // Apply each image to a different section of the cubemap
    for (i, (image, (width, height))) in images.into_iter().enumerate() {
      gl.tex_image_2d(
        glow::TEXTURE_CUBE_MAP_POSITIVE_X + (i as u32),
        0,
        internal_format as i32,
        width as i32,
        height as i32,
        0,
        self.format,
        glow::UNSIGNED_BYTE,
        Some(&image),
      );
    }

    Self::apply_texture_parameters(gl, self.tex_parameters);

    gl.bind_texture(target, None);

    Ok(Texture {
      texture,
      format: self.format,
      _marker: PhantomData,
    })
  }

  pub async unsafe fn load(self, paths: Vec<String>) -> Result<Texture<TCubemap>> {
    let file_futures = paths.into_iter().map(|path| io::load_image(path));
    let all_bytes = try_join_all(file_futures).await?;
    self.build(all_bytes)
  }
}

impl<'a, Target: TextureTarget> TextureBuilder<'a, Target> {
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

  pub fn as_cubemap(self) -> TextureBuilder<'a, TCubemap> {
    self
      .with_tex_parameter(glow::TEXTURE_MIN_FILTER, glow::LINEAR)
      .with_tex_parameter(glow::TEXTURE_MAG_FILTER, glow::LINEAR)
      .with_tex_parameter(glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE)
      .with_tex_parameter(glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE)
      .with_tex_parameter(glow::TEXTURE_WRAP_R, glow::CLAMP_TO_EDGE)
      .with_flip(false)
      .with_target()
  }

  pub fn with_target<S>(self) -> TextureBuilder<'a, S> {
    let TextureBuilder {
      gl,
      tex_parameters,
      flip,
      format,
      alignment,
      ..
    } = self;
    TextureBuilder {
      gl,
      tex_parameters,
      flip,
      format,
      alignment,
      _marker: PhantomData,
    }
  }

  fn internal_format(&self) -> u32 {
    match self.format {
      glow::RGB | glow::RGBA => self.format,
      glow::RED => glow::R8,
      _ => unimplemented!(),
    }
  }

  fn target() -> u32 {
    Target::TARGET
  }

  fn convert_image(&self, image: DynamicImage) -> (Vec<u8>, (u32, u32)) {
    let image = if self.flip { image.flipv() } else { image };
    let dimensions = image.dimensions();
    let bytes = match self.format {
      glow::RGB => image.into_rgb8().into_raw(),
      glow::RGBA => image.into_rgba8().into_raw(),
      _ => unimplemented!(),
    };

    (bytes, dimensions)
  }

  unsafe fn apply_texture_parameters(gl: &Context, tex_parameters: HashMap<u32, u32>) {
    for (key, value) in tex_parameters.into_iter() {
      gl.tex_parameter_i32(Target::TARGET, key, value as i32);
    }
  }

  pub unsafe fn render_texture(self, width: u32, height: u32) -> Result<Texture<Target>> {
    let target = Self::target();
    let internal_format = self.internal_format();
    let gl = self.gl;

    gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, self.alignment as i32);
    let texture = gl.create_texture().map_err(Error::msg)?;
    gl.bind_texture(target, Some(texture));

    gl.tex_image_2d(
      target,
      0,
      internal_format as i32,
      width as i32,
      height as i32,
      0,
      self.format,
      glow::UNSIGNED_BYTE,
      None,
    );

    Self::apply_texture_parameters(gl, self.tex_parameters);

    gl.bind_texture(target, None);

    Ok(Texture {
      texture,
      format: self.format,
      _marker: PhantomData,
    })
  }
}

#[derive(Clone)]
pub struct Texture<Target = T2d> {
  pub texture: GlTexture,
  format: u32,
  _marker: PhantomData<Target>,
}

impl<Target: TextureTarget> Texture<Target> {
  pub unsafe fn sub_image(
    &self,
    gl: &Context,
    x_offset: u32,
    y_offset: u32,
    width: u32,
    height: u32,
    pixels: &[u8],
  ) {
    let target = Target::TARGET;
    gl.bind_texture(target, Some(self.texture));
    gl.tex_sub_image_2d(
      target,
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

impl<Target: TextureTarget> BindUniform for Texture<Target> {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    // TODO: should we be asking for a new texture slot every time? should Target be a param?
    let unit = shader.new_texture_slot();
    shader.bind_uniform(gl, name, &(unit as i32));
    let gl_unit = glow::TEXTURE0 + unit;
    gl.active_texture(gl_unit);
    gl.bind_texture(Target::TARGET, Some(self.texture));
  }
}
