use crate::{
  camera::Camera,
  shader::{SetUniform, Shader},
};
use anyhow::{Error, Result};
use glm::Vec3;
use glow::{Context, HasContext};
use nalgebra_glm as glm;
use std::mem::size_of;
use tokio::try_join;

pub type GlVertexArray = <Context as HasContext>::VertexArray;

pub struct Scene {
  cube_vao: GlVertexArray,

  light_vao: GlVertexArray,
  light_pos: Vec3,

  lighting_shader: Shader,
  light_cube_shader: Shader,
}

impl Scene {
  pub async unsafe fn build(gl: &Context) -> Result<Self> {
    #[rustfmt::skip]
    let vertices = [
     -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
      0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
      0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
      0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
     -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
     -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
 
     -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,
      0.5, -0.5,  0.5,  0.0,  0.0, 1.0,
      0.5,  0.5,  0.5,  0.0,  0.0, 1.0,
      0.5,  0.5,  0.5,  0.0,  0.0, 1.0,
     -0.5,  0.5,  0.5,  0.0,  0.0, 1.0,
     -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,
 
     -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,
     -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,
     -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
     -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
     -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,
     -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,
 
      0.5,  0.5,  0.5,  1.0,  0.0,  0.0,
      0.5,  0.5, -0.5,  1.0,  0.0,  0.0,
      0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
      0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
      0.5, -0.5,  0.5,  1.0,  0.0,  0.0,
      0.5,  0.5,  0.5,  1.0,  0.0,  0.0,
 
     -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
      0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
      0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
      0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
     -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
     -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
 
     -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
      0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
      0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
      0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
     -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
     -0.5,  0.5, -0.5,  0.0,  1.0,  0.0f32
    ];

    // Setup shared vertex buffer object
    let cube_vbo = gl.create_buffer().map_err(Error::msg)?;
    let (_, vertices_bytes, _) = vertices.align_to::<u8>();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_bytes, glow::STATIC_DRAW);

    let stride = 6 * size_of::<f32>() as i32;
    let bind_data = |vao| {
      gl.bind_vertex_array(Some(vao));
      gl.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));

      gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
      gl.enable_vertex_attrib_array(0);

      gl.vertex_attrib_pointer_f32(
        1,
        3,
        glow::FLOAT,
        false,
        stride,
        3 * size_of::<f32>() as i32,
      );
      gl.enable_vertex_attrib_array(1);
    };

    // Cube to be lit
    let cube_vao = gl.create_vertex_array().map_err(Error::msg)?;
    bind_data(cube_vao);

    // Light cube
    let light_pos = glm::vec3(1.2, 0., 2.);
    let light_vao = gl.create_vertex_array().map_err(Error::msg)?;
    bind_data(light_vao);

    // Load all the shaders
    let platform = if cfg!(target_arch = "wasm32") {
      "web"
    } else {
      "native"
    };
    let (lighting_shader, light_cube_shader) = try_join!(
      Shader::load(
        gl,
        format!("assets/shaders/{}/colors.vert", platform),
        format!("assets/shaders/{}/colors.frag", platform),
      ),
      Shader::load(
        gl,
        format!("assets/shaders/{}/light_cube.vert", platform),
        format!("assets/shaders/{}/light_cube.frag", platform),
      )
    )?;

    lighting_shader.activate(&gl);
    lighting_shader.set_uniform(&gl, "objectColor", &glm::vec3(1., 0.5, 0.31));
    lighting_shader.set_uniform(&gl, "lightColor", &glm::vec3(1., 1., 1.));

    Ok(Scene {
      cube_vao,
      light_vao,
      light_pos,

      lighting_shader,
      light_cube_shader,
    })
  }

  pub fn update(&mut self, elapsed: f32) {
    self.light_pos = glm::vec3(elapsed.cos(), 0., elapsed.sin());
  }

  pub unsafe fn draw(&self, gl: &Context, camera: &Camera) {
    // Draw cube to be lit
    self.lighting_shader.activate(gl);
    self.lighting_shader.set_uniform(gl, "viewPos", &camera.pos);
    self
      .lighting_shader
      .set_uniform(gl, "lightPos", &self.light_pos);
    self
      .lighting_shader
      .set_uniform(gl, "model", &glm::identity());
    camera.bind(gl, &self.lighting_shader);

    gl.bind_vertex_array(Some(self.cube_vao));
    gl.draw_arrays(glow::TRIANGLES, 0, 36);

    // Draw light cube
    self.light_cube_shader.activate(gl);

    let mut model = glm::translation(&self.light_pos);
    model = glm::scale(&model, &glm::vec3(0.2, 0.2, 0.2));
    self.light_cube_shader.set_uniform(gl, "model", &model);

    camera.bind(gl, &self.light_cube_shader);

    gl.bind_vertex_array(Some(self.light_vao));
    gl.draw_arrays(glow::TRIANGLES, 0, 36);
  }
}
