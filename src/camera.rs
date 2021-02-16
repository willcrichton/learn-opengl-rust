use crate::{
  shader::{SetUniform, Shader},
  user_inputs::UserInputs,
};
use glm::{Mat4, Vec3};
use glow::Context;
use nalgebra_glm::{self as glm};
use winit::event::VirtualKeyCode as Key;

pub struct Camera {
  pub pos: Vec3,
  pub up: Vec3,
  pub speed: f32,
  pub pitch: f32,
  pub yaw: f32,
  pub projection: Mat4,
}

impl Default for Camera {
  fn default() -> Self {
    Camera {
      pos: glm::zero(),
      up: glm::vec3(0., 1., 0.),
      speed: 2.5,
      yaw: 0.,
      pitch: 0.,
      projection: glm::identity(),
    }
  }
}

impl Camera {
  pub fn front(&self) -> Vec3 {
    let yaw = self.yaw.to_radians();
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
    self.yaw += dx as f32;
    self.pitch = (self.pitch + (-dy as f32)).clamp(-89., 89.);

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

  pub unsafe fn bind(&self, gl: &Context, shader_program: &Shader) {
    shader_program.set_uniform(&gl, "view", &self.view_matrix());
    shader_program.set_uniform(&gl, "projection", &self.projection);
  }
}
