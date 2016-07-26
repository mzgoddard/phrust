extern crate glfw;
extern crate time;
extern crate rand;
#[macro_use]
extern crate phrust;

use time::get_time;
use rand::{random, Open01};

use self::glfw::Context;

// #[macro_use]
// mod math;
// mod particle;
// mod collision;
// mod ddvt;
// mod ddvt_bench;
// mod quad_tree;
// mod world;
// mod world_renderer;

use std::f32;
use std::f64;

// #[macro_use]
// use math;
// #[macro_use]
// use phrust;
// #[macro_use]
use phrust::math::*;
use phrust::world;
use phrust::world_renderer;
use phrust::particle;

fn now() -> f64 {
  let spec = get_time();
  (spec.sec as f64) + (spec.nsec as f64 / 1e9f64)
}

pub fn main() {
  let mut world = world::World::new(bb!(0, 0, 640, 640));
  world.dt = 0.016;
  world.dt2 = 0.016 * 0.016;
  world.gravity = v2!(0, -0.01 / world.dt / world.dt);

  for i in 0..(8192) {
    let Open01(factor_rand) = random::<Open01<f32>>();
    let Open01(radius_rand) = random::<Open01<f32>>();
    let radius = 3f32 + radius_rand * 2.0;
    world.add_particle(&mut particle::Particle {
      position: v2!(20 + (i % 300) * 2, 20 + (i / 300) * 2),
      last_position: v2!(20 + (i % 300) * 2, 20 + (i / 300) * 2),
      radius: radius,
      radius2: radius * radius,
      // factor: 0.75f32,
      // factor: 0.5f32 + factor_rand / 2.5,
      mass: f32::consts::PI * radius * radius,
      // bbox: BB::from_circle(v2!(3 + (i % 128) * 3, 320 + (i / 128) * 3), 3f32),
      .. Default::default()
    });
  }

  let mut trigger = particle::Particle {
    position: v2!(320, 100),
    last_position: v2!(320, 100),
    radius: 100.0,
    radius2: 100.0 * 100.0,
    state: particle::State::Trigger,
    .. Default::default()
  };
  world.add_particle(&mut trigger);

  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  let (mut window, events) = glfw.create_window(640, 640, "Hello this is window", glfw::WindowMode::Windowed)
    .expect("Failed to create GLFW window.");

  window.set_key_polling(true);
  window.make_current();

  world_renderer::WorldRenderer::init_gl(|s| window.get_proc_address(s) as *const _);

  const MAX_FRAMES : usize = 60;
  let mut frames = [0f64; 60];
  let mut frame_index = 0;

  let mut renderer = world_renderer::WorldRenderer::new();

  let mut remove_id = 1024;
  while !window.should_close() {
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
      handle_window_event(&mut window, event);
    }

    let start = now();
    // world.remove_particle(&mut particle::Particle {
    //   id: remove_id,
    //   .. Default::default()
    // });
    // remove_id += 1;
    {
      let dt = world.dt;
      let editor = world.edit();
      for triggered in editor.iter_triggered(trigger.id) {
        // editor.remove_particle(triggered);
          triggered.last_position.x -= 5.0 * dt;
          triggered.last_position.y -= 5.0 * dt;
      }
    }
    // world.walk_triggered(&mut trigger, |world, trigger, triggered| {
    //   // println!("{} {}", triggered.position, triggered.id);
    //   // world.remove_particle(triggered);
    //   // triggered.last_position = triggered.last_position + (trigger.position - triggered.position).unit().scale(25.0 * world.dt);
    //   triggered.last_position.x -= 5.0 * world.dt;
    //   triggered.last_position.y -= 5.0 * world.dt;
    // });
    world.step();
    let end = now();
    frames[frame_index] = end - start;
    frame_index += 1;
    if frame_index >= MAX_FRAMES {
      let avg = frames.iter().fold(0f64, |carry, &frame| carry + frame) / 60f64  * 1000f64;
      println!("min {:.2}ms max {:.2}ms avg {:.2}ms stddev {:.2}",
        frames.iter().fold(f64::MAX, |carry, &frame| if carry < frame {carry} else {frame}) * 1000f64,
        frames.iter().fold(f64::MIN, |carry, &frame| if carry > frame {carry} else {frame}) * 1000f64,
        avg,
        (frames.iter().fold(0.0, |carry, &frame| {
          carry + (frame * 1000f64 - avg).powi(2)
        }) / frames.len() as f64).sqrt()
      );
      frame_index = 0;
    }

    renderer.draw(&world);

    window.swap_buffers();
  }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
  match event {
    glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
      window.set_should_close(true)
    }
    _ => {}
  }
}
