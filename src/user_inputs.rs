use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, Event, VirtualKeyCode as Key, WindowEvent};

#[derive(Default)]
pub struct UserInputs {
  keys_pressed: HashSet<Key>,
  pub mouse_delta: (f32, f32),
}

impl UserInputs {
  pub fn pressed(&self, key: Key) -> bool {
    self.keys_pressed.contains(&key)
  }

  pub fn update(&mut self, event: &Event<()>) {
    self.mouse_delta = (0., 0.);

    match event {
      Event::WindowEvent { ref event, .. } => match event {
        WindowEvent::KeyboardInput { input, .. } => {
          if let Some(keycode) = input.virtual_keycode {
            match input.state {
              ElementState::Pressed => {
                self.keys_pressed.insert(keycode);
              }
              ElementState::Released => {
                self.keys_pressed.remove(&keycode);
              }
            }
          }
        }
        _ => {}
      },

      Event::DeviceEvent {
        event: DeviceEvent::MouseMotion { delta: (dx, dy) },
        ..
      } => {
        self.mouse_delta = (*dx as f32, *dy as f32);
      }

      _ => {}
    }
  }
}
