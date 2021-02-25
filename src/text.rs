use glyph_brush::{
  ab_glyph::FontArc, BrushAction, Color, GlyphBrush, GlyphBrushBuilder, GlyphVertex, OwnedSection,
  OwnedText,
};

use crate::{
  io,
  prelude::*,
  shader::Shader,
  texture::{Texture, TextureBuilder},
};
use std::{collections::HashMap, mem::size_of, path::Path};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct TextVertex {
  left_top: Vec2,
  right_bottom: Vec2,
  tex_left_top: Vec2,
  tex_right_bottom: Vec2,
  z: f32,
  color: Vec4,
}

pub struct Font {
  pub name: String,
  glyph_brush: GlyphBrush<TextVertex>,
  texture: Texture,
  vertex_array: GlVertexArray,
  vertex_buffer: GlBuffer,
  vertex_count: u32,
}

impl Font {
  pub async unsafe fn load(gl: &Context, path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref();

    // Give font the same name as the file stem
    let name = path
      .file_stem()
      .context("Path::file_stem")?
      .to_str()
      .context("OsStr::to_str")?
      .to_string();

    // Load font data into glyph_brush
    let bytes = io::load_file(path).await?;
    let font = FontArc::try_from_vec(bytes)?;
    let glyph_brush = GlyphBrushBuilder::using_font(font).build();

    // Make texture that glyph_brush will render into
    let (width, height) = glyph_brush.texture_dimensions();
    let texture = TextureBuilder::new(gl)
      .with_format(glow::RED)
      .with_tex_parameter(glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE)
      .with_tex_parameter(glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE)
      .with_tex_parameter(glow::TEXTURE_MIN_FILTER, glow::LINEAR)
      .with_tex_parameter(glow::TEXTURE_MAG_FILTER, glow::LINEAR)
      .render_texture(width, height)?;

    // Make vertex array/buffer for storing glyph draw locations
    let vertex_array = gl.create_vertex_array().map_err(Error::msg)?;
    gl.bind_vertex_array(Some(vertex_array));

    let vertex_buffer = gl.create_buffer().map_err(Error::msg)?;
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));

    // Set vertex attribute locations to each member of TextVertex struct
    let size_f32 = size_of::<f32>() as i32;
    let sizes = [2, 2, 2, 2, 1, 4];
    let stride = sizes.iter().sum::<i32>() * size_f32;

    let mut offset = 0;
    for (i, size) in sizes.iter().enumerate() {
      gl.enable_vertex_attrib_array(i as u32);
      gl.vertex_attrib_pointer_f32(
        i as u32,
        *size,
        glow::FLOAT,
        false,
        stride,
        offset * size_f32,
      );
      gl.vertex_attrib_divisor(i as u32, 1);
      offset += size;
    }

    gl.bind_vertex_array(None);

    Ok(Font {
      glyph_brush,
      name,
      texture,
      vertex_array,
      vertex_buffer,
      vertex_count: 0,
    })
  }

  unsafe fn queue(&mut self, section: &OwnedSection) {
    self.glyph_brush.queue(section.to_borrowed());
  }

  // Put new draw locations into the array buffer
  unsafe fn upload_vertices(&mut self, gl: &Context, vertices: Vec<TextVertex>) {
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
    let (_, vertices_bytes, _) = vertices.align_to::<u8>();
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_bytes, glow::DYNAMIC_DRAW);
    self.vertex_count = vertices.len() as u32;
  }

  pub unsafe fn draw(
    &mut self,
    gl: &Context,
    shader: &Shader,
    screen_width: u32,
    screen_height: u32,
  ) -> Result<()> {
    let texture = &self.texture;
    let brush_action = self.glyph_brush.process_queued(
      |rect, tex_data| {
        // Update a sub-region of the render texture when requested by glyph_brush
        texture.sub_image(
          gl,
          rect.min[0],
          rect.min[1],
          rect.width(),
          rect.height(),
          tex_data,
        );
      },
      |GlyphVertex {
         tex_coords,
         pixel_coords,
         bounds: _bounds,
         extra,
       }| {
        // TODO: deal with bounds
        TextVertex {
          left_top: glm::vec2(pixel_coords.min.x, pixel_coords.max.y),
          right_bottom: glm::vec2(pixel_coords.max.x, pixel_coords.min.y),
          z: extra.z,
          tex_left_top: glm::vec2(tex_coords.min.x, tex_coords.max.y),
          tex_right_bottom: glm::vec2(tex_coords.max.x, tex_coords.min.y),
          color: glm::vec4(
            extra.color[0],
            extra.color[1],
            extra.color[2],
            extra.color[3],
          ),
        }
      },
    )?;

    gl.bind_vertex_array(Some(self.vertex_array));

    if let BrushAction::Draw(vertices) = brush_action {
      self.upload_vertices(gl, vertices);
    }

    // Translate text from screen coordinates into normalized device coordinates
    let model = &glm::scale2d(
      &glm::translation2d(&glm::vec2(-1.0, -1.0)),
      &glm::vec2(2. / (screen_width as f32), 2. / (screen_height as f32)),
    );

    // Activate text shader and draw
    let mut shader = shader.activate(gl);
    shader.bind_uniform(gl, "font_tex", &self.texture);
    shader.bind_uniform(gl, "model", &model);

    // Instanced means that the vertex shader is run 4 times for each vertex (glyph location)
    gl.draw_arrays_instanced(glow::TRIANGLE_STRIP, 0, 4, self.vertex_count as i32);

    Ok(())
  }
}

pub struct Text {
  section: OwnedSection,
  font: String,
}

impl Text {
  pub fn new(
    text: impl Into<String>,
    font: impl Into<String>,
    font_size: f32,
    color: impl Into<Color>,
    position: Vec2,
  ) -> Self {
    let section = OwnedSection::default()
      .add_text(OwnedText::new(text).with_scale(font_size).with_color(color))
      .with_screen_position((position.x, position.y));
    Text {
      section,
      font: font.into(),
    }
  }

  pub unsafe fn draw(&self, fonts: &mut HashMap<String, Font>) {
    let font = fonts.get_mut(&self.font).unwrap();
    font.queue(&self.section);
  }
}
