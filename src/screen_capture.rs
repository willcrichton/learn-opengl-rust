use std::u32;

use crate::{
  geometry::Geometry,
  mesh::Mesh,
  prelude::*,
  shader::{ActiveShader, Shader},
  texture::{Texture, TextureBuilder},
};

struct Framebuffer {
  fbo: GlFramebuffer,
  render_texture: Texture,
}

impl Framebuffer {
  pub unsafe fn new(gl: &Context, width: u32, height: u32) -> Result<Self> {
    // Framebuffer contains another render target (color/depth/stencil buffers + texture)
    let fbo = gl.create_framebuffer().map_err(Error::msg)?;
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

    // Render texture is a 2D image that contains output of rendering
    let render_texture = TextureBuilder::new(gl)
      .with_format(glow::RGB)
      .with_tex_parameter(glow::TEXTURE_MIN_FILTER, glow::LINEAR)
      .with_tex_parameter(glow::TEXTURE_MAG_FILTER, glow::LINEAR)
      .render_texture(width, height)?;
    gl.framebuffer_texture_2d(
      glow::FRAMEBUFFER,
      glow::COLOR_ATTACHMENT0,
      glow::TEXTURE_2D,
      Some(render_texture.texture),
      0,
    );

    // We don't read from depth or stencil buffers so use a renderbuffer
    let rbo = gl.create_renderbuffer().map_err(Error::msg)?;
    gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
    gl.renderbuffer_storage(
      glow::RENDERBUFFER,
      glow::DEPTH24_STENCIL8,
      width as i32,
      height as i32,
    );
    gl.bind_renderbuffer(glow::RENDERBUFFER, None);
    gl.framebuffer_renderbuffer(
      glow::FRAMEBUFFER,
      glow::DEPTH_STENCIL_ATTACHMENT,
      glow::RENDERBUFFER,
      Some(rbo),
    );

    // Fail if framebuffer isn't complete
    if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
      bail!("Framebuffer is not complete");
    }

    gl.bind_framebuffer(glow::FRAMEBUFFER, None);

    Ok(Framebuffer {
      fbo,
      render_texture,
    })
  }
}

pub struct ScreenCapture {
  framebuffer: Framebuffer,
  screen_shader: Shader,
  screen_geom: Mesh,
}

impl ScreenCapture {
  pub async unsafe fn new(gl: &Context, width: u32, height: u32) -> Result<Self> {
    let framebuffer = Framebuffer::new(&gl, width, height)?;

    let screen_geom = Geometry::Plane {
      length: 2.,
      width: 2.,
      normal: glm::zero(),
    }
    .to_mesh(&gl, None)?;

    let screen_shader = Shader::load(
      &gl,
      "assets/shaders/screen.vert",
      "assets/shaders/screen.frag",
      None,
    )
    .await?;

    Ok(ScreenCapture {
      screen_shader,
      screen_geom,
      framebuffer,
    })
  }

  pub unsafe fn record(&self, gl: &Context) {
    // Record subsequent draw calls into the framebuffer by binding it
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer.fbo));
  }

  pub unsafe fn replay(&self, gl: &Context, init_shader: impl Fn(&Context, &mut ActiveShader)) {
    // Unbind the framebuffer and then draw the render texture onto the screen
    gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    gl.clear_color(1., 1., 1., 1.);
    gl.clear(glow::COLOR_BUFFER_BIT);

    let mut shader = self.screen_shader.activate(&gl);
    gl.disable(glow::DEPTH_TEST);
    shader.bind_uniform(gl, "screenTexture", &self.framebuffer.render_texture);
    init_shader(gl, &mut shader);
    self.screen_geom.draw(&gl, &mut shader);
    gl.enable(glow::DEPTH_TEST);
  }
}
