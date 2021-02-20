pub use anyhow::{Context as AnyhowContext, Error, Result};
pub use futures::try_join;
pub use glm::{Mat4, Vec2, Vec3};
pub use glow::{Context, HasContext};
pub use macros::{BindUniform, ShaderTypeDef};
pub use nalgebra_glm as glm;

pub type GlShader = <Context as HasContext>::Shader;
pub type GlProgram = <Context as HasContext>::Program;
pub type GlVertexArray = <Context as HasContext>::VertexArray;
pub type GlUniformLocation = <Context as HasContext>::UniformLocation;
pub type GlTexture = <Context as HasContext>::Texture;
pub type GlBuffer = <Context as HasContext>::Buffer;
