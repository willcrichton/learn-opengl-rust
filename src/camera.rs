use crate::{
  prelude::*,
  shader::{ActiveShader, BindUniform, ShaderTypeDef},
  user_inputs::UserInputs,
};
use winit::event::VirtualKeyCode as Key;

pub struct Camera {
  pub pos: Vec3,
  pub up: Vec3,
  pub speed: f32,
  pub sensitivity: f32,
  pub pitch: f32,
  pub yaw: f32,
  pub projection: Mat4,
}

impl Camera {
  pub fn new(pos: Vec3, projection: Mat4, look_at: Vec3) -> Self {
    let look_dir = glm::normalize(&(pos - look_at));

    Camera {
      pos,
      up: glm::vec3(0., 1., 0.),
      speed: 2.5,
      sensitivity: 0.25,
      yaw: f32::atan2(look_dir.x, look_dir.z).to_degrees(),
      pitch: f32::asin(-look_dir.y).to_degrees(),
      projection,
    }
  }

  pub fn front(&self) -> Vec3 {
    // TODO: why is -90 needed?
    let yaw = (self.yaw - 90.).to_radians();
    let pitch = self.pitch.to_radians();
    glm::vec3(
      yaw.cos() * pitch.cos(),
      pitch.sin(),
      yaw.sin() * pitch.cos(),
    )
  }

  pub fn view_matrix(&self) -> Mat4 {
    // Creates a LookAt matrix defined by rotation^T * -translation
    // where fwd = (pos - target), right = up x fwd, rotation = (right, up, fwd), and translation = pos
    glm::look_at(&self.pos, &(self.pos + self.front()), &self.up)
  }

  pub fn right(&self) -> Vec3 {
    glm::cross(&self.front(), &self.up)
  }

  pub fn speed(&self, dt: f32) -> f32 {
    self.speed * dt
  }

  pub fn update(&mut self, dt: f32, user_inputs: &UserInputs) {
    let (dx, dy) = user_inputs.mouse_delta;
    self.yaw += (dx as f32) * self.sensitivity;
    self.pitch = (self.pitch + (-dy as f32) * self.sensitivity).clamp(-89., 89.);

    let speed = self.speed(dt);
    if user_inputs.pressed(Key::W) {
      self.pos += speed * self.front();
    }
    if user_inputs.pressed(Key::S) {
      self.pos += speed * -self.front();
    }
    if user_inputs.pressed(Key::A) {
      self.pos += speed * -self.right();
    }
    if user_inputs.pressed(Key::D) {
      self.pos += speed * self.right();
    }
  }
}

impl BindUniform for Camera {
  unsafe fn bind_uniform(&self, gl: &Context, shader: &mut ActiveShader, name: &str) {
    shader.bind_uniform(gl, &format!("{}.view_pos", name), &self.pos);
    shader.bind_uniform(gl, &format!("{}.view", name), &self.view_matrix());
    shader.bind_uniform(gl, &format!("{}.projection", name), &self.projection);
  }
}

impl ShaderTypeDef for Camera {
  const TYPE_DEF: &'static str = r#"struct Camera {
  vec3 view_pos;
  mat4 view;
  mat4 projection;    
};"#;
}