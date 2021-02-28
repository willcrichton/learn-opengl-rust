use crate::{
  io,
  material::Material,
  mesh::{Mesh, Vertex},
  prelude::*,
  shader::ActiveShader,
  texture::TextureBuilder,
};
use futures::future::try_join_all;
use image::DynamicImage;
use std::{cell::RefCell, collections::HashMap, io::BufReader, path::Path};

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
    enum BytesOrImage {
      Bytes(Vec<u8>),
      Image(DynamicImage),
    }
    let files = try_join_all(dir_files.split("\n").map(|file| async move {
      let path = obj_dir.join(file);
      let data = match path.extension().map(|ext| ext.to_str().unwrap()) {
        Some("jpeg") | Some("jpg") | Some("png") => {
          BytesOrImage::Image(io::load_image(path).await?)
        }
        _ => BytesOrImage::Bytes(io::load_file(path).await?),
      };
      Ok::<_, Error>((file.to_owned(), data))
    }))
    .await?;
    let file_map = RefCell::new(files.into_iter().collect::<HashMap<_, _>>());

    // Get model name as the directory name
    let model_name = obj_dir
      .file_name()
      .context("file name")?
      .to_str()
      .context("file name unicode")?
      .to_owned();

    // Read obj and mtl files
    let obj_file = if let BytesOrImage::Bytes(bytes) = file_map
      .borrow_mut()
      .remove(&(model_name + ".obj"))
      .context("obj file missing")?
    {
      bytes
    } else {
      bail!("obj file is an image")
    };
    let mut obj_reader = BufReader::new(obj_file.as_slice());
    let (obj_models, obj_materials) = tobj::load_obj_buf(&mut obj_reader, true, |mtl_path| {
      match file_map.borrow_mut().remove(mtl_path.to_str().unwrap()) {
        Some(BytesOrImage::Bytes(bytes)) => {
          tobj::load_mtl_buf(&mut BufReader::new(bytes.as_slice()))
        }
        _ => Err(tobj::LoadError::OpenFileFailed),
      }
    })?;

    let load_texture = |path: &str| {
      let image = if let BytesOrImage::Image(image) = file_map
        .borrow_mut()
        .remove(path)
        .context("texture missing")?
      {
        image
      } else {
        bail!("texture file is not an image")
      };
      TextureBuilder::new(gl).with_flip(false).build(image)
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
