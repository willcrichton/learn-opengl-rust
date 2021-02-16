use crate::{shader::Shader, texture::Texture, window::Window};
use anyhow::Error;
use glm::Vec3;
use glow::{Context, HasContext};
use image::ImageFormat;
use instant::Instant;
use nalgebra_glm::{self as glm};
use shader::SetUniform;
use std::mem::size_of;
use winit::{
  dpi,
  event::{Event, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

mod io;
mod shader;
mod texture;
mod window;

unsafe fn build_geometry(
  gl: &Context,
) -> Result<(<Context as HasContext>::VertexArray, Vec<Vec3>), String> {
  // Initialize scene geometry as Rust values
  #[rustfmt::skip]
  let vertices = [
    -0.5, -0.5, -0.5,  0.0, 0.0,
     0.5, -0.5, -0.5,  1.0, 0.0,
     0.5,  0.5, -0.5,  1.0, 1.0,
     0.5,  0.5, -0.5,  1.0, 1.0,
    -0.5,  0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 0.0,

    -0.5, -0.5,  0.5,  0.0, 0.0,
     0.5, -0.5,  0.5,  1.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 1.0,
     0.5,  0.5,  0.5,  1.0, 1.0,
    -0.5,  0.5,  0.5,  0.0, 1.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,

    -0.5,  0.5,  0.5,  1.0, 0.0,
    -0.5,  0.5, -0.5,  1.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,
    -0.5,  0.5,  0.5,  1.0, 0.0,

     0.5,  0.5,  0.5,  1.0, 0.0,
     0.5,  0.5, -0.5,  1.0, 1.0,
     0.5, -0.5, -0.5,  0.0, 1.0,
     0.5, -0.5, -0.5,  0.0, 1.0,
     0.5, -0.5,  0.5,  0.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 0.0,

    -0.5, -0.5, -0.5,  0.0, 1.0,
     0.5, -0.5, -0.5,  1.0, 1.0,
     0.5, -0.5,  0.5,  1.0, 0.0,
     0.5, -0.5,  0.5,  1.0, 0.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,

    -0.5,  0.5, -0.5,  0.0, 1.0,
     0.5,  0.5, -0.5,  1.0, 1.0,
     0.5,  0.5,  0.5,  1.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 0.0,
    -0.5,  0.5,  0.5,  0.0, 0.0,
    -0.5,  0.5, -0.5,  0.0, 1.0f32
  ];
  let cube_positions = vec![
    glm::vec3(0.0f32, 0.0, 0.0),
    glm::vec3(2.0, 5.0, -15.0),
    glm::vec3(-1.5, -2.2, -2.5),
    glm::vec3(-3.8, -2.0, -12.3),
    glm::vec3(2.4, -0.4, -3.5),
    glm::vec3(-1.7, 3.0, -7.5),
    glm::vec3(1.3, -2.0, -2.5),
    glm::vec3(1.5, 2.0, -2.5),
    glm::vec3(1.5, 0.2, -1.5),
    glm::vec3(-1.3, 1.0, -1.5),
  ];

  // Create a Vertex Array that will reference the vertex and index buffers
  let vao = gl.create_vertex_array()?;
  gl.bind_vertex_array(Some(vao));

  // Create a vertex buffer to contain the 3-D coords of vertices
  let vbo = gl.create_buffer()?;
  gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

  // Convert f32 into a u8 slice and pass to GL
  let (_, vertices_bytes, _) = vertices.align_to::<u8>();
  gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_bytes, glow::STATIC_DRAW);

  // Make the vertex positions the first argument to the vertex shader
  let stride = 5 * size_of::<f32>() as i32;
  gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
  gl.enable_vertex_attrib_array(0);

  // and then the vertex texture coordinates
  gl.vertex_attrib_pointer_f32(
    1,
    2,
    glow::FLOAT,
    false,
    stride,
    3 * size_of::<f32>() as i32,
  );
  gl.enable_vertex_attrib_array(1);

  return Ok((vao, cube_positions));
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

async fn run() -> anyhow::Result<()> {
  // Asynchronously load assets (TODO: launch all futures before awaiting)
  let mut texture1 = Texture::load("assets/textures/container.jpg", ImageFormat::Jpeg).await?;
  let mut texture2 = Texture::load("assets/textures/awesomeface.png", ImageFormat::Png).await?;

  let platform = if cfg!(target_arch = "wasm32") {
    "web"
  } else {
    "native"
  };
  let vertex_source = io::load_shader(format!("assets/shaders/{}/simple.vert", platform)).await?;
  let fragment_source = io::load_shader(format!("assets/shaders/{}/simple.frag", platform)).await?;

  unsafe {
    // Set generic window parameters
    let wb = WindowBuilder::new()
      .with_title("LearnOpenGL")
      .with_inner_size(dpi::LogicalSize::new(1024., 768.));
    let event_loop = EventLoop::new();

    // Build platform-specific window and OpenGL context
    let (window, gl) = Window::build(wb, &event_loop);

    // Set OpenGL viewport size to window size
    // (note: this seems to only be necessary on web when logical size != physical size)
    let window_size = window.winit().inner_size();
    let (width, height) = (window_size.width, window_size.height);
    gl.viewport(0, 0, width as i32, height as i32);

    // Build scene and render pipeline components
    let (vao, cube_positions) = build_geometry(&gl).map_err(Error::msg)?;
    let shader_program = Shader::new(&gl, &vertex_source, &fragment_source);

    texture1.build_texture(&gl)?;
    texture2.build_texture(&gl)?;

    shader_program.activate(&gl);
    shader_program.set_uniform(&gl, "texture1", 0i32);
    shader_program.set_uniform(&gl, "texture2", 1i32);

    // Set camera parameters
    let view = glm::translation::<f32>(&glm::vec3(0., 0., -3.));
    let projection = glm::perspective(
      width as f32 / height as f32,
      (45f32).to_radians(),
      0.1,
      100.,
    );

    // Enable z-culling
    gl.enable(glow::DEPTH_TEST);

    let start = Instant::now();
    run_event_loop(gl, event_loop, window, move |gl| {
      let _dt = start.elapsed().as_millis() as f32 / 1000.;

      // Clear the screen with a default color
      gl.clear_color(0.2, 0.3, 0.3, 1.0);
      gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

      // Setup shaders
      shader_program.activate(&gl);

      texture1.bind(&gl, Some(glow::TEXTURE0));
      texture2.bind(&gl, Some(glow::TEXTURE1));

      shader_program.set_uniform(&gl, "view", view);
      shader_program.set_uniform(&gl, "projection", projection);

      // Setup geometry
      gl.bind_vertex_array(Some(vao));

      // Draw to the screen
      for (i, pos) in cube_positions.iter().enumerate() {
        let mut model = glm::translation::<f32>(pos);
        model = glm::rotate(
          &model,
          (20. * i as f32).to_radians(),
          &glm::vec3(1., 0.3, 0.5),
        );
        shader_program.set_uniform(&gl, "model", model);
        gl.draw_arrays(glow::TRIANGLES, 0, 36);
      }
    });
  }

  Ok(())
}

fn main() {
  let future = async {
    if let Err(err) = run().await {
      panic!("{:?}", err);
    }
  };

  #[cfg(not(target_arch = "wasm32"))]
  {
    // Use tokio to run our async functions. This builder is basically the same as
    // using #[tokio::main]
    tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap()
      .block_on(future);
  }

  #[cfg(target_arch = "wasm32")]
  {
    // Print out debug information to the console on panic
    console_error_panic_hook::set_once();

    // Tokio doesn't work on wasm yet, so we use the wasm_bindgen_futures crate to
    // run async code
    wasm_bindgen_futures::spawn_local(future);
  }
}
