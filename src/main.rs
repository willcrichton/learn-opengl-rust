use glow::*;

mod window;

use winit::{
  dpi,
  event::{Event, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

fn main() {
  unsafe {
    let wb = WindowBuilder::new()
      .with_title("LearnOpenGL")
      .with_inner_size(dpi::LogicalSize::new(1024., 768.));
    let event_loop = EventLoop::new();

    // Build platform-specific window and OpenGL context
    let (window, gl) = window::Window::build(wb, &event_loop);

    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Wait;
      match event {
        Event::LoopDestroyed => {
          return;
        }

        // "Emitted when all of the event loop's input events have been processed
        // and redraw processing is about to begin"
        Event::MainEventsCleared => {
          window.winit().request_redraw();
        }

        // Draw to the screen when requested
        Event::RedrawRequested(_) => {
          gl.clear_color(0.2, 0.3, 0.3, 1.0);
          gl.clear(glow::COLOR_BUFFER_BIT);
          
          window.swap_buffers();
        }

        Event::WindowEvent { ref event, .. } => match event {
          // Resize OpenGL viewport when window is resized
          WindowEvent::Resized(size) => {
            gl.viewport(0, 0, size.width as i32, size.height as i32);
          }

          // Exit loop when CloseRequested raised
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

          WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode.unwrap() {
            // Exit loop when Escape is pressed
            VirtualKeyCode::Escape => {
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
}
