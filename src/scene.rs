use crate::{camera::Camera, light::Light, material::Material, prelude::*, shader::Shader};

use std::mem::size_of;
use tokio::try_join;

pub struct Scene {
  cube_vao: GlVertexArray,

  light_vao: GlVertexArray,
  light: Light,

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

    let light = Light {
      position: glm::vec3(1.2, 0., 2.),
      ambient: glm::vec3(0.2, 0.2, 0.2),
      diffuse: glm::vec3(0.5, 0.5, 0.5),
      specular: glm::vec3(1., 1., 1.),
    };

    lighting_shader.activate(&gl);
    lighting_shader.bind_uniform(
      &gl,
      "material",
      &Material {
        ambient: glm::vec3(1., 0.5, 0.31),
        diffuse: glm::vec3(1., 0.5, 0.31),
        specular: glm::vec3(0.5, 0.5, 0.5),
        shininess: 32.,
      },
    );

    Ok(Scene {
      cube_vao,
      light_vao,
      light,
      lighting_shader,
      light_cube_shader,
    })
  }

  pub fn update(&mut self, elapsed: f32) {
    self.light.position = glm::vec3(elapsed.cos() * 1.5, 0., elapsed.sin() * 1.5);

    let light_color = glm::vec3(
      (elapsed * 2.).sin(),
      (elapsed * 0.7).sin(),
      (elapsed * 1.3).sin(),
    );
    self.light.ambient = light_color * 0.2;
    self.light.diffuse = light_color * 0.5;
  }

  pub unsafe fn draw(&self, gl: &Context, camera: &Camera) {
    // Draw cube to be lit
    self.lighting_shader.activate(gl);
    self.lighting_shader.bind_uniform(&gl, "light", &self.light);
    self
      .lighting_shader
      .bind_uniform(gl, "model", &glm::identity());
    self.lighting_shader.bind_uniform(gl, "camera", camera);

    gl.bind_vertex_array(Some(self.cube_vao));
    gl.draw_arrays(glow::TRIANGLES, 0, 36);

    // Draw light cube
    self.light_cube_shader.activate(gl);

    let mut model = glm::translation(&self.light.position);
    model = glm::scale(&model, &glm::vec3(0.2, 0.2, 0.2));
    self.light_cube_shader.bind_uniform(gl, "model", &model);
    self.light_cube_shader.bind_uniform(gl, "camera", camera);

    gl.bind_vertex_array(Some(self.light_vao));
    gl.draw_arrays(glow::TRIANGLES, 0, 36);
  }
}
