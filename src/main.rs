#![allow(dead_code)]

use crate::{camera::Camera, prelude::*, scene::Scene, user_inputs::UserInputs, window::Window};
use instant::Instant;
#[cfg(target_arch = "wasm32")]
use winit::event::{ElementState, MouseButton};
use winit::{
  dpi,
  event::{Event, VirtualKeyCode as Key, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

mod camera;
mod io;
mod light;
mod material;
mod prelude;
mod scene;
mod shader;
mod texture;
mod user_inputs;
mod window;

struct State {
  scene: Scene,
  camera: Camera,
  user_inputs: UserInputs,

  start: Instant,
  last_tick: Instant,
}

impl State {
  pub fn elapsed(&self) -> f32 {
    self.start.elapsed().as_millis() as f32 / 1000.
  }

  pub fn dt(&self) -> f32 {
    self.last_tick.elapsed().as_millis() as f32 / 1000.
  }
}

fn lock_cursor(window: &Window) {
  window.winit().set_cursor_visible(false);
  window.winit().set_cursor_grab(true).unwrap();
}

unsafe fn run_event_loop(
  gl: Context,
  event_loop: EventLoop<()>,
  window: Window,
  mut state: State,
  draw: impl Fn(&Context, &State) + 'static,
  update: impl Fn(&mut State, Event<()>) + 'static,
) {
  #[cfg(target_arch = "wasm32")]
  let mut cursor_locked = false;

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
        draw(&gl, &state);
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

        WindowEvent::KeyboardInput { input, .. } => {
          if let Some(keycode) = input.virtual_keycode {
            match keycode {
              // Exit loop when Escape is pressed
              Key::Escape => {
                *control_flow = ControlFlow::Exit;
              }
              _ => {}
            }
          }
        }

        // Browsers only let you lock the mouse after a user interaction, like clicking
        #[cfg(target_arch = "wasm32")]
        WindowEvent::MouseInput {
          button: MouseButton::Left,
          state: ElementState::Pressed,
          ..
        } => {
          if !cursor_locked {
            lock_cursor(&window);
            cursor_locked = true;
          }
        }

        _ => (),
      },
      _ => (),
    };

    update(&mut state, event);
  });
}

async fn run() -> anyhow::Result<()> {
  unsafe {
    // Set generic window parameters
    let wb = WindowBuilder::new()
      .with_title("LearnOpenGL")
      .with_inner_size(dpi::LogicalSize::new(1024., 768.));
    let event_loop = EventLoop::new();

    // Build platform-specific window and OpenGL context
    let (window, gl) = Window::build(wb, &event_loop);

    // Native platforms let you immediately lock the mouse
    #[cfg(not(target_arch = "wasm32"))]
    lock_cursor(&window);

    // Set OpenGL viewport size to window size
    // (note: this seems to only be necessary on web when logical size != physical size)
    let window_size = window.winit().inner_size();
    let (width, height) = (window_size.width, window_size.height);
    gl.viewport(0, 0, width as i32, height as i32);

    // Build scene and render pipeline components
    let scene = Scene::build(&gl).await?;

    // Set camera parameters
    let camera = Camera::new(
      glm::vec3(0.5, 1.5, 5.),
      glm::perspective(
        width as f32 / height as f32,
        (45f32).to_radians(),
        0.1,
        100.,
      ),
      glm::zero(),
    );

    // Enable z-culling
    gl.enable(glow::DEPTH_TEST);

    // Build monotlithic state object
    let state = State {
      camera,
      scene,
      user_inputs: UserInputs::default(),
      start: Instant::now(),
      last_tick: Instant::now(),
    };

    let draw = move |gl: &Context, state: &State| {
      // Clear the screen with a default color
      gl.clear_color(0.1, 0.1, 0.1, 1.0);
      gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

      // Draw the scene
      state.scene.draw(&gl, &state.camera);
    };

    let update = move |state: &mut State, event: Event<()>| {
      state.user_inputs.update(&event);
      state.camera.update(state.dt(), &state.user_inputs);
      state.scene.update(state.elapsed());
      state.last_tick = Instant::now();
    };

    run_event_loop(gl, event_loop, window, state, draw, update);
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
