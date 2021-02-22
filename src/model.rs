use crate::{
  io,
  material::Material,
  mesh::{Mesh, Vertex},
  prelude::*,
  shader::ActiveShader,
  texture::TextureBuilder,
};
use futures::future::try_join_all;
use std::{collections::HashMap, io::BufReader, path::Path};

#[derive(Clone)]
pub struct Model {
  pub meshes: Vec<Mesh>,
}

// fn obj_to_mesh(model: tobj::Model, mtl: tobj::Material, files: &HashMap<String, Vec<u8>>) -> Mesh {
//   panic!()
// }

impl Model {
  pub async unsafe fn load(gl: &Context, obj_dir: impl AsRef<Path>) -> Result<Model> {
    // Get a listing of all files in the model directory
    // We have to use a special dir.txt because you can't list directories on the web
    let obj_dir = obj_dir.as_ref();
    let dir_files = io::load_string(obj_dir.join("dir.txt")).await?;

    // Load all files at once into a map from file name to bytes
    let files = try_join_all(dir_files.split("\n").map(|file| async move {
      let bytes = io::load_file(obj_dir.join(file)).await?;
      Ok::<_, Error>((file.to_owned(), bytes))
    }))
    .await?;
    let file_map = files.into_iter().collect::<HashMap<_, _>>();

    // Get model name as the directory name
    let model_name = obj_dir
      .file_name()
      .context("file name")?
      .to_str()
      .context("file name unicode")?
      .to_owned();

    // Read obj and mtl files
    let obj_bytes = BufReader::new(
      &**file_map
        .get(&(model_name + ".obj"))
        .context("obj file missing")?,
    );
    let mut obj_reader = BufReader::new(obj_bytes);
    let (obj_models, obj_materials) = tobj::load_obj_buf(&mut obj_reader, true, |mtl_path| {
      match file_map.get(mtl_path.to_str().unwrap()) {
        Some(bytes) => tobj::load_mtl_buf(&mut BufReader::new(&**bytes)),
        None => Err(tobj::LoadError::OpenFileFailed),
      }
    })?;

    let load_texture = |path: &str| {
      let bytes = file_map.get(path).context("texture missing")?;
      TextureBuilder::new(gl).with_flip(false).from_bytes(bytes)
    };

    let materials = obj_materials
      .into_iter()
      .map(|obj_material| {
        Ok(Material {
          diffuse: load_texture(&obj_material.diffuse_texture)?,
          specular: load_texture(&obj_material.specular_texture)?,
          shininess: obj_material.shininess,
        })
      })
      .collect::<Result<Vec<_>>>()?;

    let meshes = obj_models
      .into_iter()
      .map(|obj_model| {
        let mesh = obj_model.mesh;

        let positions = mesh
          .positions
          .chunks(3)
          .map(|c| glm::vec3(c[0], c[1], c[2]));
        let normals = mesh.normals.chunks(3).map(|c| glm::vec3(c[0], c[1], c[2]));
        let texcoords = mesh.texcoords.chunks(2).map(|c| glm::vec2(c[0], c[1]));
        let vertices = positions
          .zip(normals)
          .zip(texcoords)
          .map(|((position, normal), tex_coords)| Vertex {
            position,
            normal,
            tex_coords,
          })
          .collect();

        let indices = mesh.indices.to_vec();

        let material = mesh.material_id.map(|id| materials[id].clone());

        Mesh::new(gl, vertices, indices, material)
      })
      .collect::<Result<Vec<_>>>()?;

    Ok(Model { meshes })
  }

  pub unsafe fn draw(&self, gl: &Context, shader: &mut ActiveShader) {
    for mesh in &self.meshes {
      mesh.draw(gl, shader);
    }
  }
}
