extern crate num_cpus;

use std::slice;
use std::iter;
use std::usize;
use std::f32;
use std::f32::consts;
use std::ptr;
// use std::boxed::FnBox;

use std::thread;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{SyncSender, Sender, Receiver, sync_channel, channel};

use super::math::*;

use super::particle;
use super::particle::Particle;
use super::collision::Collision;
use super::ddvt::VolumeRoot;

#[derive(Clone, Copy, Default)]
struct Positions {
  solutions: f32,
  ingress: f32,
  position: V2,
  last_position: V2,
}

#[derive(Clone, Default)]
struct Triggers {
  triggered: Vec<usize>,
}

#[derive(Default)]
pub struct World {
  pub dt: f32,
  pub dt2: f32,
  bb: BB,
  pub gravity: V2,

  particles: Vec<Particle>,
  triggered: Vec<Vec<usize>>,
  free_particles: Vec<usize>,
  volume_tree: VolumeRoot,
  solve_positions: Vec<Positions>,
  solve_triggered: Vec<Triggers>,

  pool: Option<WorldPool>,
}

pub struct WorldEditor<'a> {
  world: &'a mut World,
  world_evil: *mut World,
}

enum WorldJob {
  Particles(Box<ParticleJob>),
  ShowAnswers
}

struct ParticleJob {
  len: usize,
  max: usize,
  contained: usize,
  triggers: usize,
  particles: [Particle; 256],
}

struct WorldPool {
  threads: usize,
  jobs: Vec<Box<ParticleJob>>,
  positions: Vec<(Vec<Positions>, Vec<Triggers>)>,
  
  // job_tx: Sender<WorldJob>,
  next_thread: usize,
  job_txs: Vec<Sender<WorldJob>>,
  particles_rx: Receiver<Box<ParticleJob>>,
  positions_rx: Receiver<(Vec<Positions>, Vec<Triggers>)>,
  positions_rx_tx: Sender<(Vec<Positions>, Vec<Triggers>)>,
}

impl Positions {
  #[inline]
  fn add(&mut self, p: V2, i: f32) {
    self.solutions = self.solutions + 1.0;
    self.ingress = self.ingress + i;
    self.position = self.position + p;
    // self.last_position = self.last_position + lp;
  }

  #[inline]
  fn add_half(&mut self, p: V2, i: f32) {
    self.solutions = self.solutions + 0.5;
    self.ingress = self.ingress + i / 2.0;
    self.position = self.position + p.div_scale(2.0);
    // self.last_position = self.last_position + lp;
  }

  #[inline]
  fn merge(&mut self, other: &Positions) {
    self.solutions = self.solutions + other.solutions;
    self.ingress = self.ingress + other.ingress;
    self.position = self.position + other.position;
    // self.last_position = self.last_position + other.last_position;
  }

  #[inline]
  fn clear(&mut self) {
    self.solutions = 0.0;
    self.ingress = 0f32;
    self.position = V2::zero();
    // self.last_position = V2::zero();
  }
}

impl Triggers {
  fn len(&self) -> usize {
    self.triggered.len()
  }

  fn add_trigger(&mut self, id: usize) {
    self.triggered.push(id);
  }

  fn merge(&mut self, other: &Triggers) {
    for &id in other.triggered.iter() {
      if let Err(index) = self.triggered.binary_search(&id) {
        self.triggered.insert(index, id);
      }
    }
  }

  fn iter(&self) -> slice::Iter<usize> {
    self.triggered.iter()
  }

  fn clear(&mut self) {
    self.triggered.clear();
  }
}

pub struct MapParticles<'a, I : 'a> {
  iter: I,
  particles: &'a mut Vec<Particle>,
}

impl<'a, I> Iterator for MapParticles<'a, I> where I : Iterator<Item=&'a usize> + 'a {
  type Item = &'a mut Particle;
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|&id| unsafe { &mut *(&mut self.particles[id] as *mut Particle) as &'a mut Particle })
  }
}

impl<'a> MapParticlesIterator<'a> for slice::Iter<'a, usize> {}

trait MapParticlesIterator<'a> where Self : Sized + Iterator<Item=&'a usize> {
  fn map_particles(self, particles: &mut Vec<Particle>) -> MapParticles<Self> {
    MapParticles {
      iter: self,
      particles: particles,
    }
  }
}

impl<'a> WorldEditor<'a> {
  fn new(world: &'a mut World) -> WorldEditor {
    let world_evil = unsafe { &mut *(&mut *world as &mut World) as *mut World };
    WorldEditor {
      world: world,
      world_evil: world_evil,
    }
  }

  pub fn remove_particle(&self, particle: &mut Particle) {
    unsafe { &mut *self.world_evil }.remove_particle(particle);
  }

  pub fn iter_triggered(&self, trigger: usize) -> MapParticles<slice::Iter<usize>> {
    unsafe { &mut *self.world_evil }.iter_triggered(trigger)
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

  pub fn edit(&mut self) -> WorldEditor {
    WorldEditor::new(self)
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
    self.triggered.push(Default::default());
    self.solve_positions.push(Default::default());
    self.solve_triggered.push(Default::default());
    self.volume_tree.add(particle.id);
  }

  pub fn remove_particle(&mut self, particle: &mut Particle) {
    let exists = self.particles.iter().find(|p| p.id == particle.id && !p.is_dead()).is_some();
    assert!(exists);
    self.free_particles.push(particle.id);
    self.volume_tree.remove(particle.id);
    self.particles[particle.id].state = particle::State::Dead;
    // self.particles[particle.id] = Default::default();
    // particle.id = usize::MAX;
  }

  pub fn read_particle(&mut self, particle: &mut Particle) {
    *particle = self.particles[particle.id];
  }

  pub fn write_particle(&mut self, particle: &mut Particle) {
    self.particles[particle.id] = *particle;
  }

  pub fn iter_particles(&self) -> iter::Filter<slice::Iter<Particle>, fn(&&Particle) -> bool> {
    fn removed(particle: &&Particle) -> bool {
      !particle.is_dead()
    }
    self.particles.iter().filter(removed)
  }

  pub fn iter_triggered<'a>(&mut self, trigger: usize) -> MapParticles<'a, slice::Iter<usize>> {
    let evil_self = unsafe { &mut *(self as *mut World) as &mut World };
    let evil_world = unsafe { &mut *(self as *mut World) as &mut World };
    evil_self.triggered[trigger].iter().map_particles(&mut evil_world.particles)
  }

  pub fn walk_triggered<F>(&mut self, trigger: &mut Particle, mut handle: F) where F : FnMut(&mut World, &mut Particle, &mut Particle) {
    let evil_self = unsafe { &mut *(self as *mut World) as &mut World };
    let evil_world = unsafe { &mut *(self as *mut World) as &mut World };
    for &id in self.triggered[trigger.id].iter() {
      handle(evil_self, trigger, &mut evil_world.particles[id]);
    }
  }

  pub fn step(&mut self) {
    self.integrate_particles();

    self.volume_tree.update(&mut self.particles);

    {
      let mut volume_tree = unsafe { &mut *(&mut self.volume_tree as *mut VolumeRoot) as &mut VolumeRoot };

      // let mut particles : [Particle; 512] = [Default::default(); 512];
      // let mut particles_len = 0;
      // let mut contained_len = 0;
      // let mut positions = Vec::<Positions>::new();
      // while positions.len() < self.particles.len() {
      //   positions.push(Default::default());
      // }
      // for (v, (contained, uncontained)) in volume_tree.iter_volumes().enumerate() {
      //   contained_len = contained.len();
      //   self.copy_particles(&mut particles, &mut particles_len, contained, uncontained);
      //   World::test_solve_particles(&particles, particles_len, contained_len, &mut positions);
      // }
      // let mut i = 0;
      // while i < self.particles.len() {
      //   let positions = &mut positions[i];
      //   if positions.solutions > 0.0 {
      //     self.solve_positions[i].merge(positions);
      //     positions.clear();
      //   }
      //   i += 1;
      // }

      if let &mut Some(ref mut pool) = unsafe { &mut *(&mut self.pool as *mut Option<WorldPool>) as &mut Option<WorldPool> } {
        for (v, (contained, uncontained, contained_triggers, uncontained_triggers)) in volume_tree.iter_volumes().enumerate() {
          let mut job = pool.job();
          let mut particles_len = 0;
          self.copy_particles(&mut job.particles, &mut particles_len, contained, uncontained, contained_triggers, uncontained_triggers);
          job.max = self.particles.len();
          job.len = particles_len;
          job.contained = contained.len();
          job.triggers = contained.len() + uncontained.len();
          pool.send(job);
        }

        pool.solutions(&mut |positions, triggers| {
          let mut i : isize = 0;
          let l : isize = positions.len() as isize;
          let p = positions.as_mut_ptr();
          let pt = triggers.as_mut_ptr();
          let s = self.solve_positions.as_mut_ptr();
          let t = self.solve_triggered.as_mut_ptr();
          while i < l {
          // for (position, total) in positions.iter_mut().zip(self.solve_positions.iter_mut()) {
            let position = unsafe { &mut *p.offset(i) as &mut Positions };
            let triggers = unsafe { &mut *pt.offset(i) as &mut Triggers };
            if triggers.len() > 0 {
              unsafe { &mut *t.offset(i) as &mut Triggers }.merge(triggers);
              triggers.clear();
            }
            else if position.solutions > 0.0 {
              unsafe { &mut *s.offset(i) as &mut Positions }.merge(position);
              position.clear();
            }
            i += 1;
          }
        });
      }

      // println!("iter solutions");
      self.apply_solutions();
    }
  }

  fn integrate_particles(&mut self) {
    let gravity = self.gravity;
    let self_bb = self.bb;
    for particle in self.particles.iter_mut() {
      if particle.is_dynamic() {
        particle.acceleration = particle.acceleration + gravity;
        particle.integrate(self.dt2);
      }

      // let bb = particle.bbox;
      let constrain_x =
        (self_bb.l - (particle.position.x - particle.radius)).max(0.0) +
        (self_bb.r - (particle.position.x + particle.radius)).min(0.0);
      let constrain_y =
        (self_bb.b - (particle.position.y - particle.radius)).max(0.0) +
        (self_bb.t - (particle.position.y + particle.radius)).min(0.0);
      particle.bbox = BB {
        l: particle.position.x + constrain_x - particle.radius,
        b: particle.position.y + constrain_y - particle.radius,
        r: particle.position.x + constrain_x + particle.radius,
        t: particle.position.y + constrain_y + particle.radius,
      };
      particle.position.x = particle.position.x + constrain_x;
      particle.position.y = particle.position.y + constrain_y;
      particle.last_position.x = particle.last_position.x - constrain_x;
      particle.last_position.y = particle.last_position.y - constrain_y;

      // if bb.l < self.bb.l {
      //   particle.position.x += self.bb.l - bb.l;
      //   particle.last_position.x -= self.bb.l - bb.l;
      //   particle.bbox.l += self.bb.l - bb.l;
      //   particle.bbox.r += self.bb.l - bb.l;
      // }
      // else if bb.r > self.bb.r {
      //   particle.position.x += self.bb.r - bb.r;
      //   particle.last_position.x -= self.bb.r - bb.r;
      //   particle.bbox.l += self.bb.r - bb.r;
      //   particle.bbox.r += self.bb.r - bb.r;
      // }
      // if bb.b < self.bb.b {
      //   particle.position.y += self.bb.b - bb.b;
      //   particle.last_position.y -= self.bb.b - bb.b;
      //   particle.bbox.b += self.bb.b - bb.b;
      //   particle.bbox.t += self.bb.b - bb.b;
      // }
      // else if bb.t > self.bb.t {
      //   particle.position.y += self.bb.t - bb.t;
      //   particle.last_position.y -= self.bb.t - bb.t;
      //   particle.bbox.b += self.bb.t - bb.t;
      //   particle.bbox.t += self.bb.t - bb.t;
      // }
    }
  }

  fn copy_particles(&self, particles: &mut [Particle], particles_len: &mut usize, contained: &[usize], uncontained: &[usize], contained_triggers: &[usize], uncontained_triggers: &[usize]) {
    // *particles_len = 0;
    // for (i, &id) in contained.iter().chain(uncontained.iter()).enumerate() {
    //   // println!("{}", *id);
    //   // *particles_len = i + 1;
    //   particles[i] = self.particles[id];
    // }
    let s = self.particles.as_ptr();
    let mut p = particles.as_mut_ptr();
    let c = contained.as_ptr();
    let cl = contained.len();
    let u = uncontained.as_ptr();
    let ul = uncontained.len();
    let ct = contained_triggers.as_ptr();
    let ctl = contained_triggers.len();
    let ut = uncontained_triggers.as_ptr();
    let utl = uncontained_triggers.len();
    let mut i = 0;
    while i < cl {
      unsafe { *p.offset(i as isize) = *s.offset(*c.offset(i as isize) as isize); }
      i += 1;
    }
    i = 0;
    p = unsafe { p.offset(cl as isize) };
    while i < ul {
      unsafe { *p.offset(i as isize) = *s.offset(*u.offset(i as isize) as isize); }
      i += 1;
    }
    i = 0;
    p = unsafe { p.offset(ul as isize) };
    while i < ctl {
      unsafe { *p.offset(i as isize) = *s.offset(*ct.offset(i as isize) as isize); }
      i += 1;
    }
    i = 0;
    p = unsafe { p.offset(ctl as isize) };
    while i < utl {
      unsafe { *p.offset(i as isize) = *s.offset(*ut.offset(i as isize) as isize); }
      i += 1;
    }
    *particles_len = cl + ul + ctl + utl;
  }

  fn test_solve_particles(particles: &[Particle], particles_len: usize, contained_len: usize, triggers_start: usize, positions: &mut [Positions], scratch: &mut [Positions], triggers: &mut [Triggers]) {
    let mut i = 0;
    let mut j = 0;
    let mut p = particles.as_ptr();
    let mut s = scratch.as_mut_ptr();
    let mut t = triggers.as_mut_ptr();
    while i < contained_len {
      let a = unsafe { *p.offset(i as isize) };
      let a_p = &a;
      let sa = unsafe { &mut *s.offset(i as isize) };
      j = i + 1;
      while j < triggers_start {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(a_p, b) {
          let (ap, bp, ain, bin) = Collision::solve2(a_p, b);
          let sb = unsafe { &mut *s.offset(j as isize) };
          sa.add(ap, ain);
          sb.add(bp, bin);
        }
        j += 1;
      }
      i += 1;
    }
    while i < triggers_start {
      let a = unsafe { *p.offset(i as isize) };
      let ap = &a;
      let sa = unsafe { &mut *s.offset(i as isize) };
      j = i + 1;
      while j < triggers_start {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(ap, b) {
          let (ap, bp, ain, bin) = Collision::solve2(ap, b);
          let sb = unsafe { &mut *s.offset(j as isize) };
          sa.add_half(ap, ain);
          sb.add_half(bp, bin);
        }
        j += 1;
      }
      i += 1;
    }
    while i < particles_len {
      let a = unsafe { *p.offset(i as isize) };
      let ap = &a;
      let ta = unsafe { &mut *t.offset(a.id as isize) };
      j = 0;
      while j < triggers_start {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(ap, b) {
          ta.add_trigger(b.id);
        }
        j += 1;
      }
      j = i + 1;
      while j < particles_len {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(ap, b) {
          let tb = unsafe { &mut *t.offset(b.id as isize) };
          ta.add_trigger(b.id);
          tb.add_trigger(a.id);
        }
        j += 1;
      }
      i += 1;
    }

    i = 0;
    while i < particles_len {
      positions[particles[i].id].merge(&scratch[i]);
      scratch[i].clear();
      i += 1;
    }
  }

  fn apply_solutions(&mut self) {
    let mut i = 0;
    let l = self.solve_positions.len();
    let s = self.solve_positions.as_mut_ptr();
    let p = self.particles.as_mut_ptr();
    let dt2 = self.dt2;
    while i < l {
      let particle = unsafe { &mut *p.offset(i as isize) };
      if particle.is_dynamic() {
        // let positions = &mut self.solve_positions[i];
        let positions = unsafe { &mut *s.offset(i as isize) };
      // for (i, positions) in self.solve_positions.iter_mut().enumerate() {
        if positions.solutions > 0.0 {
          // let particle = &mut self.particles[i];
          // let particle = unsafe { &mut *p.offset(i as isize) };
          let solutions = positions.solutions;
          let inv_solutions = 1.0 / positions.solutions;
          if positions.ingress > 0.25 &&
            particle.acceleration.dot(positions.position) < 0.0 {
            particle.last_position =
              particle.acceleration
                .project_scale_add(
                  positions.position,
                  dt2 * positions.ingress * particle.radius,
                  positions.last_position.scale_add(
                    inv_solutions,
                    (particle.last_position - particle.position)
                      .scale_add(
                        // 0.9999,
                        1.0 - 0.0001 * solutions,
                        particle.position
                      )
                  )
                );
          }
          else {
            particle.last_position =
              positions.last_position.scale_add(
                inv_solutions,
                (particle.last_position - particle.position)
                  .scale_add(
                    // 0.9999,
                    1.0 - 0.0001 * solutions,
                    particle.position
                  )
              );
          }
          particle.position =
            positions.position.scale_add(inv_solutions, particle.position);
          particle.acceleration = V2::zero();
          positions.clear();
        }
      }
      else if particle.is_trigger() {
        self.triggered[i].clear();
        self.triggered[i].extend(self.solve_triggered[i].iter().cloned());
        self.solve_triggered[i].clear();
      }
      i += 1;
    }
  }
}

impl Default for ParticleJob {
  fn default() -> ParticleJob {
    ParticleJob {
      len: 0,
      max: 0,
      contained: 0,
      triggers: 0,
      particles: [Default::default(); 256],
    }
  }
}

impl WorldPool {
  fn new() -> WorldPool {
    // let (job_tx, job_rx) = channel();
    // let job_rx_mutex = Arc::new(Mutex::new(job_rx));
    let mut job_txs = Vec::<Sender<WorldJob>>::new();
    let (particles_tx, particles_rx) = channel();
    let (positions_tx, positions_rx) = channel();
    let (positions_rx_tx, positions_tx_rx) = channel();
    let positions_tx_rx_mutex = Arc::new(Mutex::new(positions_tx_rx));

    let threads = num_cpus::get();
    println!("WorldPool using {} threads.", threads);

    for i in 0..threads {
      let (job_tx, job_rx) = channel();
      job_txs.push(job_tx);
      // let job_rx_mutex = job_rx_mutex.clone();
      let particles_tx = particles_tx.clone();
      let positions_tx = positions_tx.clone();
      let positions_tx_rx_mutex = positions_tx_rx_mutex.clone();
      thread::spawn(move || {
        let mut maybe_solutions : Option<Vec<Positions>> = Some(Vec::<Positions>::new());
        let mut maybe_triggers = Some(Vec::<Triggers>::new());
        let mut scratch = [Positions { .. Default::default() }; 256];
        loop {
          // let job = {
          //   job_rx_mutex.lock().unwrap().recv().unwrap()
          // };
          let job  = job_rx.recv().unwrap();
          match job {
            WorldJob::Particles(particles) => {
              // println!("rx particles job");
              if let (Some(solutions), Some(triggers)) = (maybe_solutions.as_mut(), maybe_triggers.as_mut()) {
                while solutions.len() < particles.max {
                  solutions.push(Default::default());
                }
                while triggers.len() < particles.max {
                  triggers.push(Default::default());
                }
                World::test_solve_particles(&particles.particles, particles.len, particles.contained, particles.triggers, solutions, &mut scratch, triggers);
                particles_tx.send(particles);
              }
            },
            WorldJob::ShowAnswers => {
              // println!("rx answers job");
              positions_tx.send((
                maybe_solutions.take().unwrap(),
                maybe_triggers.take().unwrap(),
              ));
              let (solutions, triggers) = positions_tx_rx_mutex.lock().unwrap().recv().unwrap();
              maybe_solutions = Some(solutions);
              maybe_triggers = Some(triggers);
            },
          }
        }
      });
    }

    WorldPool {
      threads: threads,
      jobs: Vec::<Box<ParticleJob>>::new(),
      positions: Vec::<(Vec<Positions>, Vec<Triggers>)>::new(),
      // job_tx: job_tx,
      next_thread: 0,
      job_txs: job_txs,
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
    let job_tx = &mut self.job_txs[self.next_thread];
    job_tx.send(WorldJob::Particles(job));
    self.next_thread = self.next_thread + 1;
    if self.next_thread >= self.threads {
      self.next_thread = 0;
    }
  }

  fn solutions(&mut self, handle: &mut FnMut(&mut Vec<Positions>, &mut Vec<Triggers>)) {
    for i in 0..self.threads {
      self.job_txs[i].send(WorldJob::ShowAnswers);
    }
    for i in 0..self.threads {
      let (mut solutions, mut triggers) = self.positions_rx.recv().unwrap();
      handle(&mut solutions, &mut triggers);
      self.positions.push((solutions, triggers));
    }
    for i in 0..self.threads {
      self.positions_rx_tx.send(self.positions.pop().unwrap());
    }
  }
}
