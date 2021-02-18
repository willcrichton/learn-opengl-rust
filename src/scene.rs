use crate::{
  camera::Camera,
  light::{DirLight, PointLight, SpotLight},
  material::Material,
  prelude::*,
  shader::Shader,
  texture::Texture,
};

use image::ImageFormat;
use std::mem::size_of;
use tokio::try_join;

pub struct Scene {
  cube_vao: GlVertexArray,
  material: Material,

  light_vao: GlVertexArray,
  light: SpotLight,

  lighting_shader: Shader,
  light_cube_shader: Shader,
}

impl Scene {
  pub async unsafe fn build(gl: &Context) -> Result<Self> {
    #[rustfmt::skip]
    let vertices = [
      // positions          // normals           // texture coords
      -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 0.0,
       0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 0.0,
       0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 1.0,
       0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 1.0,
      -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 1.0,
      -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 0.0,

      -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,   0.0, 0.0,
       0.5, -0.5,  0.5,  0.0,  0.0, 1.0,   1.0, 0.0,
       0.5,  0.5,  0.5,  0.0,  0.0, 1.0,   1.0, 1.0,
       0.5,  0.5,  0.5,  0.0,  0.0, 1.0,   1.0, 1.0,
      -0.5,  0.5,  0.5,  0.0,  0.0, 1.0,   0.0, 1.0,
      -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,   0.0, 0.0,

      -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 0.0,
      -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,  1.0, 1.0,
      -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 1.0,
      -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 1.0,
      -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,  0.0, 0.0,
      -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 0.0,

       0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0, 0.0,
       0.5,  0.5, -0.5,  1.0,  0.0,  0.0,  1.0, 1.0,
       0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0, 1.0,
       0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0, 1.0,
       0.5, -0.5,  0.5,  1.0,  0.0,  0.0,  0.0, 0.0,
       0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0, 0.0,

      -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0, 1.0,
       0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  1.0, 1.0,
       0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0, 0.0,
       0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0, 0.0,
      -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  0.0, 0.0,
      -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0, 1.0,

      -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 1.0,
       0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  1.0, 1.0,
       0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0, 0.0,
       0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0, 0.0,
      -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  0.0, 0.0,
      -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 1.0f32
    ];

    // Setup shared vertex buffer object
    let cube_vbo = gl.create_buffer().map_err(Error::msg)?;
    let (_, vertices_bytes, _) = vertices.align_to::<u8>();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_bytes, glow::STATIC_DRAW);

    let stride = 8 * size_of::<f32>() as i32;
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

      gl.vertex_attrib_pointer_f32(
        2,
        2,
        glow::FLOAT,
        false,
        stride,
        6 * size_of::<f32>() as i32,
      );
      gl.enable_vertex_attrib_array(2);
    };

    // Cube to be lit
    let cube_vao = gl.create_vertex_array().map_err(Error::msg)?;
    bind_data(cube_vao);

    // Light cube
    let light_vao = gl.create_vertex_array().map_err(Error::msg)?;
    bind_data(light_vao);

    // Load all the shaders
    let (lighting_shader, light_cube_shader, diffuse_map, specular_map) = try_join!(
      Shader::load(
        gl,
        "assets/shaders/colors.vert",
        "assets/shaders/colors.frag",
      ),
      Shader::load(
        gl,
        "assets/shaders/light_cube.vert",
        "assets/shaders/light_cube.frag",
      ),
      Texture::load(gl, "assets/textures/container2.png", ImageFormat::Png, 0),
      Texture::load(
        gl,
        "assets/textures/container2_specular.png",
        ImageFormat::Png,
        1
      )
    )?;

    // let light = DirLight {
    //   direction: glm::vec3(-0.2, -1.0, -0.3),
    //   ambient: glm::vec3(0.2, 0.2, 0.2),
    //   diffuse: glm::vec3(0.5, 0.5, 0.5),
    //   specular: glm::vec3(1., 1., 1.),
    // };

    // let light = PointLight {
    //   position: glm::vec3(1.2, 0., 2.),
    //   ambient: glm::vec3(0.2, 0.2, 0.2),
    //   diffuse: glm::vec3(0.5, 0.5, 0.5),
    //   specular: glm::vec3(1., 1., 1.),
    //   constant: 1.,
    //   linear: 0.09,
    //   quadratic: 0.032
    // };

    let light = SpotLight {
      position: glm::zero(),
      direction: glm::zero(),
      inner_cut_off: 12.5_f32.to_radians().cos(),
      outer_cut_off: 17.5_f32.to_radians().cos(),
      diffuse: glm::vec3(0.5, 0.5, 0.5),
      specular: glm::vec3(1., 1., 1.),
      constant: 1.,
      linear: 0.09,
      quadratic: 0.032,
    };

    let material = Material {
      diffuse: diffuse_map,
      specular: specular_map,
      shininess: 64.,
    };

    Ok(Scene {
      cube_vao,
      light_vao,
      light,
      material,
      lighting_shader,
      light_cube_shader,
    })
  }

  pub fn update(&mut self, _elapsed: f32, camera: &Camera) {
    //self.light.position = glm::vec3(elapsed.cos() * 1.5, 0., elapsed.sin() * 1.5);s
    self.light.position = camera.pos;
    self.light.direction = camera.front();
  }

  pub unsafe fn draw(&self, gl: &Context, camera: &Camera) {
    // Draw cube to be lit
    self.lighting_shader.activate(gl);
    self
      .lighting_shader
      .bind_uniform(&gl, "global_light", &self.light);
    self
      .lighting_shader
      .bind_uniform(&gl, "material", &self.material);
    self.lighting_shader.bind_uniform(gl, "camera", camera);

    gl.bind_vertex_array(Some(self.cube_vao));

    let cube_positions = vec![
      glm::vec3(0.0f32, 0.0, 0.0),
      glm::vec3(2.0, 5.0, -15.0),
      glm::vec3(-1.5, -2.2, -2.5),
      glm::vec3(-3.8, -2.0, -12.3),
      glm::vec3(2.4, -0.4, -3.5),
      glm::vec3(-1.7, 3.0, -7.5),
      glm::vec3(1.3, -2.0, -2.5),
      glm::vec3(1.5, 2.0, -2.5),
      glm::vec3(1.5, 0.2, -1.5),
      glm::vec3(-1.3, 1.0, -1.5),
    ];
    for (i, pos) in cube_positions.iter().enumerate() {
      let model = glm::rotate(
        &glm::translation(pos),
        (20. * (i as f32)).to_radians(),
        &glm::vec3(1., 0.3, 0.5),
      );
      self.lighting_shader.bind_uniform(gl, "model", &model);
      gl.draw_arrays(glow::TRIANGLES, 0, 36);
    }

    // // Draw light cube
    // self.light_cube_shader.activate(gl);

    // let mut model = glm::translation(&self.light.position);
    // model = glm::scale(&model, &glm::vec3(0.2, 0.2, 0.2));
    // self.light_cube_shader.bind_uniform(gl, "model", &model);
    // self.light_cube_shader.bind_uniform(gl, "camera", camera);

    // gl.bind_vertex_array(Some(self.light_vao));
    // gl.draw_arrays(glow::TRIANGLES, 0, 36);
  }
}
