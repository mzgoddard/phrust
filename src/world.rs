use std::slice;
use std::usize;
use std::f32;
use std::f32::consts;
use std::ptr;

use std::thread;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{SyncSender, Sender, Receiver, sync_channel, channel};

use super::math::*;

use super::particle::Particle;
use super::collision::Collision;
use super::ddvt::VolumeRoot;

#[derive(Clone, Copy, Default)]
struct Positions {
  solutions: f32,
  ingress: f32,
  last_position: V2,
  position: V2,
}

#[derive(Default)]
pub struct World {
  pub dt: f32,
  bb: BB,
  pub gravity: V2,

  particles: Vec<Particle>,
  free_particles: Vec<usize>,
  volume_tree: VolumeRoot,
  solve_positions: Vec<Positions>,

  pool: Option<WorldPool>,
}

enum WorldJob {
  Particles(Box<ParticleJob>),
  ShowAnswers
}

struct ParticleJob {
  len: usize,
  max: usize,
  contained: usize,
  particles: [Particle; 256],
}

struct WorldPool {
  jobs: Vec<Box<ParticleJob>>,
  job_tx: SyncSender<WorldJob>,
  particles_rx: Receiver<Box<ParticleJob>>,
  positions_rx: Receiver<Vec<Positions>>,
  positions_rx_tx: Sender<Vec<Positions>>,
}

impl Positions {
  fn add(&mut self, p: V2, i: f32) {
    self.solutions += 1.0;
    self.ingress += i;
    // self.last_position = self.last_position + lp;
    self.position = self.position + p;
  }

  fn add_half(&mut self, p: V2, i: f32) {
    self.solutions += 0.5;
    self.ingress += i / 2.0;
    // self.last_position = self.last_position + lp;
    self.position = self.position + p.div_scale(2.0);
  }

  fn merge(&mut self, other: &Positions) {
    self.solutions += other.solutions;
    self.ingress += other.ingress;
    self.last_position = self.last_position + other.last_position;
    self.position = self.position + other.position;
  }

  fn clear(&mut self) {
    self.solutions = 0.0;
    self.ingress = 0f32;
    self.last_position = V2::zero();
    self.position = V2::zero();
  }
}

impl World {
  pub fn new(bb: BB) -> World {
    World {
      bb: bb,
      volume_tree: VolumeRoot::new(bb),
      pool: Some(WorldPool::new()),
      .. Default::default()
    }
  }

  pub fn add_particle(&mut self, particle: &mut Particle) {
    let id = if let Some(freeid) = self.free_particles.pop() {
      freeid
    }
    else {
      self.particles.len()
    };
    particle.id = id;
    self.particles.push(*particle);
    self.solve_positions.push(Default::default());
    self.volume_tree.add(particle.id);
  }

  pub fn remove_particle(&mut self, particle: &mut Particle) {
    self.free_particles.push(particle.id);
    self.particles[particle.id] = Default::default();
    self.volume_tree.remove(particle.id);
    particle.id = usize::MAX;
  }

  pub fn read_particle(&mut self, particle: &mut Particle) {
    *particle = self.particles[particle.id];
  }

  pub fn write_particle(&mut self, particle: &mut Particle) {
    self.particles[particle.id] = *particle;
  }

  pub fn iter_particles(&self) -> slice::Iter<Particle> {
    self.particles.iter()
  }

  pub fn step(&mut self) {
    self.integrate_particles();

    self.volume_tree.update(&mut self.particles);

    {
      let mut particles : [Particle; 512] = [Default::default(); 512];
      let mut particles_len = 0;
      let mut contained_len = 0;
      let mut positions : [Positions; 512] = [Default::default(); 512];
      // let mut collisions = Vec::<Collision>::with_capacity(1024);
      // let mut collisions : [Collision; 32640] = [Default::default(); 32640];
      // let mut collision_index = 0;
      // println!("iter_volumes");
      let mut volume_tree = unsafe { &mut *(&mut self.volume_tree as *mut VolumeRoot) as &mut VolumeRoot };
      // for (v, (contained, uncontained)) in volume_tree.iter_volumes().enumerate() {
      //   contained_len = contained.len();
      //   self.copy_particles(&mut particles, &mut particles_len, contained, uncontained);
      //   World::test_solve_particles(&particles, particles_len, contained_len, &mut positions);
      //
      //   let mut i = 0;
      //   while i < particles_len {
      //     let positions = &mut positions[i];
      //     if positions.solutions > 0.0 {
      //       let particle = &particles[i];
      //       self.solve_positions[particle.id].merge(positions);
      //       positions.clear();
      //     }
      //     i += 1;
      //   }
      // }

      if let &mut Some(ref mut pool) = unsafe { &mut *(&mut self.pool as *mut Option<WorldPool>) as &mut Option<WorldPool> } {
        for (v, (contained, uncontained)) in volume_tree.iter_volumes().enumerate() {
          let mut job = pool.job();
          let mut particles_len = 0;
          self.copy_particles(&mut job.particles, &mut particles_len, contained, uncontained);
          job.max = self.particles.len();
          job.len = particles_len;
          job.contained = contained.len();
          pool.send(job);
        }

        pool.solutions(&mut |positions| {
          for (position, total) in positions.iter_mut().zip(self.solve_positions.iter_mut()) {
            if position.solutions > 0.0 {
              total.merge(position);
              position.clear();
            }
          }
        });
      }

      // println!("iter solutions");
      self.apply_solutions();
    }

    // for particle in self.particles.iter_mut() {
    //   let bb = particle.bbox;
    //   if bb.l < self.bb.l {
    //     particle.position.x += self.bb.l - bb.l;
    //   }
    //   if bb.b < self.bb.b {
    //     particle.position.y += self.bb.b - bb.b;
    //   }
    //   if bb.r > self.bb.r {
    //     particle.position.x += self.bb.r - bb.r;
    //   }
    //   if bb.t > self.bb.t {
    //     particle.position.y += self.bb.t - bb.t;
    //   }
    // }
  }

  fn integrate_particles(&mut self) {
    for particle in self.particles.iter_mut() {
      particle.acceleration = particle.acceleration + self.gravity;
      particle.integrate(self.dt);
      // println!("{} {}", particle.id, particle.bbox);

      let bb = particle.bbox;
      if bb.l < self.bb.l {
        particle.position.x += self.bb.l - bb.l;
        particle.last_position.x -= self.bb.l - bb.l;
      }
      if bb.b < self.bb.b {
        particle.position.y += self.bb.b - bb.b;
        particle.last_position.y -= self.bb.b - bb.b;
      }
      if bb.r > self.bb.r {
        particle.position.x += self.bb.r - bb.r;
        particle.last_position.x -= self.bb.r - bb.r;
      }
      if bb.t > self.bb.t {
        particle.position.y += self.bb.t - bb.t;
        particle.last_position.y -= self.bb.t - bb.t;
      }
      particle.velocity = (particle.last_position - particle.position).scale(0.9999);
      particle.position_double = particle.position.scale(2.0);
      particle.position_pre_calc = particle.radius2 - particle.position.x.powi(2) - particle.position.y.powi(2);
      particle.bbox = particle.bb();
    }
  }

  fn copy_particles(&self, particles: &mut [Particle], particles_len: &mut usize, contained: &[usize], uncontained: &[usize]) {
    *particles_len = 0;
    for (i, &id) in contained.iter().chain(uncontained.iter()).enumerate() {
      // println!("{}", *id);
      *particles_len = i + 1;
      particles[i] = self.particles[id];
    }
  }

  // fn test_particles(&self, particles: &[Particle], particles_len: usize, collisions: &mut [Collision], collision_index: &mut usize) {
  //   let mut i = 0;
  //   let mut j = 0;
  //   let mut p = particles.as_ptr();
  //   let mut c = collisions.as_mut_ptr();
  //   // let mut collision = unsafe { &mut *(&mut collisions[*collision_index] as *mut Collision) as &mut Collision };
  //   while i < particles_len {
  //     // let a = &particles[i];
  //     let a = unsafe { &*p.offset(i as isize) };
  //     j = i + 1;
  //     while j < particles_len {
  //       // if collision_index <= collisions.len() {
  //       //   collisions.push(Default::default());
  //       // }
  //       if let Some(collision) = Collision::test(
  //         a,
  //         // particles[j],
  //         unsafe { &*p.offset(j as isize) },
  //         i, j
  //       ) {
  //         unsafe { ptr::write(c.offset(*collision_index as isize), collision) };
  //         // collisions[*collision_index] = collision;
  //         *collision_index += 1;
  //         // collision = unsafe { &mut *(&mut collisions[*collision_index] as *mut Collision) as &mut Collision };
  //       }
  //       j += 1;
  //     }
  //     i += 1;
  //   }
  //   // for i in 0..particles_len {
  //   //   for j in (i + 1)..particles_len {
  //   //     if collisions[collision_index].test(&particles[i], &particles[j], i as u32, j as u32) {
  //   //       collision_index += 1;
  //   //     }
  //   //   }
  //   // }
  //   // let iter_a = particles.iter().take(particles_len).enumerate();
  //   // let mut iter_ = iter_a.clone();
  //   // for (i, a) in iter_a {
  //   //   iter_.next();
  //   //   for (j, b) in iter_.clone() {
  //   //     if collisions[*collision_index].test(a, b, i as u32, j as u32) {
  //   //       *collision_index += 1;
  //   //     }
  //   //   }
  //   // }
  // }
  //
  // fn solve_collisions(&mut self, particles: &[Particle], collisions: &[Collision], collision_index: usize) {
  //   let p = particles.as_ptr();
  //   let s = self.solve_positions.as_mut_ptr();
  //   for (i, collision) in collisions.iter().take(collision_index).enumerate() {
  //     let a = unsafe { &*p.offset(collision.a_ddvt_index as isize) };
  //     let b = unsafe { &*p.offset(collision.b_ddvt_index as isize) };
  //     // let a = &particles[collision.a_ddvt_index];
  //     // let b = &particles[collision.b_ddvt_index];
  //     let (ap, alp, bp, blp) = collision.solve(a, b);
  //     (unsafe { &mut *s.offset(a.id as isize) }).add(alp, ap, collision.ingress);
  //     (unsafe { &mut *s.offset(b.id as isize) }).add(blp, bp, collision.ingress);
  //     // self.solve_positions[a.id].add(alp, ap, collision.ingress);
  //     // self.solve_positions[b.id].add(blp, bp, collision.ingress);
  //   }
  // }

  fn test_solve_particles(particles: &[Particle], particles_len: usize, contained_len: usize, positions: &mut [Positions]) {
    let mut i = 0;
    let mut j = 0;
    let mut p = particles.as_ptr();
    let mut s = positions.as_mut_ptr();
    while i < contained_len {
      let a = unsafe { *p.offset(i as isize) };
      let a_p = &a;
      let sa = unsafe { &mut *s.offset(a.id as isize) };
      j = i + 1;
      while j < contained_len {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(a_p, b) {
          let (ap, bp, ain, bin) = Collision::solve2(a_p, b);
          let sb = unsafe { &mut *s.offset(b.id as isize) };
          sa.add(ap, ain);
          sb.add(bp, bin);
        }
        j += 1;
      }
      while j < particles_len {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(a_p, b) {
          let (ap, bp, ain, bin) = Collision::solve2(a_p, b);
          let sb = unsafe { &mut *s.offset(b.id as isize) };
          sa.add(ap, ain);
          sb.add(bp, bin);
        }
        j += 1;
      }
      i += 1;
    }
    while i < particles_len {
      let a = unsafe { *p.offset(i as isize) };
      let ap = &a;
      let sa = unsafe { &mut *s.offset(a.id as isize) };
      j = i + 1;
      while j < particles_len {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(ap, b) {
          let (ap, bp, ain, bin) = Collision::solve2(ap, b);
          let sb = unsafe { &mut *s.offset(b.id as isize) };
          sa.add_half(ap, ain);
          sb.add_half(bp, bin);
        }
        j += 1;
      }
      i += 1;
    }
  }

  fn apply_solutions(&mut self) {
    for (i, positions) in self.solve_positions.iter_mut().enumerate() {
      if positions.solutions > 0.0 {
        let particle = &mut self.particles[i];
        particle.last_position = (
          positions.last_position +
          (
            particle.position + particle.velocity
          ).scale(positions.solutions as f32)
          // positions.position.scale(0.5)
        ).div_scale(positions.solutions as f32);
        particle.position = (
          particle.position.scale(positions.solutions as f32) +
          positions.position
        ).div_scale(positions.solutions as f32);
        if particle.acceleration.dot(positions.position) < 0.0 {
          let accel_mag = particle.acceleration.mag();
          particle.last_position = particle.last_position +
            particle.acceleration
              .project(positions.position)
              .scale(
                // self.dt * self.dt / accel_mag * (particle.radius / f32::max(particle.radius - positions.position.mag(), 1.0))
                self.dt * self.dt * (positions.ingress * particle.radius)
              );
        }
        particle.acceleration = V2::zero();
        let pressure = particle.pressure;
        let radius = particle.radius;
        // let new_pressure_diff = (1f32 + (positions.ingress * radius).log(
        //   (radius)
        // ) - pressure);
        let new_pressure_diff = 1.0 - pressure + radius / (radius - positions.ingress);
        particle.pressure = f32::max(
          1f32,
          f32::min(
            4f32,
            (if new_pressure_diff > 0.0 {pressure + 4f32} else {pressure}) * 0.125
            // pressure + if new_pressure_diff > 0.0 {0.0001} else {-0.00001}
            // new_pressure_diff / if new_pressure_diff > 0.0 {10f32} else {1100f32} + pressure,
          )
        );
        // particle.pressure_radius = particle.pressure * particle.radius;
        // particle.pressure_radius2 = particle.pressure * particle.pressure * particle.radius2;
        // particle.pressure_mass = particle.pressure * particle.mass * particle.pressure;
        particle.pressure_radius = particle.radius;
        particle.pressure_radius2 = particle.radius2;
        particle.pressure_mass = particle.mass;
        positions.clear();
      }
      else {
        let pressure = self.particles[i].pressure;
        self.particles[i].pressure = f32::max(1.0, pressure * (1.0 - 1.0 / 1100.0));
        let particle = &mut self.particles[i];
        // particle.pressure_radius = particle.pressure * particle.radius;
        // particle.pressure_radius2 = particle.pressure * particle.pressure * particle.radius2;
        // particle.pressure_mass = particle.pressure * particle.mass *particle.pressure;
        particle.pressure_radius = particle.radius;
        particle.pressure_radius2 = particle.radius2;
        particle.pressure_mass = particle.mass;
      }
    }
  }
}

impl Default for ParticleJob {
  fn default() -> ParticleJob {
    ParticleJob {
      len: 0,
      max: 0,
      contained: 0,
      particles: [Default::default(); 256],
    }
  }
}

impl WorldPool {
  fn new() -> WorldPool {
    let (job_tx, job_rx) = sync_channel(4);
    let job_rx_mutex = Arc::new(Mutex::new(job_rx));
    let (particles_tx, particles_rx) = channel();
    let (positions_tx, positions_rx) = channel();
    let (positions_rx_tx, positions_tx_rx) = channel();
    let positions_tx_rx_mutex = Arc::new(Mutex::new(positions_tx_rx));

    for i in 0..4 {
      let job_rx_mutex = job_rx_mutex.clone();
      let particles_tx = particles_tx.clone();
      let positions_tx = positions_tx.clone();
      let positions_tx_rx_mutex = positions_tx_rx_mutex.clone();
      thread::spawn(move || {
        let mut maybe_solutions : Option<Vec<Positions>> = Some(Vec::<Positions>::new());
        loop {
          let job = {
            job_rx_mutex.lock().unwrap().recv().unwrap()
          };
          match job {
            WorldJob::Particles(particles) => {
              // println!("rx particles job");
              if let Some(ref mut solutions) = maybe_solutions {
                while solutions.len() < particles.max {
                  solutions.push(Default::default());
                }
                World::test_solve_particles(&particles.particles, particles.len, particles.contained, solutions);
                particles_tx.send(particles);
              }
            },
            WorldJob::ShowAnswers => {
              // println!("rx answers job");
              positions_tx.send(maybe_solutions.take().unwrap());
              maybe_solutions = Some(positions_tx_rx_mutex.lock().unwrap().recv().unwrap());
            },
          }
        }
      });
    }

    WorldPool {
      jobs: Vec::<Box<ParticleJob>>::new(),
      job_tx: job_tx,
      particles_rx: particles_rx,
      positions_rx: positions_rx,
      positions_rx_tx: positions_rx_tx,
    }
  }

  fn job(&mut self) -> Box<ParticleJob> {
    if let Ok(particles) = self.particles_rx.try_recv() {
      particles
    }
    else if let Some(particles) = self.jobs.pop() {
      particles
    }
    else {
      Box::new(Default::default())
    }
  }

  fn send(&mut self, job: Box<ParticleJob>) {
    self.job_tx.send(WorldJob::Particles(job));
  }

  fn solutions(&mut self, handle: &mut FnMut(&mut Vec<Positions>)) {
    for i in 0..4 {
      self.job_tx.send(WorldJob::ShowAnswers);
    }
    for i in 0..4 {
      let mut solutions = self.positions_rx.recv().unwrap();
      handle(&mut solutions);
      self.positions_rx_tx.send(solutions);
    }
  }
}
