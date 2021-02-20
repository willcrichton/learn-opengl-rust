use crate::{
  camera::Camera,
  light::{DirLight, PointLight, SpotLight},
  model::Model,
  prelude::*,
  shader::Shader,
};

pub struct Scene {
  model: Model,

  pointlight_base: PointLight,
  spotlight: SpotLight,
  sun: DirLight,

  lighting_shader: Shader,
}

impl Scene {
  pub async unsafe fn build(gl: &Context) -> Result<Self> {
    // Load all the assets
    let (lighting_shader, model) = try_join!(
      Shader::load(
        gl,
        "assets/shaders/colors.vert",
        "assets/shaders/colors.frag"
      ),
      Model::load(gl, "assets/models/backpack")
    )?;

    let pointlight_base = PointLight {
      position: glm::vec3(1.2, 0., 2.),
      ambient: glm::vec3(0.2, 0.2, 0.2),
      diffuse: glm::vec3(0.5, 0.5, 0.5),
      specular: glm::vec3(1., 1., 1.),
      constant: 1.,
      linear: 0.09,
      quadratic: 0.032,
    };

    let spotlight = SpotLight {
      position: glm::zero(),
      direction: glm::zero(),
      inner_cut_off: 12.5_f32.to_radians().cos(),
      outer_cut_off: 17.5_f32.to_radians().cos(),
      ambient: glm::zero(),
      diffuse: glm::vec3(0.5, 0.5, 0.5),
      specular: glm::vec3(1., 1., 1.),
      constant: 1.,
      linear: 0.09,
      quadratic: 0.032,
    };

    let sun = DirLight {
      direction: glm::vec3(-0.2, -1.0, -0.3),
      ambient: glm::vec3(0.05, 0.05, 0.05),
      diffuse: glm::vec3(0.4, 0.4, 0.4),
      specular: glm::vec3(0.5, 0.5, 0.5),
    };

    Ok(Scene {
      pointlight_base,
      spotlight,
      sun,
      lighting_shader,
      model,
    })
  }

  pub fn update(&mut self, _elapsed: f32, camera: &Camera) {
    self.spotlight.position = camera.pos;
    self.spotlight.direction = camera.front();
  }

  pub unsafe fn draw(&self, gl: &Context, camera: &Camera) {
    // Draw cube to be lit
    let point_light_positions = [
      glm::vec3(0.7, 0.2, 2.0),
      glm::vec3(2.3, -3.3, -4.0),
      glm::vec3(-4.0, 2.0, -12.0),
      glm::vec3(0.0, 0.0, -3.0),
    ];
    let mut ctx = self.lighting_shader.activate(gl);
    self
      .lighting_shader
      .bind_uniform(&gl, "dir_lights", &vec![&self.sun], &mut ctx);
    self
      .lighting_shader
      .bind_uniform(&gl, "spot_lights", &vec![&self.spotlight], &mut ctx);
    self.lighting_shader.bind_uniform(
      &gl,
      "point_lights",
      &point_light_positions
        .iter()
        .cloned()
        .map(|position| PointLight {
          position,
          ..self.pointlight_base
        })
        .collect::<Vec<_>>(),
      &mut ctx,
    );
    self
      .lighting_shader
      .bind_uniform(gl, "camera", camera, &mut ctx);
    self.lighting_shader.bind_uniform(
      gl,
      "model",
      &glm::translation(&glm::vec3(0., 0., 0.)),
      &mut ctx,
    );
    self.model.draw(gl, &self.lighting_shader);
  }
}
