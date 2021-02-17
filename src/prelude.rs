pub use anyhow::{Context as AnyhowContext, Error, Result};
pub use glm::{Mat4, Vec3};
pub use glow::{Context, HasContext};
pub use nalgebra_glm as glm;

pub type GlShader = <Context as HasContext>::Shader;
pub type GlProgram = <Context as HasContext>::Program;
pub type GlVertexArray = <Context as HasContext>::VertexArray;
pub type GlUniformLocation = <Context as HasContext>::UniformLocation;
pub type GlTexture = <Context as HasContext>::Texture;
