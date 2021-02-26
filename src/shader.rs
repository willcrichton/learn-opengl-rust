use glm::Mat3;
use na::{
  dimension::{U1, U3, U4},
  storage::Storage,
};
use std::{marker::PhantomData, mem::size_of, path::Path, slice};
use std140::ReprStd140;

use crate::{io, prelude::*};

pub struct Shader {
  id: GlProgram,
}

impl Shader {
  pub async unsafe fn load(
    gl: &Context,
    vertex_path: impl AsRef<Path>,
    fragment_path: impl AsRef<Path>,
  ) -> Result<Self> {
    let vertex_path = vertex_path.as_ref();
    let (vertex_source, fragment_source) =
      try_join!(io::load_string(vertex_path), io::load_string(fragment_path))?;
    Self::new(gl, vertex_source, fragment_source)
      .context(format!("With shader path {:?}", vertex_path))
  }

  pub unsafe fn new(
    gl: &Context,
    mut vertex_source: String,
    mut fragment_source: String,
  ) -> Result<Self> {
    // Add directives needed for each platform
    let header = if cfg!(target_arch = "wasm32") {
      "#version 300 es\nprecision highp float;"
    } else {
      "#version 330 core"
    };

    // Add struct definitions for all types in the crate
    let defs = [
      crate::camera::CameraBlock::BLOCK_DEF,
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
    let vertex_shader = Self::build_shader(&gl, glow::VERTEX_SHADER, &vertex_source)?;
    let fragment_shader = Self::build_shader(&gl, glow::FRAGMENT_SHADER, &fragment_source)?;

    // Link shaders into a single program
    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(shader_program, vertex_shader);
    gl.attach_shader(shader_program, fragment_shader);

    gl.link_program(shader_program);
    if !gl.get_program_link_status(shader_program) {
      bail!(
        "Shader program failed to link with error: {}",
        gl.get_program_info_log(shader_program)
      );
    }

    // Cleanup shaders after linking
    gl.delete_shader(vertex_shader);
    gl.delete_shader(fragment_shader);

    Ok(Shader { id: shader_program })
  }

  unsafe fn build_shader(gl: &Context, shader_type: u32, source: &str) -> Result<GlShader> {
    // Create a new OpenGL shader object
    let shader = gl.create_shader(shader_type).unwrap();

    // Pass source to OpenGL
    gl.shader_source(shader, source);

    // Call the OpenGL shader compiler
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
      bail!(
        "{} shader failed to compile with error: {}",
        match shader_type {
          glow::VERTEX_SHADER => "Vertex",
          glow::FRAGMENT_SHADER => "Fragment",
          _ => "???",
        },
        gl.get_shader_info_log(shader)
      );
    }

    Ok(shader)
  }

  unsafe fn location(&self, gl: &Context, name: &str) -> Option<GlUniformLocation> {
    gl.get_uniform_location(self.id, name)
  }

  unsafe fn block_location(&self, gl: &Context, name: &str) -> Option<u32> {
    gl.get_uniform_block_index(self.id, name)
  }

  fn program(&self) -> GlProgram {
    self.id
  }

  // I wanted to call this "use" but that's a Rust keyword :'(
  pub unsafe fn activate(&self, gl: &Context) -> ActiveShader {
    gl.use_program(Some(self.id));
    ActiveShader::new(self)
  }
}

// Trait for custom shader structs that contains a GLSL type definition
pub trait ShaderTypeDef {
  const TYPE_DEF: &'static str;
}

pub trait ShaderBlockDef {
  const BLOCK_DEF: &'static str;
}

pub struct ActiveShader<'a> {
  shader: &'a Shader,
  num_textures: u32,
}

// TODO: this API still doesn't feel quite right wrt handling texture slots
impl<'a> ActiveShader<'a> {
  pub fn new(shader: &'a Shader) -> Self {
    ActiveShader {
      shader,
      num_textures: 0,
    }
  }

  pub fn new_texture_slot(&mut self) -> u32 {
    let slot = self.num_textures;
    self.num_textures += 1;
    slot
  }

  pub fn program(&self) -> GlProgram {
    self.shader.program()
  }

  pub unsafe fn bind_uniform<T: BindUniform>(&mut self, gl: &Context, name: &str, value: &T) {
    value.bind_uniform(gl, self, name);
  }

  pub unsafe fn location(&self, gl: &Context, name: &str) -> Option<GlUniformLocation> {
    self.shader.location(gl, name)
  }

  pub unsafe fn block_location(&self, gl: &Context, name: &str) -> Option<u32> {
    self.shader.block_location(gl, name)
  }

  pub fn reset_textures(&mut self) {
    self.num_textures = 0;
  }
}

// A Rustic way to expose the uniform_* methods is to have a single trait which
// we implement for each type.
pub trait BindUniform {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str);
}

impl<T: BindUniform> BindUniform for Vec<T> {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    shader.bind_uniform(gl, &format!("{}_len", name), &(self.len() as i32));
    for (i, value) in self.iter().enumerate() {
      shader.bind_uniform(gl, &format!("{}[{}]", name, i), value);
    }
  }
}

impl<T: BindUniform> BindUniform for &T {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    (*self).bind_uniform(gl, shader, name);
  }
}

impl BindUniform for [f32; 4] {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
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
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    gl.uniform_1_i32(shader.location(gl, name).as_ref(), *self);
  }
}

impl BindUniform for f32 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    gl.uniform_1_f32(shader.location(gl, name).as_ref(), *self);
  }
}

impl BindUniform for u32 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    gl.uniform_1_u32(shader.location(gl, name).as_ref(), *self);
  }
}

impl BindUniform for Vec3 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    gl.uniform_3_f32(shader.location(gl, name).as_ref(), self.x, self.y, self.z);
  }
}

impl BindUniform for Mat3 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    gl.uniform_matrix_3_f32_slice(shader.location(gl, name).as_ref(), false, self.as_slice());
  }
}

impl BindUniform for Mat4 {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    gl.uniform_matrix_4_f32_slice(shader.location(gl, name).as_ref(), false, self.as_slice());
  }
}

// Represents a data buffer that passes uniforms to shaders.
// Data must be laid out in the std140 layout, which is enforced by the
// std140::ReprStd140 trait.
pub struct UniformBlock<T: ReprStd140> {
  ubo: GlBuffer,
  binding: u32,
  _marker: PhantomData<T>,
}

impl<T: ReprStd140> UniformBlock<T> {
  // TODO: auto-generate binding slots?
  pub unsafe fn new(gl: &Context, binding: u32) -> Result<Self> {
    // Pre-allocate size_of<T> bytes
    let ubo = gl.create_buffer().map_err(Error::msg)?;
    gl.bind_buffer(glow::UNIFORM_BUFFER, Some(ubo));
    gl.buffer_data_size(
      glow::UNIFORM_BUFFER,
      size_of::<T>() as i32,
      glow::STATIC_DRAW,
    );
    gl.bind_buffer(glow::UNIFORM_BUFFER, None);

    // Put the buffer at the given binding
    gl.bind_buffer_base(glow::UNIFORM_BUFFER, binding, Some(ubo));

    Ok(UniformBlock {
      ubo,
      binding,
      _marker: PhantomData,
    })
  }

  // Copy value into the uniform buffer
  pub unsafe fn upload(&self, gl: &Context, value: &T) {
    gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.ubo));

    // Small hack to convert &T into &[u8]
    let data = slice::from_raw_parts(value as *const T as *const u8, size_of::<T>());

    gl.buffer_sub_data_u8_slice(glow::UNIFORM_BUFFER, 0, data);
    gl.bind_buffer(glow::UNIFORM_BUFFER, None);
  }
}

impl<T: ReprStd140> BindUniform for UniformBlock<T> {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    gl.uniform_block_binding(
      shader.program(),
      shader.block_location(gl, name).unwrap(),
      self.binding,
    );
  }
}

// Utility trait to convert nalgebra types into std140 types
pub trait GlmStd140Ext<T> {
  fn to_std140(&self) -> T;
}

impl<S> GlmStd140Ext<std140::vec3> for na::Matrix<f32, U3, U1, S>
where
  S: Storage<f32, U3, U1>,
{
  fn to_std140(&self) -> std140::vec3 {
    std140::vec3(self[0], self[1], self[2])
  }
}

impl<S> GlmStd140Ext<std140::vec4> for na::Matrix<f32, U4, U1, S>
where
  S: Storage<f32, U4, U1>,
{
  fn to_std140(&self) -> std140::vec4 {
    std140::vec4(self[0], self[1], self[2], self[3])
  }
}

impl GlmStd140Ext<std140::mat4x4> for Mat4 {
  fn to_std140(&self) -> std140::mat4x4 {
    std140::mat4x4(
      self.column(0).to_std140(),
      self.column(1).to_std140(),
      self.column(2).to_std140(),
      self.column(3).to_std140(),
    )
  }
}
