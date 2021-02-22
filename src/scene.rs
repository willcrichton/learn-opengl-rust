use crate::{
  camera::Camera,
  geometry::Geometry,
  light::{DirLight, PointLight, SpotLight},
  material::Material,
  model::Model,
  prelude::*,
  shader::{ActiveShader, Shader},
  texture::Texture,
};

struct Entity {
  model: Model,
  transform: Mat4,
}

impl Entity {
  unsafe fn draw(&self, gl: &Context, shader: &mut ActiveShader) {
    shader.bind_uniform(gl, "model", &self.transform);
    self.model.draw(gl, shader);
  }
}

pub struct Scene {
  entities: Vec<Entity>,

  point_lights: Vec<PointLight>,
  spot_lights: Vec<SpotLight>,
  dir_lights: Vec<DirLight>,

  shader: Shader,
}

impl Scene {
  pub async unsafe fn build(gl: &Context) -> Result<Self> {
    // Load all the assets
    let (shader, metal_texture, marble_texture) = try_join!(
      Shader::load(
        gl,
        "assets/shaders/colors.vert",
        "assets/shaders/depth.frag"
      ),
      Texture::load(gl, "assets/textures/metal.png", true),
      Texture::load(gl, "assets/textures/marble.jpg", true)
    )?;

    let plane_model = Geometry::Plane {
      length: 10.,
      width: 10.,
      normal: glm::vec3(0., 1., 0.),
    }
    .to_mesh(
      gl,
      Some(Material {
        diffuse: metal_texture.clone(),
        specular: metal_texture,
        shininess: 16.,
      }),
    )?
    .to_model();
    let plane = Entity {
      model: plane_model,
      transform: glm::translation(&glm::vec3(0., -0.5, 0.)),
    };

    let box_model = Geometry::Cube {
      length: 1.,
      width: 1.,
      height: 1.,
    }
    .to_mesh(
      gl,
      Some(Material {
        diffuse: marble_texture.clone(),
        specular: marble_texture,
        shininess: 16.,
      }),
    )?
    .to_model();

    let cube1 = Entity {
      model: box_model.clone(),
      transform: glm::translation(&glm::vec3(-1., 0., -1.)),
    };
    let cube2 = Entity {
      model: box_model,
      transform: glm::translation(&glm::vec3(2., 0., 0.)),
    };

    let sun = DirLight {
      direction: glm::vec3(1., -1., 0.),
      ambient: glm::vec3(0.2, 0.2, 0.2),
      diffuse: glm::vec3(0.5, 0.5, 0.5),
      specular: glm::vec3(1., 1., 1.),
    };

    Ok(Scene {
      entities: vec![plane, cube1, cube2],
      point_lights: vec![],
      spot_lights: vec![],
      dir_lights: vec![sun],
      shader,
    })
  }

  pub fn update(&mut self, _elapsed: f32, _camera: &Camera) {}

  pub unsafe fn draw(&self, gl: &Context, camera: &Camera) {
    let mut shader = self.shader.activate(gl);
    shader.bind_uniform(&gl, "dir_lights", &self.dir_lights);
    shader.bind_uniform(&gl, "spot_lights", &self.spot_lights);
    shader.bind_uniform(&gl, "point_lights", &self.point_lights);
    shader.bind_uniform(gl, "camera", camera);

    for entity in &self.entities {
      entity.draw(gl, &mut shader);
    }
  }
}
