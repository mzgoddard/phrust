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

  for i in 0..(8192 * 2) {
    let Open01(factor_rand) = random::<Open01<f32>>();
    let Open01(radius_rand) = random::<Open01<f32>>();
    let radius = 3f32 + radius_rand * 0.2;
    world.add_particle(&mut particle::Particle {
      position: v2!(3 + (i % 256) * 2, 3 + (i / 256) * 4),
      last_position: v2!(3 + (i % 256) * 2, 3 + (i / 256) * 4),
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
  };

  world_renderer::WorldRenderer::new(world, &mut step)
}
