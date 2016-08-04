// extern crate glfw;

use std::any::Any;
use std::collections::vec_deque::VecDeque;
use std::time::{Instant};

use time::get_time;
use rand::{random, Open01};

// use super::*;

// use self::glfw::Context;

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
use super::math::*;
use super::world;
use super::world::*;
use super::world_renderer;
use super::particle;
use super::particle::Particle;

fn now() -> f64 {
  let spec = get_time();
  (spec.sec as f64) + (spec.nsec as f64 / 1e9f64)
}

fn seconds(sec: u64, nsec: u32) -> f64 {
  (sec as f64) + (nsec as f64 / 1e9f64)
}

struct Pulse {
  trigger: Particle,
  dt: f32,
  strength: f32,
  radians: f32,
}

impl WorldEffect for Pulse {
  fn add_to_world(&mut self, editor: &world::WorldEditor) {
    editor.add_particle(&mut self.trigger);
  }
  fn remove_from_world(&mut self, editor: &world::WorldEditor) {
    editor.remove_particle(&mut self.trigger);
  }
  fn apply(&mut self, editor: &world::WorldEditor) {
    self.radians += self.dt;
    for triggered in editor.iter_triggered(self.trigger.id) {
      let lpx = triggered.last_position.x;
      let lpy = triggered.last_position.y;
      triggered.last_position =
        triggered.last_position +
          (-self.trigger.position + triggered.position).unit()
            .scale(-self.strength * ((self.radians).sin().max(0.5) - 0.5) * 2.0);
    }
  }
}

struct Flow {
  particles: Vec<Particle>,
  flows: Vec<V2>,
  strength: f32,
  radius: f32,
}

impl Flow {
  fn new(strength: f32, radius: f32, positions: Vec<V2>) -> Flow {
    let mut flows = Vec::<V2>::new();
    for (a, b) in positions.iter().cloned().zip(positions.iter().cloned().skip(1)) {
      flows.push((a - b).unit());
    }
    let last_flow = flows.last().cloned().unwrap_or(v2!(0, 1));
    flows.push(last_flow);

    Flow {
      particles: positions.iter().map(|p| Particle {
        position: *p,
        radius: radius,
        state: particle::State::Trigger,
        .. Default::default()
      }).collect(),
      flows: flows,
      strength: strength,
      radius: radius,
    }
  }
}

impl WorldEffect for Flow {
  fn add_to_world(&mut self, editor: &world::WorldEditor) {
    for particle in self.particles.iter_mut() {
      editor.add_particle(particle);
    }
  }
  fn remove_from_world(&mut self, editor: &world::WorldEditor) {
    for particle in self.particles.iter_mut() {
      editor.remove_particle(particle);
    }
  }
  fn apply(&mut self, editor: &world::WorldEditor) {
    for (i, trigger) in self.particles.iter().enumerate() {
      for triggered in editor.iter_triggered(trigger.id) {
        triggered.last_position = triggered.last_position + self.flows[i].scale(self.strength);
      }
    }
  }
}

pub struct DemoInput {
  pub cursor: V2,
  pub cursor_press: bool,
}

#[derive(Default)]
struct FlowController {
  effect_ref: Option<WorldEffectRef>,
  positions: Vec<V2>,
  cursor_press: bool,
}

impl FlowController {
  fn update(&mut self, input_state: &DemoInput, world: &mut World) {
    if !input_state.cursor_press && !self.cursor_press {
      self.cursor_press = true;
    }
    if input_state.cursor_press && self.cursor_press && self.positions.len() < 20 {
      let cursor = input_state.cursor;
      if let Some(mut last) = self.positions.last().cloned() {
        while last.dist(cursor) > 25.0 && self.positions.len() < 20 {
          let unit = (cursor - last).unit();
          last = last + unit.scale(25.0);
          self.add_position(last);
        }
      }
      else {
        self.add_position(cursor);
      }
      self.add_effect(world);
    }
    else if self.positions.len() > 0 {
      self.cursor_press = false;
      self.add_effect(world);
      self.clear_positions();
    }
  }

  fn add_position(&mut self, pos: V2) {
    self.positions.push(pos);
  }

  fn clear_positions(&mut self) {
    self.positions.clear();
  }

  fn add_effect(&mut self, world: &mut World) {
    if let Some(effect_ref) = self.effect_ref.take() {
      world.remove_effect(effect_ref);
    }
    let dt = world.dt;
    self.effect_ref = Some(world.add_effect(Flow::new(0.625 * dt, 50.0, self.positions.clone())));
  }
}

pub struct Demo {
  pub options: DemoOptions,
  pub world: World,
  flow_controller: FlowController,

  frames: Vec<f64>,
  more_frames: VecDeque<f64>,
}

pub struct DemoOptions {
  pub world_bb: BB,
  pub world_dt: f32,
  pub world_gravity: V2,
  pub particle_count: usize,
  pub particle_base: f32,
  pub particle_range: f32,
}

impl Default for DemoOptions {
  fn default() -> DemoOptions {
    DemoOptions {
      world_bb: bb!(-640, -640, 640, 640),
      world_dt: 0.016,
      world_gravity: v2!(0, -0.01 / 0.016 / 0.016),
      particle_count: 16384,
      particle_base: 4.0,
      particle_range: 2.0,
    }
  }
}

impl Demo {
  pub fn new(options: DemoOptions) -> Demo {
    let world_bb = options.world_bb;
    let mut world = world::World::new(options.world_bb);
    world.dt = options.world_dt;
    world.dt2 = options.world_dt * options.world_dt;
    world.gravity = options.world_gravity;

    for i in 0..options.particle_count {
      let Open01(factor_rand) = random::<Open01<f32>>();
      let Open01(radius_rand) = random::<Open01<f32>>();
      let radius_base = options.particle_base;
      let radius_range = options.particle_range;
      let radius_mid = radius_base + radius_range / 2.0;
      let radius_max = radius_base + radius_range;
      let radius = radius_base + radius_rand * radius_range;
      let per_row = ((world_bb.width() - radius_max * 2.0) / radius_mid / 2.0) as usize;
      world.add_particle(&mut particle::Particle {
        position: v2!(
          world_bb.l + radius_max + (i % per_row) as f32 * radius_mid * 2.0,
          world_bb.b + radius_max + (i / per_row) as f32 * radius_mid * 0.65),
        radius: radius,
        // friction: 0.03,
        // friction2: 0.03 * 0.03,
        friction: 0.03 * (0.016f32).log(options.world_dt),
        friction2: (0.03 * (0.016f32).log(options.world_dt)).powi(2),
        // Set drag based on dt relative to 60hz. A base drag of 0.9999 results
        // to a max velocity of 2.56 for an object accelerating at 1.
        drag: (0.9999f32).powf((options.world_dt * options.world_dt / 0.0001) / (0.016 * 0.016 / 0.0001)),
        mass: f32::consts::PI * radius * radius,
        .. Default::default()
      });
    }

    world.add_effect(Pulse {
      dt: options.world_dt,
      strength: 6.25 * options.world_dt,
      radians: 0.0,
      trigger: particle::Particle {
        position: v2!(0, -320),
        radius: 50.0,
        state: particle::State::Trigger,
        .. Default::default()
      },
    });

    world.add_effect(Pulse {
      dt: options.world_dt,
      strength: 6.25 * options.world_dt,
      radians: 0.0,
      trigger: particle::Particle {
        position: v2!(160, -320),
        radius: 50.0,
        state: particle::State::Trigger,
        .. Default::default()
      },
    });

    world.add_effect(Pulse {
      dt: options.world_dt,
      strength: 6.25 * options.world_dt,
      radians: 0.0,
      trigger: particle::Particle {
        position: v2!(-160, -320),
        radius: 50.0,
        state: particle::State::Trigger,
        .. Default::default()
      },
    });

    let mut flow_controller = FlowController { .. Default::default() };
    flow_controller.add_position(v2!(0, -295));
    flow_controller.add_position(v2!(0, -270));
    flow_controller.add_position(v2!(0, -245));
    flow_controller.add_effect(&mut world);
    flow_controller.clear_positions();

    Demo {
      options: options,
      world: world,
      flow_controller: flow_controller,

      frames: Vec::new(),
      more_frames: VecDeque::new(),
    }
  }

  pub fn step(&mut self, input_state: &DemoInput) {
    self.flow_controller.update(input_state, &mut self.world);

    let start = Instant::now();

    self.world.step();

    let elapsed = start.elapsed();
    let duration = seconds(elapsed.as_secs(), elapsed.subsec_nanos());

    self.more_frames.push_back(duration);
    if self.more_frames.len() >= 1000 {
      self.more_frames.pop_front();
    }
    self.frames.push(duration);

    if self.frames.len() >= 60 {
      let avg = self.frames.iter().fold(0f64, |carry, &frame| carry + frame) / 60f64  * 1000f64;
      let more_avg = self.more_frames.iter().fold(0f64, |carry, &frame| carry + frame) / self.more_frames.len() as f64  * 1000f64;
      println!("min {:.2}ms {:.2}ms max {:.2}ms avg {:.2}ms {:.2}ms stddev {:.2} {:.2}",
        self.frames.iter().fold(f64::MAX, |carry, &frame| if carry < frame {carry} else {frame}) * 1000f64,
        self.more_frames.iter().fold(f64::MAX, |carry, &frame| if carry < frame {carry} else {frame}) * 1000f64,
        self.frames.iter().fold(f64::MIN, |carry, &frame| if carry > frame {carry} else {frame}) * 1000f64,
        avg,
        more_avg,
        (self.frames.iter().fold(0.0, |carry, &frame| {
          carry + (frame * 1000f64 - avg).powi(2)
        }) / self.frames.len() as f64).sqrt(),
        (self.more_frames.iter().fold(0.0, |carry, &frame| {
          carry + (frame * 1000f64 - more_avg).powi(2)
        }) / self.more_frames.len() as f64).sqrt()
      );
      self.frames.clear();
    }
  }
}
