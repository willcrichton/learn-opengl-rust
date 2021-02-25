#![allow(dead_code)]

use crate::{camera::Camera, prelude::*, scene::Scene, user_inputs::UserInputs, window::Window};
use instant::Instant;
use screen_capture::ScreenCapture;
#[cfg(target_arch = "wasm32")]
use winit::event::{ElementState, MouseButton};
use winit::{
  dpi,
  event::{Event, VirtualKeyCode as Key, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

mod camera;
mod geometry;
mod io;
mod light;
mod material;
mod mesh;
mod model;
mod prelude;
mod scene;
mod screen_capture;
mod shader;
mod text;
mod texture;
mod user_inputs;
mod window;

struct State {
  scene: Scene,
  camera: Camera,
  user_inputs: UserInputs,
  shader_effect: i32,

  start: Instant,
  last_tick: Instant,
}

impl State {
  pub fn elapsed(&self) -> f32 {
    self.start.elapsed().as_nanos() as f32 / 1e9
  }

  pub fn dt(&self) -> f32 {
    self.last_tick.elapsed().as_nanos() as f32 / 1e9
  }
}

fn lock_cursor(window: &Window) {
  window.winit().set_cursor_visible(false);
  window.winit().set_cursor_grab(true).unwrap();
}

const DRAW_RATE: f32 = 60.;

unsafe fn run_event_loop(
  gl: Context,
  event_loop: EventLoop<()>,
  window: Window,
  mut state: State,
  draw: impl Fn(&Context, &mut State) + 'static,
  update: impl Fn(&mut State, Event<()>, bool) + 'static,
) {
  #[cfg(target_arch = "wasm32")]
  let mut cursor_locked = false;
  #[cfg(not(target_arch = "wasm32"))]
  let cursor_locked = true;

  // Event loop
  let mut last_draw = Instant::now();
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
        draw(&gl, &mut state);
        window.swap_buffers();
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

    let since_last_draw = last_draw.elapsed().as_nanos() as f32 / 1e9;
    if since_last_draw > 1. / DRAW_RATE {
      draw(&gl, &mut state);
      window.swap_buffers();
      last_draw = Instant::now();
    }

    update(&mut state, event, cursor_locked);
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

    // Turn on OpenGL features
    gl.enable(glow::DEPTH_TEST);
    gl.enable(glow::STENCIL_TEST);
    gl.enable(glow::BLEND);
    gl.enable(glow::CULL_FACE);

    gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

    // Build monotlithic state object
    let state = State {
      camera,
      scene,
      user_inputs: UserInputs::default(),
      start: Instant::now(),
      last_tick: Instant::now(),
      shader_effect: 0,
    };

    let screen_capture = ScreenCapture::new(&gl, width, height).await?;

    let draw = move |gl: &Context, state: &mut State| {
      screen_capture.record(gl);

      // Clear the screen with a default color
      gl.clear_color(0.1, 0.1, 0.1, 1.0);
      gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
      gl.enable(glow::DEPTH_TEST);

      // Draw the scene
      state.scene.draw(gl, &state.camera, width, height).unwrap();

      screen_capture.replay(gl, |gl, shader| {
        shader.bind_uniform(gl, "effect", &state.shader_effect);
      });
    };

    let update = move |state: &mut State, event: Event<()>, cursor_locked| {
      if cursor_locked {
        state.user_inputs.update(&event);
      }

      if state.user_inputs.just_pressed(Key::Tab) {
        let num_effects = 6;
        state.shader_effect = if state.user_inputs.pressed(Key::LShift) {
          (state.shader_effect + num_effects - 1) % num_effects
        } else {
          (state.shader_effect + 1) % num_effects
        };
      }

      state.camera.update(state.dt(), &state.user_inputs);
      state.scene.update(state.elapsed(), &state.camera);
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
