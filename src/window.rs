use glow::*;
use winit::{
  event_loop::EventLoop,
  window::{self as winit_window, WindowBuilder},
};

#[cfg(not(target_arch = "wasm32"))]
mod platform {
  use super::*;

  pub struct Window(glutin::ContextWrapper<glutin::PossiblyCurrent, winit_window::Window>);

  impl Window {
    // Use glutin to get an OpenGL context from winit via glutin::ContextWrapper
    pub fn build(wb: WindowBuilder, event_loop: &EventLoop<()>) -> (Self, Context) {
      unsafe {
        let windowed_context = glutin::ContextBuilder::new()
          .with_vsync(true)
          .build_windowed(wb, event_loop)
          .unwrap();
        let windowed_context = windowed_context.make_current().unwrap();
        let gl =
          glow::Context::from_loader_function(|s| windowed_context.get_proc_address(s) as *const _);

        (Window(windowed_context), gl)
      }
    }

    pub fn winit(&self) -> &winit_window::Window {
      self.0.window()
    }

    pub fn swap_buffers(&self) {
      self.0.swap_buffers().unwrap();
    }
  }
}

#[cfg(target_arch = "wasm32")]
mod platform {
  use super::*;
  use std::collections::HashMap;
  use wasm_bindgen::{JsCast, JsValue};
  use winit::platform::web::WindowBuilderExtWebSys;

  pub struct Window(winit_window::Window);

  impl Window {
    // Use winit::WindowBuilder::with_canvas and glow::Context::from_webgl2_context
    // to set up the web window
    pub fn build(wb: WindowBuilder, event_loop: &EventLoop<()>) -> (Self, Context) {
      let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("learn-opengl")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

      // Have to explicitly ask for a stencil buffer to be allocated, see:
      // https://www.khronos.org/registry/webgl/specs/1.0/#WEBGLCONTEXTATTRIBUTES
      let mut context_options = HashMap::new();
      context_options.insert("stencil", true);
      let context_options_js = JsValue::from_serde(&context_options).unwrap();

      let webgl2_context = canvas
        .get_context_with_context_options("webgl2", &context_options_js)
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .unwrap();

      let window = wb.with_canvas(Some(canvas)).build(event_loop).unwrap();
      let gl = Context::from_webgl2_context(webgl2_context);
      (Window(window), gl)
    }

    pub fn winit(&self) -> &winit_window::Window {
      &self.0
    }

    pub fn swap_buffers(&self) {}
  }
}

pub use platform::Window;
