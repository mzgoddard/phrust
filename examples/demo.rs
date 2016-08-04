extern crate glfw;
#[macro_use]
extern crate ph;

use std::time::{Instant, Duration};

use self::glfw::Context;

use ph::math::*;
use ph::demo::*;
use ph::world_renderer::*;

fn seconds(duration: Duration) -> f64 {
  let (sec, nsec) = (duration.as_secs(), duration.subsec_nanos());
  (sec as f64) + (nsec as f64 / 1e9f64)
}

pub fn main() {
  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  let (mut window, events) = glfw.create_window(640, 640, "Hello this is window", glfw::WindowMode::Windowed)
    .expect("Failed to create GLFW window.");

  window.set_key_polling(true);
  window.set_mouse_button_polling(true);
  window.set_cursor_pos_polling(true);
  window.make_current();
  // glfw.set_swap_interval(4);

  GLDriver::init(|s| window.get_proc_address(s) as *const _);

  let mut renderer = WorldRenderer::new();

  let dt = 0.033;
  let mut demo = Demo::new(DemoOptions {
    world_dt: dt,
    world_gravity: v2!(0, -0.01 / 0.016 / 0.016 / 2.0),
    // particle_count: 4096,
    .. Default::default()
  });

  let mut input_state = DemoInput {
    cursor: v2!(0, 0),
    cursor_press: false,
  };

  demo.step(&input_state);

  let mut last = Instant::now();
  let mut elapsed = seconds(last.elapsed());

  while !window.should_close() {
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
      // println!("{:?}", event);
      handle_window_event(&mut input_state, &mut window, event);
    }

    elapsed += seconds(last.elapsed());
    last = Instant::now();

    let mut stepped = false;

    while elapsed > dt as f64 {
      elapsed -= dt as f64;
      demo.step(&input_state);
      stepped = true;
    }

    if stepped {
      renderer.update(&demo.world);
    }

    renderer.draw();
    window.swap_buffers();
  }
}

fn handle_window_event(state: &mut DemoInput, window: &mut glfw::Window, event: glfw::WindowEvent) {
  match event {
    glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
      window.set_should_close(true)
    }
    glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, press, _) => {
      state.cursor_press = press == glfw::Action::Press;
    }
    glfw::WindowEvent::CursorPos(x, y) => {
      state.cursor = v2!(x, 640.0 - y).scale(2.0) - v2!(640, 640);
    }
    _ => {}
  }
}
