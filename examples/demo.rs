extern crate glfw;
extern crate time;
extern crate rand;
#[macro_use]
extern crate phrust;

use std::any::Any;
use std::collections::vec_deque::VecDeque;
use std::time::{Instant};

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
use phrust::world::*;
use phrust::world_renderer;
use phrust::particle;
use phrust::particle::Particle;

fn now() -> f64 {
  let spec = get_time();
  (spec.sec as f64) + (spec.nsec as f64 / 1e9f64)
}

fn seconds(sec: u64, nsec: u32) -> f64 {
  (sec as f64) + (nsec as f64 / 1e9f64)
}

// enum EffectOrRef {
//   Effect(WorldEffect),
//   Ref(WorldEffectRef),
// }
//
// trait ParticleEffect {
//   fn particles(&mut self) -> &mut [Particle];
// }
//
// struct OwnedParticleEffect {
//   particles: Vec<Particle>,
// }
//
// impl world::WorldEffect for OwnedParticleEffect {
//   fn add_to_world(&mut self, editor: &world::WorldEditor) {
//     for particle in self.particles.iter_mut() {
//       editor.add_particle(particle);
//     }
//   }
//   fn remove_from_world(&mut self, editor: &world::WorldEditor) {
//     for particle in self.particles.iter_mut() {
//       editor.remove_particle(particle);
//     }
//   }
//   fn apply(&mut self, editor: &world::WorldEditor) {
//   }
// }
//
// impl ParticleEffect for OwnedParticleEffect {
//   fn particles(&mut self) -> &mut [Particle] {
//     &mut self.particles
//   }
// }
//
// struct ApplyOnTriggered<F, PE> where F : Fn(&mut Particle, &mut Particle) {
//   particles: PE,
//   onTriggered: F,
// }
//
// impl<F, PE, I> ApplyOnTriggered<F, PE> where PE : ParticleEffect<I> {
//   fn iter_particles(&mut self) -> slice::IterMut<Particle> {
//     self.particles.iter_particles()
//   }
// }
//
// impl<F, PE> world::WorldEffect for ApplyOnTriggered<F, PE> where F : Fn(&mut Particle, &mut Particle), PE : ParticleEffect {
//   fn add_to_world(&mut self, editor: &world::WorldEditor) {
//     self.particles.add_to_world(editor);
//   }
//   fn remove_from_world(&mut self, editor: &world::WorldEditor) {
//     self.particles.remove_from_world(editor);
//   }
//   fn apply(&mut self, editor: &world::WorldEditor) {
//     for particle in self.particles.particles().iter_mut() {
//       for triggered in editor.iter_triggered(particle.id) {
//         self.onTriggered(triggered, particle);
//       }
//     }
//   }
// }
//
// trait AppliableOnTriggered<F, PE> {
//   fn as_apply_on_triggered(&mut self) -> &mut ApplyOnTriggered<F, PE>;
// }
//
// impl<T, F, PE> world::WorldEffect for T where T : AppliableOnTriggered<F, PE> {
//   fn add_to_world(&mut self, editor: &world::WorldEditor) {
//     self.as_apply_on_triggered().add_to_world(world);
//   }
//   fn remove_from_world(&mut self, editor: &world::WorldEditor) {
//     self.as_apply_on_triggered().remove_from_world(world);
//   }
//   fn apply(&mut self, editor: &world::WorldEditor) {
//     self.as_apply_on_triggered().apply(world);
//   }
// }
//
// struct Pulse<F, PE> {
//   applyOnTrigger: ApplyOnTriggered<F, PE>,
//   radians: f32,
// }
//
// impl<F, PE> AppliableOnTriggered<F, PE> Pulse<F, PE> {
//   fn as_apply_on_triggered(&mut self) -> &mut ApplyOnTriggered<F, PE>
// }
//
// impl<F, PE> Pulse<F, PE> {
//   fn new(particle: Particle) -> Pulse<F, PE> {
//     Pulse {
//       radians: 0.0,
//       applyOnTrigger: ApplyOnTriggered {
//         particles: OwnedParticleEffect {
//
//         },
//         onTrigger: |triggerd, particle| {
//
//         },
//       },
//     }
//   }
// }
//
// impl<F, PE> WorldEffect for Pulse<F, PE> {
//   fn apply(&mut self, editor: &WorldEditor) {
//     self.radians
//   }
// }

// trait OwnedParticleEffect {
//   fn owned_particle(&mut self) -> &mut Particle;
// }
//
// trait OwnedParticleEffectApply {
//   fn apply(&mut self, editor: &world::WorldEditor);
// }
//
// impl<T> WorldEffect for T where T : OwnedParticleEffect + OwnedParticleEffectApply {
//   fn add_to_world(&mut self, editor: &world::WorldEditor) {
//     editor.add_particle(self.owned_particle());
//   }
//   fn remove_from_world(&mut self, editor: &world::WorldEditor) {
//     editor.remove_particle(self.owned_particle());
//   }
// }

struct Pulse {
  trigger: Particle,
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
    self.radians += 0.016;
    for triggered in editor.iter_triggered(self.trigger.id) {
      let lpx = triggered.last_position.x;
      let lpy = triggered.last_position.y;
      triggered.last_position = triggered.last_position + (-self.trigger.position + triggered.position).unit().scale(-0.1 * ((self.radians).sin().max(0.5) - 0.5) * 2.0);
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
    println!("{:?}", flows);

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

struct InputState {
  cursor: V2,
  cursor_press: bool,
}

pub fn main() {
  let world_bb = bb!(-320, -640, 320, 640);
  let mut world = world::World::new(world_bb);
  world.dt = 0.016;
  world.dt2 = 0.016 * 0.016;
  world.gravity = v2!(0, -0.01 / world.dt / world.dt);

  for i in 0..(8192 * 2) {
    let Open01(factor_rand) = random::<Open01<f32>>();
    let Open01(radius_rand) = random::<Open01<f32>>();
    let radius_base = 2.5;
    let radius_range = 2.0;
    let radius_mid = radius_base + radius_range / 2.0;
    let radius_max = radius_base + radius_range;
    let radius = radius_base + radius_rand * radius_range;
    let per_row = ((world_bb.width() - radius_max * 2.0) / radius_mid / 2.0) as usize;
    world.add_particle(&mut particle::Particle {
      position: v2!(
        world_bb.l + radius_max + (i % per_row) as f32 * radius_mid * 2.0,
        world_bb.b + radius_max + (i / per_row) as f32 * radius_mid * 0.65),
      last_position: v2!(
        world_bb.l + radius_max + (i % per_row) as f32 * radius_mid * 2.0,
        world_bb.b + radius_max + (i / per_row) as f32 * radius_mid * 0.65),
      radius: radius,
      // radius2: radius * radius,
      friction: 0.03,
      friction2: 0.0009,
      drag: 0.9999,
      // factor: 0.75f32,
      // factor: 0.5f32 + factor_rand / 2.5,
      mass: f32::consts::PI * radius * radius,
      // bbox: BB::from_circle(v2!(3 + (i % 128) * 3, 320 + (i / 128) * 3), 3f32),
      .. Default::default()
    });
  }

  // let mut trigger = particle::Particle {
  //   position: v2!(0, -320),
  //   last_position: v2!(0, -320),
  //   radius: 50.0,
  //   // radius2: 50.0 * 50.0,
  //   state: particle::State::Trigger,
  //   .. Default::default()
  // };
  // world.add_particle(&mut trigger);

  let mut pulse_effect_ref = world.add_effect(Pulse {
    radians: 0.0,
    trigger: particle::Particle {
      position: v2!(0, -320),
      last_position: v2!(0, -320),
      radius: 50.0,
      // radius2: 50.0 * 50.0,
      state: particle::State::Trigger,
      .. Default::default()
    },
  });

  world.add_effect(Pulse {
    radians: 0.0,
    trigger: particle::Particle {
      position: v2!(160, -320),
      last_position: v2!(160, -320),
      radius: 50.0,
      // radius2: 50.0 * 50.0,
      state: particle::State::Trigger,
      .. Default::default()
    },
  });

  world.add_effect(Pulse {
    radians: 0.0,
    trigger: particle::Particle {
      position: v2!(-160, -320),
      last_position: v2!(-160, -320),
      radius: 50.0,
      // radius2: 50.0 * 50.0,
      state: particle::State::Trigger,
      .. Default::default()
    },
  });

  let mut flow_positions = Vec::<V2>::new();
  let mut flow_ref : Option<WorldEffectRef> = None;
  let mut input_state = InputState {
    cursor: v2!(0, 0),
    cursor_press: false,
  };

  flow_ref = Some(world.add_effect(Flow::new(0.01, 50.0, vec!(
    v2!(0, -295),
    v2!(0, -270),
    v2!(0, -245),
  ))));

  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  let (mut window, events) = glfw.create_window(640, 640, "Hello this is window", glfw::WindowMode::Windowed)
    .expect("Failed to create GLFW window.");

  window.set_key_polling(true);
  window.set_mouse_button_polling(true);
  window.set_cursor_pos_polling(true);
  window.make_current();

  world_renderer::WorldRenderer::init_gl(|s| window.get_proc_address(s) as *const _);

  const MAX_FRAMES : usize = 60;
  let mut frames = [0f64; 60];
  let mut frame_index = 0;
  let mut more_frames = VecDeque::new();

  let mut renderer = world_renderer::WorldRenderer::new();

  let mut remove_id = 1024;
  let mut radians = 0.0;
  while !window.should_close() {
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
      // println!("{:?}", event);
      handle_window_event(&mut input_state, &mut window, event);
    }

    if input_state.cursor_press && flow_positions.len() < 20 {
      let cursor = input_state.cursor.scale(2.0) - v2!(640, 640);
      if let Some(mut last) = flow_positions.last().cloned() {
        while last.dist(cursor) > 25.0 && flow_positions.len() < 20 {
          let unit = (cursor - last).unit();
          last = last + unit.scale(25.0);
          flow_positions.push(last);
        }
      }
      else {
        flow_positions.push(cursor);
      }
      if let Some(flow_ref) = flow_ref.take() {
        world.remove_effect(flow_ref);
      }
      flow_ref = Some(world.add_effect(Flow::new(0.01, 50.0, flow_positions.clone())));
    }
    else if flow_positions.len() > 0 {
      input_state.cursor_press = false;
      if let Some(flow_ref) = flow_ref.take() {
        world.remove_effect(flow_ref);
      }
      flow_ref = Some(world.add_effect(Flow::new(0.01, 50.0, flow_positions.clone())));
      flow_positions.clear();
    }

    let start = Instant::now();
    // world.remove_particle(&mut particle::Particle {
    //   id: remove_id,
    //   .. Default::default()
    // });
    // remove_id += 1;
    let dt = world.dt;
    radians += dt;
    {
      // world.borrow_effect_mut(&mut pulse_effect_ref).get_type_id();
      if let Some(ref mut pulse_effect) = pulse_effect_ref.as_downcast_mut::<Pulse>(&mut world) {
        // println!("{}", pulse_effect.radians);
        pulse_effect.radians += 0.05;
      }
      // if let Some(ref mut pulse_effect) = (world.borrow_effect_mut(&mut pulse_effect_ref) as &mut Any).downcast_mut::<Pulse>() {
      // }
    }
    // world.bb.l = -320.0 + (radians / 10.0).sin() * 160.0;
    // world.bb.r = world.bb.l + 640.0;
    // world.read_particle(&mut trigger);
    // trigger.position.x -= 1.0;
    // if trigger.position.x < trigger.radius {
    //   trigger.position.x = 640.0 - trigger.radius;
    // }
    // // trigger.position.x = 320.0 + 160.0 * (radians as f32).sin();
    // world.write_particle(&mut trigger);
    // world.walk_triggered(&mut trigger, |world, trigger, triggered| {
    //   // println!("{} {}", triggered.position, triggered.id);
    //   // world.remove_particle(triggered);
    //   // triggered.last_position = triggered.last_position + (trigger.position - triggered.position).unit().scale(25.0 * world.dt);
    //   triggered.last_position.x -= 5.0 * world.dt;
    //   triggered.last_position.y -= 5.0 * world.dt;
    // });
    world.step();
    // let end = now();
    let elapsed = start.elapsed();
    let duration = seconds(elapsed.as_secs(), elapsed.subsec_nanos());
    // println!("{:.2}ms", (end - start) * 1000.0);
    // panic!("done now!");
    more_frames.push_back(duration);
    if more_frames.len() >= 1000 {
      more_frames.pop_front();
    }
    frames[frame_index] = duration;
    frame_index += 1;
    if frame_index >= MAX_FRAMES {
      let avg = frames.iter().fold(0f64, |carry, &frame| carry + frame) / 60f64  * 1000f64;
      let more_avg = more_frames.iter().fold(0f64, |carry, &frame| carry + frame) / more_frames.len() as f64  * 1000f64;
      println!("min {:.2}ms {:.2}ms max {:.2}ms avg {:.2}ms {:.2}ms stddev {:.2} {:.2}",
        frames.iter().fold(f64::MAX, |carry, &frame| if carry < frame {carry} else {frame}) * 1000f64,
        more_frames.iter().fold(f64::MAX, |carry, &frame| if carry < frame {carry} else {frame}) * 1000f64,
        frames.iter().fold(f64::MIN, |carry, &frame| if carry > frame {carry} else {frame}) * 1000f64,
        avg,
        more_avg,
        (frames.iter().fold(0.0, |carry, &frame| {
          carry + (frame * 1000f64 - avg).powi(2)
        }) / frames.len() as f64).sqrt(),
        (more_frames.iter().fold(0.0, |carry, &frame| {
          carry + (frame * 1000f64 - more_avg).powi(2)
        }) / more_frames.len() as f64).sqrt()
      );
      frame_index = 0;
    }

    renderer.draw(&world);

    window.swap_buffers();
  }
}

fn handle_window_event(state: &mut InputState, window: &mut glfw::Window, event: glfw::WindowEvent) {
  match event {
    glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
      window.set_should_close(true)
    }
    glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, press, _) => {
      // println!("button left {:?}", press);
      state.cursor_press = press == glfw::Action::Press;
    }
    glfw::WindowEvent::CursorPos(x, y) => {
      // println!("mouse move to {}", v2!(x, 640.0 - y));
      state.cursor = v2!(x, 640.0 - y);
    }
    _ => {}
  }
}
