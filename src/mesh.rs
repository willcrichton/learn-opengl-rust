use crate::{material::Material, model::Model, prelude::*, shader::ActiveShader};
use std::mem::size_of;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vertex {
  pub position: Vec3,
  pub normal: Vec3,
  pub tex_coords: Vec2,
}

impl Vertex {
  pub fn from_flat_array(values: &[f32]) -> Vec<Vertex> {
    values
      .chunks(8)
      .map(|chunk| Vertex {
        position: glm::vec3(chunk[0], chunk[1], chunk[2]),
        normal: glm::vec3(chunk[3], chunk[4], chunk[5]),
        tex_coords: glm::vec2(chunk[6], chunk[7]),
      })
      .collect()
  }
}

#[derive(Clone)]
pub struct Mesh {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u32>,
  pub material: Option<Material>,

  vao: GlVertexArray,
  vbo: GlBuffer,
  ebo: GlBuffer,
}

impl Mesh {
  pub unsafe fn new(
    gl: &Context,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    material: Option<Material>,
  ) -> Result<Mesh> {
    // Vertex array
    let vao = gl.create_vertex_array().map_err(Error::msg)?;
    gl.bind_vertex_array(Some(vao));

    // Index buffer
    let ebo = gl.create_buffer().map_err(Error::msg)?;
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

    let (_, indices_bytes, _) = indices.align_to::<u8>();
    gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_bytes, glow::STATIC_DRAW);

    // Vertex buffer
    let vbo = gl.create_buffer().map_err(Error::msg)?;
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

    let (_, vertices_bytes, _) = vertices.align_to::<u8>();
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_bytes, glow::STATIC_DRAW);

    // Vertex buffer attributes
    let size_f32 = size_of::<f32>() as i32;
    let sizes = [3, 3, 2];
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
      offset += size;
    }

    // Reset vertex array
    gl.bind_vertex_array(None);

    Ok(Mesh {
      vertices,
      indices,
      material,
      vao,
      ebo,
      vbo,
    })
  }

  pub unsafe fn draw(&self, gl: &Context, shader: &mut ActiveShader) {
    if let Some(material) = self.material.as_ref() {
      shader.bind_uniform(gl, "material", material);
    }

    gl.bind_vertex_array(Some(self.vao));
    gl.draw_elements(
      glow::TRIANGLES,
      self.indices.len() as i32,
      glow::UNSIGNED_INT,
      0,
    );
    gl.bind_vertex_array(None);

    shader.reset_textures();
  }

  pub fn to_model(self) -> Model {
    Model { meshes: vec![self] }
  }
}
