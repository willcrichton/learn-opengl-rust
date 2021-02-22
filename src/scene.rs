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
  floor: Entity,
  cubes: Vec<Entity>,

  point_lights: Vec<PointLight>,
  spot_lights: Vec<SpotLight>,
  dir_lights: Vec<DirLight>,

  light_shader: Shader,
  border_shader: Shader
}

impl Scene {
  pub async unsafe fn build(gl: &Context) -> Result<Self> {
    // Load all the assets
    let (light_shader, border_shader, metal_texture, marble_texture) = try_join!(
      Shader::load(
        gl,
        "assets/shaders/colors.vert",
        "assets/shaders/colors.frag"
      ),
      Shader::load(
        gl,
        "assets/shaders/border.vert",
        "assets/shaders/border.frag"
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
      floor: plane,
      cubes: vec![cube1, cube2],
      point_lights: vec![],
      spot_lights: vec![],
      dir_lights: vec![sun],
      light_shader,
      border_shader
    })
  }

  pub fn update(&mut self, _elapsed: f32, _camera: &Camera) {}

  pub unsafe fn draw(&self, gl: &Context, camera: &Camera) {
    let mut shader = self.light_shader.activate(gl);
    shader.bind_uniform(&gl, "dir_lights", &self.dir_lights);
    shader.bind_uniform(&gl, "spot_lights", &self.spot_lights);
    shader.bind_uniform(&gl, "point_lights", &self.point_lights);
    shader.bind_uniform(gl, "camera", camera);

    gl.stencil_mask(0x00);
    self.floor.draw(gl, &mut shader);

    gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
    gl.stencil_func(glow::ALWAYS, 1, 0xFF);
    gl.stencil_mask(0xFF);

    for cube in &self.cubes {
      cube.draw(gl, &mut shader);
    }

    gl.stencil_func(glow::NOTEQUAL, 1, 0xFF);
    gl.stencil_mask(0x00);
    gl.disable(glow::DEPTH_TEST);

    let mut shader = self.border_shader.activate(gl);
    shader.bind_uniform(gl, "camera", camera);
    for cube in &self.cubes {
      cube.draw(gl, &mut shader);
    }

    gl.stencil_mask(0xFF);
    gl.stencil_func(glow::ALWAYS, 1, 0xFF);
    gl.enable(glow::DEPTH_TEST);
  }
}
