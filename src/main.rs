use crate::{shader::{Shader, SetUniform}, window::Window};
use glow::{Context, HasContext};
use instant::Instant;
use std::mem::size_of;
use winit::{
  dpi,
  event::{Event, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

mod shader;
mod window;

unsafe fn build_geometry(gl: &Context) -> <Context as HasContext>::VertexArray {
  // Initialize scene geometry as Rust values
  #[rustfmt::skip]
  let vertices: &[f32] = &[
     0.5,  0.5, 0.0, 1.0, 0.0, 0.0,
     0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
    -0.5, -0.5, 0.0, 0.0, 0.0, 1.0,
    -0.5,  0.5, 0.0, 1.0, 1.0, 0.0
  ];
  let indices: &[u32] = &[0, 1, 3, 1, 2, 3];

  // Create a Vertex Array that will reference the vertex and index buffers
  let vao = gl.create_vertex_array().unwrap();
  gl.bind_vertex_array(Some(vao));

  // Create a vertex buffer to contain the 3-D coords of vertices
  let vbo = gl.create_buffer().unwrap();
  gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

  // Convert f32 into a u8 slice and pass to GL
  let (_, vertices_bytes, _) = vertices.align_to::<u8>();
  gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_bytes, glow::STATIC_DRAW);

  // Make the vertex buffer the first argument to the vertex shader
  gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 6 * size_of::<f32>() as i32, 0);
  gl.enable_vertex_attrib_array(0);

  gl.vertex_attrib_pointer_f32(
    1,
    3,
    glow::FLOAT,
    false,
    6 * size_of::<f32>() as i32,
    3 * size_of::<f32>() as i32,
  );
  gl.enable_vertex_attrib_array(1);

  // Create an index buffer to contain the set of triangles
  let ebo = gl.create_buffer().unwrap();
  gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

  let (_, indices_bytes, _) = indices.align_to::<u8>();
  gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_bytes, glow::STATIC_DRAW);

  return vao;
}

unsafe fn run_event_loop(
  gl: Context,
  event_loop: EventLoop<()>,
  window: Window,
  draw: impl Fn(&Context) + 'static,
) {
  // Event loop
  event_loop.run(move |event, _, control_flow| {
    // Poll means the loop will return continually to check for events rather than listening to
    // a cvar or something
    *control_flow = ControlFlow::Poll;

    match event {
      Event::LoopDestroyed => {
        return;
      }

      // Draw to the screen when requested
      Event::RedrawRequested(_) => {
        draw(&gl);
        window.swap_buffers();

        // We're drawing in a tight loop so immediately request redraw after drawing
        window.winit().request_redraw();
      }

      Event::WindowEvent { ref event, .. } => match event {
        // Resize OpenGL viewport when window is resized
        WindowEvent::Resized(size) => {
          gl.viewport(0, 0, size.width as i32, size.height as i32);
        }

        // Exit loop when CloseRequested raised
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

        WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
          // Exit loop when Escape is pressed
          Some(VirtualKeyCode::Escape) => {
            *control_flow = ControlFlow::Exit;
          }
          _ => {}
        },
        _ => (),
      },
      _ => (),
    }
  });
}

fn main() {
  #[cfg(target_arch = "wasm32")]
  console_error_panic_hook::set_once();

  unsafe {
    let wb = WindowBuilder::new()
      .with_title("LearnOpenGL")
      .with_inner_size(dpi::LogicalSize::new(1024., 768.));
    let event_loop = EventLoop::new();

    // Build platform-specific window and OpenGL context
    let (window, gl) = Window::build(wb, &event_loop);

    // Set OpenGL viewport size to window size
    // (note: this seems to only be necessary on web when logical size != physical size)
    let window_size = window.winit().inner_size();
    gl.viewport(0, 0, window_size.width as i32, window_size.height as i32);

    // Build scene and render pipeline components
    let vao = build_geometry(&gl);

    let shader_program = {
      #[cfg(not(target_arch = "wasm32"))]
      let (vertex_source, fragment_source) = (
        include_bytes!("shaders/native/simple.vert"),
        include_bytes!("shaders/native/simple.frag"),
      );

      #[cfg(target_arch = "wasm32")]
      let (vertex_source, fragment_source) = (
        include_bytes!("shaders/web/simple.vert"),
        include_bytes!("shaders/web/simple.frag"),
      );

      Shader::new(&gl, vertex_source, fragment_source)
    };

    let start = Instant::now();

    run_event_loop(gl, event_loop, window, move |gl| {
      // Clear the screen with a default color
      gl.clear_color(0.2, 0.3, 0.3, 1.0);
      gl.clear(glow::COLOR_BUFFER_BIT);

      let elapsed = (start.elapsed().as_millis() as f32) / 1000.;
      let green_value = elapsed.sin() / 2. + 0.5;
      shader_program.set_uniform(&gl, "ourColorUniform", [0., green_value, 0., 1.]);

      // Bind geometry and shaders
      shader_program.activate(&gl);
      gl.bind_vertex_array(Some(vao));

      // Draw to the screen
      gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
    });
  }
}
