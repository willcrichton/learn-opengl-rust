use std::path::Path;

use crate::{io, prelude::*};
use tokio::try_join;

pub struct Shader {
  id: GlProgram,
}

impl Shader {
  pub async unsafe fn load(
    gl: &Context,
    vertex_path: impl AsRef<Path>,
    fragment_path: impl AsRef<Path>,
  ) -> Result<Self> {
    let (vertex_source, fragment_source) =
      try_join!(io::load_shader(vertex_path), io::load_shader(fragment_path))?;
    Ok(Self::new(gl, vertex_source, fragment_source))
  }

  pub unsafe fn new(gl: &Context, mut vertex_source: String, mut fragment_source: String) -> Self {
    // Add directives needed for each platform
    let header = if cfg!(target_arch = "wasm32") {
      "#version 300 es\nprecision highp float;"
    } else {
      "#version 330 core"
    };

    // Add struct definitions for all types in the crate
    let defs = [
      crate::camera::Camera::TYPE_DEF,
      crate::material::Material::TYPE_DEF,
      crate::light::PointLight::TYPE_DEF,
      crate::light::DirLight::TYPE_DEF,
      crate::light::SpotLight::TYPE_DEF,
    ]
    .join("\n");

    let preprocess = |source| format!("{}\n{}\n{}", header, defs, source);

    vertex_source = preprocess(vertex_source);
    fragment_source = preprocess(fragment_source);

    // Compile individual shaders into OpenGL objects
    let vertex_shader = Self::build_shader(&gl, glow::VERTEX_SHADER, &vertex_source);
    let fragment_shader = Self::build_shader(&gl, glow::FRAGMENT_SHADER, &fragment_source);

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

  unsafe fn build_shader(gl: &Context, shader_type: u32, source: &str) -> GlShader {
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

  unsafe fn location(&self, gl: &Context, name: &str) -> Option<GlUniformLocation> {
    gl.get_uniform_location(self.id, name)
  }

  // I wanted to call this "use" but that's a Rust keyword :'(
  pub unsafe fn activate(&self, gl: &Context) {
    gl.use_program(Some(self.id));
  }

  pub unsafe fn bind_uniform<T: BindUniform>(&self, gl: &Context, name: &str, value: &T) {
    value.bind_uniform(gl, self, name);
  }
}

pub trait ShaderTypeDef {
  const TYPE_DEF: &'static str;
}

// A Rustic way to expose the uniform_* methods is to have a single trait which
// we implement for each type.
pub trait BindUniform {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str);
}

impl BindUniform for [f32; 4] {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    gl.uniform_4_f32(
      shader.location(gl, name).as_ref(),
      self[0],
      self[1],
      self[2],
      self[3],
    );
  }
}

impl BindUniform for i32 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    gl.uniform_1_i32(shader.location(gl, name).as_ref(), *self);
  }
}

impl BindUniform for f32 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    gl.uniform_1_f32(shader.location(gl, name).as_ref(), *self);
  }
}

impl BindUniform for u32 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    gl.uniform_1_u32(shader.location(gl, name).as_ref(), *self);
  }
}

impl BindUniform for Vec3 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    gl.uniform_3_f32(shader.location(gl, name).as_ref(), self.x, self.y, self.z);
  }
}

impl BindUniform for Mat4 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &Shader, name: &str) {
    gl.uniform_matrix_4_f32_slice(shader.location(gl, name).as_ref(), false, self.as_slice());
  }
}
