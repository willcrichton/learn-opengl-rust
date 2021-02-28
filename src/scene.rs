use std::{collections::HashMap, path::Path};

use crate::{
  camera::{Camera, CameraBlock},
  geometry::Geometry,
  light::{DirLight, PointLight, SpotLight},
  material::Material,
  mesh::Mesh,
  model::Model,
  prelude::*,
  shader::{ActiveShader, Shader, UniformBlock},
  text::{Font, Text},
  texture::{TCubemap, Texture, TextureBuilder},
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
  grasses: Vec<Entity>,
  exploder: Entity,

  light_shader: Shader,
  point_lights: Vec<PointLight>,
  spot_lights: Vec<SpotLight>,
  dir_lights: Vec<DirLight>,

  text_shader: Shader,
  text: Text,
  fonts: HashMap<String, Font>,

  camera_ubo: UniformBlock<CameraBlock>,

  skybox_shader: Shader,
  skybox: Mesh,
  skybox_texture: Texture<TCubemap>,
}

impl Scene {
  pub async unsafe fn build(gl: &Context) -> Result<Self> {
    // Load all the assets
    let (
      light_shader,
      text_shader,
      skybox_shader,
      metal_texture,
      marble_texture,
      grass_texture,
      skybox_texture,
      font,
      backpack_model,
    ) = try_join!(
      Shader::load(
        gl,
        "assets/shaders/colors.vert",
        "assets/shaders/colors.frag",
        if cfg!(target_arch = "wasm32") {
          None
        } else {
          Some(Path::new("assets/shaders/explode.geom"))
        }
      ),
      Shader::load(
        gl,
        "assets/shaders/text.vert",
        "assets/shaders/text.frag",
        None
      ),
      Shader::load(
        gl,
        "assets/shaders/skybox.vert",
        "assets/shaders/skybox.frag",
        None
      ),
      TextureBuilder::new(gl).load("assets/textures/metal.png"),
      TextureBuilder::new(gl).load("assets/textures/marble.jpg"),
      TextureBuilder::new(gl)
        .with_tex_parameter(glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE)
        .with_tex_parameter(glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE)
        .load("assets/textures/blending_transparent_window.png"),
      TextureBuilder::new(gl).as_cubemap().load(
        vec!["right", "left", "top", "bottom", "front", "back"]
          .into_iter()
          .map(|path| format!("assets/cubemaps/skybox/{}.jpg", path))
          .collect::<Vec<_>>()
      ),
      Font::load(gl, "assets/fonts/DejaVuSans.ttf"),
      Model::load(gl, "assets/models/backpack")
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
      model: box_model.clone(),
      transform: glm::translation(&glm::vec3(2., 0., 0.)),
    };

    let grass_model = Geometry::Plane {
      length: 1.0,
      width: 1.0,
      normal: glm::vec3(0., 1., 0.),
    }
    .to_mesh(
      gl,
      Some(Material {
        diffuse: grass_texture.clone(),
        specular: grass_texture,
        shininess: 0.,
      }),
    )?
    .to_model();

    let grasses = vec![
      glm::vec3(-1., 0., -0.48),
      glm::vec3(2.0, 0.0, 0.51),
      glm::vec3(0.0, 0.0, 0.7),
      glm::vec3(-0.3, 0.0, -2.3),
      glm::vec3(0.5, 0.0, -0.6),
    ]
    .into_iter()
    .map(|pos| {
      let transform = glm::rotate(
        &glm::translation(&pos),
        -90_f32.to_radians(),
        &glm::vec3(1., 0., 0.),
      );
      Entity {
        model: grass_model.clone(),
        transform,
      }
    })
    .collect();

    let sun = DirLight {
      direction: glm::vec3(-1., -1., -1.),
      ambient: glm::vec3(0.2, 0.2, 0.2),
      diffuse: glm::vec3(0.8, 0.8, 0.8),
      specular: glm::vec3(1., 1., 1.),
    };

    let fonts = hashmap! {
      font.name.clone() => font
    };

    let text = if cfg!(target_arch = "wasm32") {
      "Geometry shader doesn't work on WebGL2 yet :("
    } else {
      "Press tab to cycle effects"
    };
    let text = Text::new(
      text,
      "DejaVuSans",
      48.,
      [1., 1., 1., 1.],
      glm::vec2(30., 30.),
    );

    let skybox = Geometry::Cube {
      width: 2.,
      length: 2.,
      height: 2.,
    }
    .to_mesh(gl, None)?;

    // Bind camera uniform block ahead of time to all shaders
    let camera_ubo = UniformBlock::new(gl, 0)?;
    light_shader
      .activate(gl)
      .bind_uniform(gl, "CameraBlock", &camera_ubo);
    skybox_shader
      .activate(gl)
      .bind_uniform(gl, "CameraBlock", &camera_ubo);

    let exploder = Entity {
      model: backpack_model,
      transform: glm::translation(&glm::vec3(1.5, 0., 1.5)),
    };

    Ok(Scene {
      floor: plane,
      cubes: vec![cube1, cube2],
      grasses,
      point_lights: vec![],
      spot_lights: vec![],
      dir_lights: vec![sun],
      text_shader,
      light_shader,
      skybox_shader,
      fonts,
      text,
      skybox,
      skybox_texture,
      camera_ubo,
      exploder,
    })
  }

  pub fn update(&mut self, _elapsed: f32, _camera: &Camera) {}

  pub unsafe fn draw(
    &mut self,
    gl: &Context,
    camera: &Camera,
    time: f32,
    screen_width: u32,
    screen_height: u32,
  ) -> Result<()> {
    // Update camera uniform block for all bound shaders
    self.camera_ubo.upload(gl, &camera.uniform_block());

    // Draw all lit objects
    let mut shader = self.light_shader.activate(gl);
    shader.bind_uniform(gl, "dir_lights", &self.dir_lights);
    shader.bind_uniform(gl, "spot_lights", &self.spot_lights);
    shader.bind_uniform(gl, "point_lights", &self.point_lights);
    shader.bind_uniform(gl, "time", &time);
    shader.bind_uniform(gl, "should_explode", &false);

    self.floor.draw(gl, &mut shader);

    for cube in &self.cubes {
      cube.draw(gl, &mut shader);
    }

    shader.bind_uniform(gl, "should_explode", &true);
    self.exploder.draw(gl, &mut shader);
    shader.bind_uniform(gl, "should_explode", &false);

    // Sort transparent objs in order of dist to camera so transparency works correctly
    let mut grasses = self.grasses.iter().collect::<Vec<_>>();
    grasses.sort_by_key(|grass| {
      let translation = grass.transform.column_part(3, 3);
      ordered_float::OrderedFloat(glm::length2(&(camera.pos - translation)))
    });
    for grass in grasses.into_iter().rev() {
      grass.draw(gl, &mut shader);
    }

    // Draw cubemap skybox
    let mut shader = self.skybox_shader.activate(gl);
    shader.bind_uniform(gl, "skybox", &self.skybox_texture);

    // Have to disable face culling because we're viewing the inside of a cube
    gl.disable(glow::CULL_FACE);
    self.skybox.draw(gl, &mut shader);
    gl.enable(glow::CULL_FACE);

    // Draw text, which queues draw commands on the individual fonts
    self.text.draw(&mut self.fonts);

    // Flush each fonts draw commands
    for font in self.fonts.values_mut() {
      font.draw(gl, &self.text_shader, screen_width, screen_height)?;
    }

    Ok(())
  }
}
