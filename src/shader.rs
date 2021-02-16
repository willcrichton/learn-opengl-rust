use glow::{Context, HasContext};
use nalgebra_glm::Mat4;

pub struct Shader {
  id: <Context as HasContext>::Program,
}

impl Shader {
  pub unsafe fn new(gl: &Context, vertex_source: &str, fragment_source: &str) -> Self {
    // Compile individual shaders into OpenGL objects
    let vertex_shader = Self::build_shader(&gl, glow::VERTEX_SHADER, vertex_source);
    let fragment_shader = Self::build_shader(&gl, glow::FRAGMENT_SHADER, fragment_source);

    // Link shaders into a single program
    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(shader_program, vertex_shader);
    gl.attach_shader(shader_program, fragment_shader);

    gl.link_program(shader_program);
    if !gl.get_program_link_status(shader_program) {
      panic!(
        "Shader program failed to link with error: {}",
        gl.get_program_info_log(shader_program)
      );
    }

    // Cleanup shaders after linking
    gl.delete_shader(vertex_shader);
    gl.delete_shader(fragment_shader);

    Shader { id: shader_program }
  }

  unsafe fn build_shader(
    gl: &Context,
    shader_type: u32,
    source: &str,
  ) -> <Context as HasContext>::Shader {
    // Create a new OpenGL shader object
    let shader = gl.create_shader(shader_type).unwrap();

    // Pass source to OpenGL
    gl.shader_source(shader, source);

    // Call the OpenGL shader compiler
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
      panic!(
        "Shader failed to compile with error: {}",
        gl.get_shader_info_log(shader)
      );
    }

    return shader;
  }

  unsafe fn location(
    &self,
    gl: &Context,
    name: &str,
  ) -> Option<<Context as HasContext>::UniformLocation> {
    gl.get_uniform_location(self.id, name)
  }

  // I wanted to call this "use" but that's a Rust keyword :'(
  pub unsafe fn activate(&self, gl: &Context) {
    gl.use_program(Some(self.id));
  }
}

// A Rustic way to expose the uniform_* methods is to have a single polymorphic trait which
// we implement for each type.
pub trait SetUniform<T> {
  unsafe fn set_uniform(&self, gl: &Context, name: &str, value: &T);
}

impl SetUniform<[f32; 4]> for Shader {
  unsafe fn set_uniform(&self, gl: &Context, name: &str, value: &[f32; 4]) {
    gl.uniform_4_f32(
      self.location(gl, name).as_ref(),
      value[0],
      value[1],
      value[2],
      value[3],
    );
  }
}

impl SetUniform<i32> for Shader {
  unsafe fn set_uniform(&self, gl: &Context, name: &str, value: &i32) {
    gl.uniform_1_i32(self.location(gl, name).as_ref(), *value);
  }
}

impl SetUniform<Mat4> for Shader {
  unsafe fn set_uniform(&self, gl: &Context, name: &str, value: &Mat4) {
    gl.uniform_matrix_4_f32_slice(self.location(gl, name).as_ref(), false, value.as_slice());
  }
}
