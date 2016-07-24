extern crate time;
extern crate rand;

use time::get_time;
use rand::{random, Open01};

#[macro_use]
mod math;
mod particle;
mod collision;
mod ddvt;
mod ddvt_bench;
mod quad_tree;
mod world;
mod world_renderer;

use std::f32;
use std::f64;

// #[macro_use]
// use math;
#[macro_use]
use math::*;

fn now() -> f64 {
  let spec = get_time();
  (spec.sec as f64) + (spec.nsec as f64 / 1e9f64)
}

pub fn main() {
  let mut world = world::World::new(bb!(0, 0, 640, 640));
  world.dt = 0.016;
  world.gravity = v2!(0, -0.01 / world.dt / world.dt);

  for i in 0..8192 {
    let Open01(factor_rand) = random::<Open01<f32>>();
    let Open01(radius_rand) = random::<Open01<f32>>();
    let radius = 3f32 + radius_rand;
    world.add_particle(&mut particle::Particle {
      position: v2!(3 + (i % 128) * 3, 320 + (i / 128) * 3),
      last_position: v2!(3 + (i % 128) * 3, 320 + (i / 128) * 3),
      radius: radius,
      radius2: radius * radius,
      // factor: 0.75f32,
      // factor: 0.5f32 + factor_rand / 2.5,
      mass: f32::consts::PI * radius * radius,
      // bbox: BB::from_circle(v2!(3 + (i % 128) * 3, 320 + (i / 128) * 3), 3f32),
      .. Default::default()
    });
  }

  const MAX_FRAMES : usize = 60;
  let mut frames = [0f64; 60];
  let mut frame_index = 0;

  let mut step = move |w : &mut world::World| {
    let start = now();
    w.step();
    let end = now();
    frames[frame_index] = end - start;
    frame_index += 1;
    if frame_index >= MAX_FRAMES {
      println!("{:.2}ms {:.2}ms {:.2}ms",
        frames.iter().fold(f64::MAX, |carry, &frame| if carry < frame {carry} else {frame}) * 1000f64,
        frames.iter().fold(f64::MIN, |carry, &frame| if carry > frame {carry} else {frame}) * 1000f64,
        frames.iter().fold(0f64, |carry, &frame| carry + frame) / 60f64 * 1000f64,
        // frames.iter().fold(|carry, frame| carry < frame ? carry : frame),
      );
      frame_index = 0;
    }
  };

  world_renderer::WorldRenderer::new(world, &mut step)
}
