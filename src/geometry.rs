use crate::{
  material::Material,
  mesh::{Mesh, Vertex},
  prelude::*,
};

pub enum Geometry {
  Cube {
    length: f32,
    width: f32,
    height: f32,
  },
  Plane {
    length: f32,
    width: f32,
    normal: Vec3,
  },
}

impl Geometry {
  pub fn to_vertices_indices(&self) -> (Vec<Vertex>, Vec<u32>) {
    match *self {
      Geometry::Cube {
        length,
        width,
        height,
      } => {
        #[rustfmt::skip]
        let mut vertices = Vertex::from_flat_array(&[ // Back face
          -0.5, -0.5, -0.5,  0.0, 0.0, -1.0, 0.0, 0.0, // Bottom-left
           0.5,  0.5, -0.5,  0.0, 0.0, -1.0, 1.0, 1.0, // top-right
           0.5, -0.5, -0.5,  0.0, 0.0, -1.0, 1.0, 0.0, // bottom-right         
           0.5,  0.5, -0.5,  0.0, 0.0, -1.0, 1.0, 1.0, // top-right
          -0.5, -0.5, -0.5,  0.0, 0.0, -1.0, 0.0, 0.0, // bottom-left
          -0.5,  0.5, -0.5,  0.0, 0.0, -1.0, 0.0, 1.0, // top-left
          // Front face
          -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,  0.0, 0.0, // bottom-left
           0.5, -0.5,  0.5,  0.0,  0.0, 1.0,  1.0, 0.0, // bottom-right
           0.5,  0.5,  0.5,  0.0,  0.0, 1.0,  1.0, 1.0, // top-right
           0.5,  0.5,  0.5,  0.0,  0.0, 1.0,  1.0, 1.0, // top-right
          -0.5,  0.5,  0.5,  0.0,  0.0, 1.0,  0.0, 1.0, // top-left
          -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,  0.0, 0.0, // bottom-left
          // Left face
          -0.5,  0.5,  0.5,  -1.0, 0.0, 0.0,  1.0, 0.0, // top-right
          -0.5,  0.5, -0.5,  -1.0, 0.0, 0.0,  1.0, 1.0, // top-left
          -0.5, -0.5, -0.5,  -1.0, 0.0, 0.0,  0.0, 1.0, // bottom-left
          -0.5, -0.5, -0.5,  -1.0, 0.0, 0.0,  0.0, 1.0, // bottom-left
          -0.5, -0.5,  0.5,  -1.0, 0.0, 0.0,  0.0, 0.0, // bottom-right
          -0.5,  0.5,  0.5,  -1.0, 0.0, 0.0,  1.0, 0.0, // top-right
          // Right face
           0.5,  0.5,  0.5,  1.0, 0.0, 0.0,  1.0, 0.0, // top-left
           0.5, -0.5, -0.5,  1.0, 0.0, 0.0,  0.0, 1.0, // bottom-right
           0.5,  0.5, -0.5,  1.0, 0.0, 0.0,  1.0, 1.0, // top-right         
           0.5, -0.5, -0.5,  1.0, 0.0, 0.0,  0.0, 1.0, // bottom-right
           0.5,  0.5,  0.5,  1.0, 0.0, 0.0,  1.0, 0.0, // top-left
           0.5, -0.5,  0.5,  1.0, 0.0, 0.0,  0.0, 0.0, // bottom-left     
          // Bottom face
          -0.5, -0.5, -0.5,  0.0, -1.0, 0.0,  0.0, 1.0, // top-right
           0.5, -0.5, -0.5,  0.0, -1.0, 0.0,  1.0, 1.0, // top-left
           0.5, -0.5,  0.5,  0.0, -1.0, 0.0,  1.0, 0.0, // bottom-left
           0.5, -0.5,  0.5,  0.0, -1.0, 0.0,  1.0, 0.0, // bottom-left
          -0.5, -0.5,  0.5,  0.0, -1.0, 0.0,  0.0, 0.0, // bottom-right
          -0.5, -0.5, -0.5,  0.0, -1.0, 0.0,  0.0, 1.0, // top-right
          // Top face
          -0.5,  0.5, -0.5,  0.0, 1.0, 0.0,  0.0, 1.0, // top-left
           0.5,  0.5,  0.5,  0.0, 1.0, 0.0,  1.0, 0.0, // bottom-right
           0.5,  0.5, -0.5,  0.0, 1.0, 0.0,  1.0, 1.0, // top-right     
           0.5,  0.5,  0.5,  0.0, 1.0, 0.0,  1.0, 0.0, // bottom-right
          -0.5,  0.5, -0.5,  0.0, 1.0, 0.0,  0.0, 1.0, // top-left
          -0.5,  0.5,  0.5,  0.0, 1.0, 0.0,  0.0, 0.0  // bottom-left        
        ]);
        for vertex in vertices.iter_mut() {
          vertex.position = vertex
            .position
            .component_mul(&glm::vec3(width, height, length));
        }

        let indices = (0..(vertices.len() as u32)).collect::<Vec<_>>();

        (vertices, indices)
      }

      Geometry::Plane {
        length,
        width,
        normal,
      } => {
        let vertices = vec![-1., 1.]
          .into_iter()
          .map(move |i| {
            vec![-1., 1.].into_iter().map(move |j| Vertex {
              position: glm::vec3(length * i / 2., 0., width * j / 2.),
              normal: normal.clone(),
              tex_coords: glm::vec2(i / 2. + 0.5, j / 2. + 0.5),
            })
          })
          .flatten()
          .collect();
        let indices = vec![0, 1, 2, 1, 3, 2, 0, 2, 1, 1, 2, 3];
        (vertices, indices)
      }
    }
  }

  pub unsafe fn to_mesh(&self, gl: &Context, material: Option<Material>) -> Result<Mesh> {
    let (vertices, indices) = self.to_vertices_indices();
    Mesh::new(gl, vertices, indices, material)
  }
}
