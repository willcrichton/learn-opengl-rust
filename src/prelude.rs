pub use crate::shader::GlmStd140Ext;
pub use anyhow::{bail, Context as AnyhowContext, Error, Result};
pub use futures::try_join;
pub use glm::{Mat4, Vec2, Vec3, Vec4};
pub use glow::{Context, HasContext};
pub use macros::{BindUniform, ShaderBlockDef, ShaderTypeDef};
pub use maplit::hashmap;
pub use nalgebra as na;
pub use nalgebra_glm as glm;

pub type GlShader = <Context as HasContext>::Shader;
pub type GlProgram = <Context as HasContext>::Program;
pub type GlVertexArray = <Context as HasContext>::VertexArray;
pub type GlUniformLocation = <Context as HasContext>::UniformLocation;
pub type GlTexture = <Context as HasContext>::Texture;
pub type GlBuffer = <Context as HasContext>::Buffer;
pub type GlFramebuffer = <Context as HasContext>::Framebuffer;
