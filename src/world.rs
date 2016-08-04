extern crate num_cpus;

use std::slice;
use std::iter;
use std::usize;
use std::f32;
use std::ptr;
// use std::boxed::FnBox;

use std::mem;
// use std::marker::Reflect;
// use std::raw::TraitObject;
use std::any::Any;
use std::sync::mpsc::*;

use super::math::*;

use super::particle;
use super::particle::Particle;
use super::collision::Collision;
use super::ddvt::VolumeRoot;

use super::pool::{Consumer, ConsumerPool};

#[derive(Clone, Copy, Default)]
struct Positions {
  solutions: f32,
  ingress: f32,
  position: V2,
  last_position: V2,
}

#[derive(Clone, Default)]
struct Triggers {
  used: bool,
  triggered: Vec<usize>,
}

pub struct World {
  pub dt: f32,
  pub dt2: f32,
  pub bb: BB,
  pub gravity: V2,

  changes: Vec<WorldChange>,
  effects: Vec<Box<WorldEffect>>,
  particles: Vec<Particle>,
  triggered: Vec<Vec<usize>>,
  free_particles: Vec<usize>,
  volume_tree: VolumeRoot,
  solve_positions: Vec<Positions>,
  solve_triggered: Vec<Triggers>,

  pool: Option<WorldPool>,
}

enum WorldChange {
  RemoveEffect(*mut WorldEffect),
}

#[allow(dead_code)]
pub struct WorldEditor<'a> {
  world: &'a mut World,
  world_evil: *mut World,
  // changes: Vec<WorldChange>,
}

pub struct WorldEffectRef {
  ptr: *mut WorldEffect,
}

impl WorldEffectRef {
  pub fn as_mut<'a>(&'a mut self, _: &'a mut World) -> &'a mut WorldEffect {
    unsafe { mem::transmute(self.ptr) }
  }

  pub fn as_downcast_mut<'a, T>(&'a mut self, world: &'a mut World) -> Option<&'a mut T> where T : Any {
    self.as_mut(world).as_any().downcast_mut::<T>()
  }
}

// impl WorldEffectRef {
//   fn new<WE : 'static>(effect: &mut WE) -> WorldEffectRef where WE : WorldEffect {
//     WorldEffectRef {
//       ptr: unsafe { &mut *effect as *mut WorldEffect },
//     }
//   }
// }
//
// impl<'a, WE> From<&'a mut WE> for WorldEffectRef where WE : WorldEffect + 'a {
//   fn from(effect: &'a mut WE) -> WorldEffectRef {
//     WorldEffectRef::new(effect)
//   }
// }

// pub trait WorldEffectMethods {
//   fn add_to_world(&mut self, editor: &WorldEditor);
//   fn remove_from_world(&mut self, editor: &WorldEditor);
//   fn apply(&mut self, editor: &WorldEditor);
// }

pub trait AsAny : Any {
  fn as_any(&mut self) -> &mut Any;
}

pub trait WorldEffect : AsAny {
  fn add_to_world(&mut self, editor: &WorldEditor);
  fn remove_from_world(&mut self, editor: &WorldEditor);
  fn apply(&mut self, editor: &WorldEditor);
  // fn as_any(&mut self) -> &mut Any;
  // fn as_any(&mut self) -> &mut Any {
  //   &mut *self as &mut Any
  // }
}

impl<T> AsAny for T where T : Any {
  fn as_any(&mut self) -> &mut Any {
    self as &mut Any
  }
}

// impl<T> T where T : WorldEffect {
//   fn as_any(&mut self) -> &mut Any {
//     self as &mut Any
//   }
// }

// impl<T> WorldEffect for T where T : Any {}

// pub trait ToWorldEffectRef where Self : WorldEffect + Sized + 'static {
//   fn as_world_effect_ref(&mut self) -> WorldEffectRef {
//     WorldEffectRef {
//       ptr: unsafe { &mut *self as *mut WorldEffect },
//     }
//   }
// }

// impl<T: WorldEffect + Sized + 'static> <T> {
//   fn as_world_effect_ref(&mut self) -> WorldEffectRef {
//     WorldEffectRef {
//       ptr: unsafe { &mut *self as *mut WorldEffect },
//     }
//   }
// }

const PARTICLES_PER_JOB : usize = 4096;

enum WorldJob {
  Particles(Box<ParticleJob>),
  Integrate(Box<IntegrateJob>),
  Merge(Box<MergeJob>),
  Apply(Box<ApplyJob>),
  TakeAnswers((Vec<Positions>, Vec<Triggers>)),
  ShowAnswers,
  ShutDown,
}

impl WorldJob {
  fn unwrap_particles(self) -> Box<ParticleJob> {
    if let WorldJob::Particles(job) = self {
      job
    }
    else {
      panic!("Must be ParticleJob to unwrap_merge")
    }
  }

  fn unwrap_integrate(self) -> Box<IntegrateJob> {
    if let WorldJob::Integrate(job) = self {
      job
    }
    else {
      panic!("Must be IntegrateJob to unwrap_merge")
    }
  }

  fn unwrap_merge(self) -> Box<MergeJob> {
    if let WorldJob::Merge(job) = self {
      job
    }
    else {
      panic!("Must be MergeJob to unwrap_merge")
    }
  }

  fn unwrap_apply(self) -> Box<ApplyJob> {
    if let WorldJob::Apply(job) = self {
      job
    }
    else {
      panic!("Must be ApplyJob to unwrap_apply")
    }
  }

  fn unwrap_answers(self) -> (Vec<Positions>, Vec<Triggers>) {
    if let WorldJob::TakeAnswers(answers) = self {
      answers
    }
    else {
      panic!("Must be ApplyJob to unwrap_apply")
    }
  }
}

impl Clone for WorldJob {
  fn clone(&self) -> WorldJob {
    match self {
      &WorldJob::ShowAnswers => {
        WorldJob::ShowAnswers
      },
      &WorldJob::ShutDown => {
        WorldJob::ShutDown
      },
      _ => {
        panic!("Can't clone given WorldJob")
      },
    }
  }
}

struct ParticleJob {
  // len: usize,
  max: usize,
  // contained: usize,
  // triggers: usize,
  contained: *const Vec<usize>,
  uncontained: *const Vec<usize>,
  contained_triggers: *const Vec<usize>,
  uncontained_triggers: *const Vec<usize>,
  source: *const Vec<Particle>,
  // particles: [Particle; 256],
}

unsafe impl Send for ParticleJob {}

struct IntegrateJob {
  start: usize,
  end: usize,
  particles: *mut Particle,

  bb: BB,
  dt2: f32,
  gravity: V2,
}

unsafe impl Send for IntegrateJob {}

struct MergeJob {
  start: usize,
  end: usize,
  run: usize,
  positions_a: *mut Positions,
  triggers_a: *mut Triggers,
  positions_b: *mut Positions,
  triggers_b: *mut Triggers,
}

unsafe impl Send for MergeJob {}

struct ApplyJob {
  start: usize,
  end: usize,
  dt2: f32,
  particles: *mut Particle,
  triggered: *mut Vec<usize>,
  answer_positions: *mut Positions,
  answer_triggers: *mut Triggers,
}

unsafe impl Send for ApplyJob {}

struct WorldPool {
  threads: usize,
  particles_out: usize,
  integrates_out: usize,
  particle_jobs: Vec<Box<ParticleJob>>,
  integrate_jobs: Vec<Box<IntegrateJob>>,
  merge_jobs: Vec<Box<MergeJob>>,
  apply_jobs: Vec<Box<ApplyJob>>,
  positions: Vec<(Vec<Positions>, Vec<Triggers>)>,

  result_rx: Receiver<WorldJob>,
  pool: ConsumerPool<WorldJob, WorldJob>,
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
    self.ingress = self.ingress + i * 0.5;
    self.position = p.scale_add(0.5, self.position);
    // self.last_position = self.last_position + lp;
  }

  #[inline]
  fn merge(&mut self, other: &Positions) {
    // let first_solution = self.solutions == 0.0;
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
  fn add_trigger(&mut self, id: usize) {
    self.used = true;
    self.triggered.push(id);
  }

  fn merge(&mut self, other: &Triggers) {
    self.used = true;
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
    self.used = false;
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
    let world_evil = &mut *(&mut *world as &mut World) as *mut World;
    WorldEditor {
      world: world,
      world_evil: world_evil,
    }
  }

  pub fn add_particle(&self, particle: &mut Particle) {
    unsafe { &mut *self.world_evil }.add_particle(particle);
  }

  pub fn remove_particle(&self, particle: &mut Particle) {
    // unsafe { self.changes }
    // unsafe { &mut *(&mut (*self) as *mut WorldEditor) as &mut WorldEditor }.changes.push(WorldChange::RemoveParticle(particle.id));
    unsafe { &mut *self.world_evil }.remove_particle(particle);
  }

  pub fn iter_effects(&self) -> slice::IterMut<Box<WorldEffect>> {
    unsafe { &mut *self.world_evil }.iter_effects()
  }

  pub fn iter_triggered(&self, trigger: usize) -> iter::Filter<MapParticles<'a, slice::Iter<usize>>, fn(&&mut Particle) -> bool> {
    unsafe { &mut *self.world_evil }.iter_triggered(trigger)
  }
}

impl World {
  pub fn new(bb: BB) -> World {
    World {
      dt: 0.0,
      dt2: 0.0,
      gravity: V2::zero(),
      bb: bb,

      volume_tree: VolumeRoot::new(bb!(-2621440, -2621440, 2621440, 2621440)),
      pool: Some(WorldPool::new()),

      changes: Vec::new(),
      effects: Vec::new(),
      particles: Vec::new(),
      triggered: Vec::new(),
      free_particles: Vec::new(),
      solve_positions: Vec::new(),
      solve_triggered: Vec::new(),
    }
  }

  pub fn edit(&mut self) -> WorldEditor {
    WorldEditor::new(self)
  }

  pub fn add_particle(&mut self, particle: &mut Particle) {
    let id = if let Some(freeid) = self.free_particles.pop() {
      assert!(self.particles[freeid].is_dead());
      freeid
    }
    else {
      self.particles.len()
    };
    particle.id = id;
    if particle.last_position.x == f32::INFINITY {
      particle.last_position = particle.position;
    }
    while id >= self.particles.len() {
      self.particles.push(Default::default());
      self.triggered.push(Default::default());
      self.solve_positions.push(Default::default());
      self.solve_triggered.push(Default::default());
    }
    self.particles[particle.id] = *particle;
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

  pub fn add_effect<WE>(&mut self, effect: WE) -> WorldEffectRef where WE : WorldEffect {
    let mut effect_box = Box::new(effect);
    {
      let mut editor = self.edit();
      effect_box.add_to_world(&mut editor);
    }
    let effect_ref = WorldEffectRef {
      ptr: &mut *effect_box as *mut WorldEffect,
    };
    self.effects.push(effect_box);
    effect_ref
  }

  pub fn borrow_effect_mut<'a>(&'a mut self, effect: &'a mut WorldEffectRef) -> &'a mut WorldEffect {
    effect.as_mut(self)
  }

  pub fn remove_effect(&mut self, effect: WorldEffectRef) {
    self.changes.push(WorldChange::RemoveEffect(effect.ptr));
  }

  fn _remove_effect(&mut self, effect: &mut WorldEffect) {
    if let Some(index) = self.effects.iter().position(|e| &*e.as_ref() as *const WorldEffect == &*effect as *const WorldEffect) {
      let mut removed_effect = self.effects.swap_remove(index);
      let mut editor = self.edit();
      removed_effect.remove_from_world(&mut editor);
    }
    else {
      panic!("Couldn't find exising effect to remove it");
    }
  }

  pub fn iter_effects(&mut self) -> slice::IterMut<Box<WorldEffect>> {
    self.effects.iter_mut()
  }

  pub fn iter_particles(&self) -> iter::Filter<slice::Iter<Particle>, fn(&&Particle) -> bool> {
    fn removed(particle: &&Particle) -> bool {
      !particle.is_dead()
    }
    self.particles.iter().filter(removed)
  }

  pub fn iter_triggered<'a>(&mut self, trigger: usize) -> iter::Filter<MapParticles<'a, slice::Iter<usize>>, fn(&&mut Particle) -> bool> {
    let evil_self = unsafe { &mut *(self as *mut World) as &mut World };
    let evil_world = unsafe { &mut *(self as *mut World) as &mut World };
    fn is_not_dead(particle: &&mut Particle) -> bool {
      !particle.is_dead()
    }
    evil_self.triggered[trigger].iter()
    .map_particles(&mut evil_world.particles)
    .filter(is_not_dead)
  }

  pub fn walk_triggered<F>(&mut self, trigger: &mut Particle, mut handle: F) where F : FnMut(&mut World, &mut Particle, &mut Particle) {
    let evil_self = unsafe { &mut *(self as *mut World) as &mut World };
    let evil_world = unsafe { &mut *(self as *mut World) as &mut World };
    for &id in self.triggered[trigger.id].iter() {
      handle(evil_self, trigger, &mut evil_world.particles[id]);
    }
  }

  pub fn step(&mut self) {
    self.apply_effects();

    self.integrate_particles();

    self.volume_tree.update(&mut self.particles);

    self.test_solve();
  }

  #[inline(never)]
  fn apply_effects(&mut self) {
    {
      let editor = self.edit();
      for effect in editor.iter_effects() {
        effect.apply(&editor);
      }
    }

    for change in self.changes.split_off(0).into_iter() {
      match change {
        WorldChange::RemoveEffect(effect_ptr) => {
          self._remove_effect(unsafe { &mut *effect_ptr });
        },
      }
    }
  }

  #[inline(never)]
  fn integrate_particles(&mut self) {
    if let Some(ref mut pool) = self.pool {
      let mut i = 0;
      let len = self.particles.len();
      while i < self.particles.len() {
        let mut job = pool.integrate_job();
        job.start = i;
        job.end = if i + PARTICLES_PER_JOB > len {len} else {i + PARTICLES_PER_JOB};
        job.particles = self.particles.as_mut_ptr();
        job.dt2 = self.dt2;
        job.bb = self.bb;
        job.gravity = self.gravity;
        pool.send_integrate(job);
        i += PARTICLES_PER_JOB;
      }
      pool.wait_on_integrate();
    }
  }

  fn _integrate_particles(job: &mut IntegrateJob) {
    let gravity = job.gravity;
    let self_bb = job.bb;
    let mut i = job.start;
    while i < job.end {
      let particle = unsafe { &mut *job.particles.offset(i as isize) };
      i += 1;
    // for particle in unsafe { &mut *job.particles }.iter_mut() {
      if particle.is_dynamic() {
        particle.acceleration = particle.acceleration + gravity;
        particle.integrate(job.dt2);
      }

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

      // particle.bbox = particle.bb();
      // let bb = particle.bbox;
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

  #[inline(never)]
  fn test_solve(&mut self) {
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
      for (contained, uncontained, contained_triggers, uncontained_triggers) in volume_tree.iter_volumes() {
        if contained.len() + uncontained.len() + contained_triggers.len() + uncontained_triggers.len() > 256 {
          println!("Skipping volume that can't be subdivided under current 256 particle limit");
        }

        let mut job = pool.job();
        // let mut particles_len = 0;
        // World::copy_particles(self.particles.as_ptr(), &mut job.particles, &mut particles_len, contained, uncontained, contained_triggers, uncontained_triggers);
        job.source = &self.particles;
        job.max = self.particles.len();
        job.contained = &*contained;
        job.uncontained = &*uncontained;
        job.contained_triggers = &*contained_triggers;
        job.uncontained_triggers = &*uncontained_triggers;
        // job.len = particles_len;
        // job.contained = contained.len();
        // job.triggers = contained.len() + uncontained.len();
        pool.send(job);
      }

      // pool.solutions(&mut |positions, triggers| {
      //   // World::merge_solutions(self.solve_positions.as_mut_ptr(), self.solve_triggered.as_mut_ptr(), positions, triggers);
      //   // self.apply_solutions(positions, triggers);
      // });

      let particles = &mut self.particles;
      let triggered = &mut self.triggered;
      pool.apply_solutions(self.dt2, particles, triggered);
    }

    // println!("iter solutions");
    // let solve_positions = unsafe { &mut *(&mut self.solve_positions as *mut _) };
    // let solve_triggered = unsafe { &mut *(&mut self.solve_triggered as *mut _) };
    // self.apply_solutions(solve_positions, solve_triggered);
  }

  #[inline(never)]
  fn copy_particles(s: *const Vec<Particle>, particles: &mut [Particle], particles_len: &mut usize, contained: *const Vec<usize>, uncontained: *const Vec<usize>, contained_triggers: *const Vec<usize>, uncontained_triggers: *const Vec<usize>) {
    // *particles_len = 0;
    // for (i, &id) in contained.iter().chain(uncontained.iter()).enumerate() {
    //   // println!("{}", *id);
    //   // *particles_len = i + 1;
    //   particles[i] = self.particles[id];
    // }
    let s = unsafe { (*s).as_ptr() };
    let mut p = particles.as_mut_ptr();
    let c = unsafe { (*contained).as_ptr() };
    let cl = unsafe { (*contained).len() };
    let u = unsafe { (*uncontained).as_ptr() };
    let ul = unsafe { (*uncontained).len() };
    let ct = unsafe { (*contained_triggers).as_ptr() };
    let ctl = unsafe { (*contained_triggers).len() };
    let ut = unsafe { (*uncontained_triggers).as_ptr() };
    let utl = unsafe { (*uncontained_triggers).len() };
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

  fn _test_solve_particles(particles: &mut ParticleJob, particle_copies: &mut [Particle; 256], solutions: &mut Vec<Positions>, scratch: &mut [Positions; 256], triggers: &mut Vec<Triggers>) {
    while solutions.len() < particles.max {
      solutions.push(Default::default());
    }
    while triggers.len() < particles.max {
      triggers.push(Default::default());
    }
    let mut len = 0;
    let contained = unsafe { (*particles.contained).len() };
    let triggers_len = unsafe { (*particles.contained).len() + (*particles.uncontained).len() };
    World::copy_particles(particles.source, particle_copies, &mut len, particles.contained, particles.uncontained, particles.contained_triggers, particles.uncontained_triggers);
    // println!("{} {} {}", len, contained, triggers_len);
    World::test_solve_particles(&*particle_copies, len, contained, triggers_len, solutions, scratch, triggers);
  }

  fn test_solve_particles(particles: &[Particle], particles_len: usize, contained_len: usize, triggers_start: usize, positions: &mut [Positions], scratch: &mut [Positions], triggers: &mut [Triggers]) {
    let mut i = 0;
    let mut j;
    let p = particles.as_ptr();
    let s = scratch.as_mut_ptr();
    let t = triggers.as_mut_ptr();
    let mut c = [0 as usize; 256];
    let mut ci = 0;
    while i < contained_len {
      let a = unsafe { &*p.offset(i as isize) };
      let sa = unsafe { &mut *s.offset(i as isize) };
      j = i + 1;
      while j < triggers_start {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(a, b) {
          c[ci] = j;
          ci += 1;
          // let (ap, bp, ain, bin) = Collision::solve2(a, b);
          // let sb = unsafe { &mut *s.offset(j as isize) };
          // sa.add(ap, ain);
          // sb.add(bp, bin);
        }
        j += 1;
      }
      while ci > 0 {
        ci -= 1;
        j = c[ci];
        let b = unsafe { &*p.offset(j as isize) };
        let sb = unsafe { &mut *s.offset(j as isize) };
        let (ap, bp, ain, bin) = Collision::solve2(a, b);
        sa.add(ap, ain);
        sb.add(bp, bin);
      }
      i += 1;
    }
    while i < triggers_start {
      let a = unsafe { &*p.offset(i as isize) };
      // let ap = &a;
      let sa = unsafe { &mut *s.offset(i as isize) };
      j = i + 1;
      while j < triggers_start {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(a, b) {
          c[ci] = j;
          ci += 1;
          // let (ap, bp, ain, bin) = Collision::solve2(a, b);
          // let sb = unsafe { &mut *s.offset(j as isize) };
          // sa.add_half(ap, ain);
          // sb.add_half(bp, bin);
        }
        j += 1;
      }
      while ci > 0 {
        ci -= 1;
        j = c[ci];
        let b = unsafe { &*p.offset(j as isize) };
        let sb = unsafe { &mut *s.offset(j as isize) };
        let (ap, bp, ain, bin) = Collision::solve2(a, b);
        sa.add_half(ap, ain);
        sb.add_half(bp, bin);
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

  fn _merge_answers(merge: &mut MergeJob) {
    let mut i : isize = merge.start as isize;
    let l : isize = merge.end as isize;
    let s = merge.positions_a;
    let t = merge.triggers_a;
    let p = merge.positions_b;
    let pt = merge.triggers_b;
    while i < l {
      let position = unsafe { &mut *p.offset(i) as &mut Positions };
      if position.solutions > 0.0 {
        unsafe { &mut *s.offset(i) as &mut Positions }.merge(position);
        position.clear();
      }
      else {
        let triggers = unsafe { &mut *pt.offset(i) as &mut Triggers };
        if triggers.used {
          unsafe { &mut *t.offset(i) as &mut Triggers }.merge(triggers);
          triggers.clear();
        }
      }
      i += 1;
    }
  }

  fn apply_answers(apply: &mut ApplyJob) {
    World::apply_solutions(apply.start, apply.end, apply.dt2, apply.particles, apply.triggered, apply.answer_positions, apply.answer_triggers);
  }

  #[inline(never)]
  fn apply_solutions(start: usize, end: usize, dt2: f32, p: *mut Particle, t: *mut Vec<usize>, s: *mut Positions, st: *mut Triggers) {
    let mut i = start;
    let l = end;
    // let s = solve_positions.as_mut_ptr();
    // let p = particles.as_mut_ptr();
    let dt2 = dt2;
    while i < l {
      let particle = unsafe { &mut *p.offset(i as isize) };
      if particle.is_dynamic() {
        let positions = unsafe { &mut *s.offset(i as isize) };
        if positions.solutions > 0.0 {
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
                        1.0 - particle.friction2 * solutions,
                        particle.position
                      )
                  )
                );
          }
          else {
            // // drag
            // p = p + 0.9 * (p - lp)
            // // friction
            // lp = p - 0.9 * (p - lp)
            // lp = p + 0.9 * (lp - p)
            particle.last_position =
              (particle.last_position - particle.position)
              .scale_add(
                1.0 - particle.friction2 * solutions,
                positions.last_position.scale_add(
                  inv_solutions, particle.position
                )
              );
            particle.last_position =
              positions.last_position.scale_add(
                inv_solutions,
                (particle.last_position - particle.position)
                  .scale_add(
                    // 0.9999,
                    1.0 - particle.friction2 * solutions,
                    particle.position
                  )
              );
          }
          particle.position =
            positions.position.scale_add(inv_solutions, particle.position);
          particle.acceleration = V2::zero();
          positions.clear();
        }
        else {
          particle.acceleration = V2::zero();
        }
      }
      else if particle.is_trigger() {
        unsafe {
          (*t.offset(i as isize)).clear();
          (*t.offset(i as isize)).extend((*st.offset(i as isize)).iter().cloned());
          (*st.offset(i as isize)).clear();
        }
        // triggered[i].clear();
        // triggered[i].extend(solve_triggered[i].iter().cloned());
        // solve_triggered[i].clear();
      }
      i += 1;
    }
  }
}

// impl Default for ParticleJob {
//   fn default() -> ParticleJob {
//     ParticleJob {
//       // len: 0,
//       max: 0,
//       // contained: 0,
//       // triggers: 0,
//       contained: Default::default(),
//       uncontained: Default::default(),
//       contained_triggers: Default::default(),
//       uncontained_triggers: Default::default(),
//       source: Default::default(),
//       // particles: [Default::default(); 256],
//     }
//   }
// }

impl WorldPool {
  fn new() -> WorldPool {;
    let (result_tx, result_rx) = channel();

    let threads = num_cpus::get();
    // let threads = 1;
    println!("WorldPool using {} threads.", threads);

    let pool = ConsumerPool::new(result_tx.clone(), || {
      let mut maybe_solutions : Option<Vec<Positions>> = Some(Vec::<Positions>::new());
      let mut maybe_triggers = Some(Vec::<Triggers>::new());
      let mut particle_copies = [Particle { .. Default::default() }; 256];
      let mut scratch = [Positions { .. Default::default() }; 256];

      Consumer::new(move |job| {
        match job {
          WorldJob::Particles(mut particles) => {
            if let (Some(solutions), Some(triggers)) = (maybe_solutions.as_mut(), maybe_triggers.as_mut()) {
              World::_test_solve_particles(&mut *particles, &mut particle_copies, solutions, &mut scratch, triggers);
            }
            Some(WorldJob::Particles(particles))
          },

          WorldJob::Integrate(mut integrate) => {
            World::_integrate_particles(&mut *integrate);
            Some(WorldJob::Integrate(integrate))
          },

          WorldJob::Merge(mut merge) => {
            World::_merge_answers(&mut *merge);
            Some(WorldJob::Merge(merge))
          },

          WorldJob::Apply(mut apply) => {
            World::apply_answers(&mut *apply);
            Some(WorldJob::Apply(apply))
          },

          WorldJob::TakeAnswers((solutions, triggers)) => {
            maybe_solutions = Some(solutions);
            maybe_triggers = Some(triggers);
            None
          },

          WorldJob::ShowAnswers => {
            Some(WorldJob::TakeAnswers((
              maybe_solutions.take().unwrap(),
              maybe_triggers.take().unwrap(),
            )))
          },

          WorldJob::ShutDown => {
            None
          },
        }
      })
    });

    WorldPool {
      threads: threads,
      particles_out: 0,
      integrates_out: 0,
      particle_jobs: Vec::new(),
      integrate_jobs: Vec::new(),
      merge_jobs: Vec::new(),
      apply_jobs: Vec::new(),
      positions: Vec::new(),

      result_rx: result_rx,
      pool: pool,
    }
  }

  fn job(&mut self) -> Box<ParticleJob> {
    if let Some(particles) = self.particle_jobs.pop() {
      particles
    }
    else {
      Box::new(ParticleJob {
        max: 0,
        contained: ptr::null(),
        uncontained: ptr::null(),
        contained_triggers: ptr::null(),
        uncontained_triggers: ptr::null(),
        source: ptr::null(),
      })
    }
  }

  fn integrate_job(&mut self) -> Box<IntegrateJob> {
    if let Some(integrate) = self.integrate_jobs.pop() {
      integrate
    }
    else {
      Box::new(IntegrateJob {
        start: 0,
        end: 0,
        particles: ptr::null_mut(),

        dt2: 0.0,
        bb: BB::infinity(),
        gravity: V2::zero(),
      })
    }
  }

  fn merge_job(&mut self) -> Box<MergeJob> {
    if let Some(merge) = self.merge_jobs.pop() {
      merge
    }
    else {
      Box::new(MergeJob {
        start: 0,
        end: 0,
        run: 0,
        positions_a: ptr::null_mut(),
        triggers_a: ptr::null_mut(),
        positions_b: ptr::null_mut(),
        triggers_b: ptr::null_mut(),
      })
    }
  }

  fn apply_job(&mut self) -> Box<ApplyJob> {
    if let Some(apply) = self.apply_jobs.pop() {
      apply
    }
    else {
      Box::new(ApplyJob {
        start: 0,
        end: 0,
        dt2: 0.0,
        particles: ptr::null_mut(),
        triggered: ptr::null_mut(),
        answer_positions: ptr::null_mut(),
        answer_triggers: ptr::null_mut(),
      })
    }
  }

  fn _send(&mut self, job: WorldJob) {
    self.pool.send(job).unwrap();
  }

  fn _send_all(&mut self, job: WorldJob) {
    for _ in 0..self.threads {
      self.pool.send(job.clone()).unwrap();
    }
  }

  fn send(&mut self, job: Box<ParticleJob>) {
    self.particles_out += 1;
    self._send(WorldJob::Particles(job));
  }

  fn send_integrate(&mut self, job: Box<IntegrateJob>) {
    self.integrates_out += 1;
    self._send(WorldJob::Integrate(job));
  }

  fn send_merge(&mut self, job: Box<MergeJob>) {
    self._send(WorldJob::Merge(job));
  }

  fn send_apply(&mut self, job: Box<ApplyJob>) {
    self._send(WorldJob::Apply(job));
  }

  fn wait_on_integrate(&mut self) {
    self.pool.process_main();

    while self.integrates_out > 0 {
      self.integrates_out -= 1;
      self.integrate_jobs.push(self.result_rx.recv().unwrap().unwrap_integrate());
    }
  }

  fn apply_solutions(&mut self, dt2: f32, particles: &mut Vec<Particle>, triggered: &mut Vec<Vec<usize>>) {
    // if let (&mut Some(ref mut solutions), &mut Some(ref mut triggers)) = (&mut self.maybe_solutions, &mut self.maybe_triggers) {
    // }
    self.pool.process_main();

    while self.particles_out > 0 {
      self.particles_out -= 1;
      self.particle_jobs.push(self.result_rx.recv().unwrap().unwrap_particles());
    }

    self._send_all(WorldJob::ShowAnswers);

    self.pool.process_main();

    {
      let (mut solutions, mut triggers) = self.result_rx.recv().unwrap().unwrap_answers();
      for _ in 0..(self.threads - 1) {
        self.positions.push(self.result_rx.recv().unwrap().unwrap_answers());
      }
      let mut jobs_out = 0;
      if self.threads >= 2 {
        let &mut(ref mut answer_positions, ref mut answer_triggers) = unsafe { &mut *(&mut self.positions[0] as *mut (Vec<Positions>, Vec<Triggers>)) };
        let mut j = 0;
        let len = answer_positions.len();
        while j < len {
          let mut job = self.merge_job();
          job.start = j;
          job.end = if j + PARTICLES_PER_JOB > len {len} else {j + PARTICLES_PER_JOB};
          job.run = 0;
          job.positions_a = solutions.as_mut_ptr();
          job.triggers_a = triggers.as_mut_ptr();
          job.positions_b = answer_positions.as_mut_ptr();
          job.triggers_b = answer_triggers.as_mut_ptr();
          self.send_merge(job);
          jobs_out += 1;
          j += PARTICLES_PER_JOB;
        }
        self.merge_jobs.clear();
      }

      self.pool.process_main();

      for r in 1..(self.threads - 1) {
        let &mut(ref mut answer_positions, ref mut answer_triggers) = unsafe { &mut *(&mut self.positions[r] as *mut (Vec<Positions>, Vec<Triggers>)) };
        let mut jobs_in = 0;
        while self.merge_jobs.len() > 0 {
          let mut job = self.merge_jobs.pop().unwrap();
          job.run = r;
          job.positions_b = answer_positions.as_mut_ptr();
          job.triggers_b = answer_triggers.as_mut_ptr();
          self.send_merge(job);
        }
        while jobs_in < jobs_out {
          let mut job = self.result_rx.recv().unwrap().unwrap_merge();
          if job.run > r {
            self.merge_jobs.push(job);
          }
          else {
            job.run = r;
            job.positions_b = answer_positions.as_mut_ptr();
            job.triggers_b = answer_triggers.as_mut_ptr();
            self.send_merge(job);
            jobs_in += 1;
          }
        }

        self.pool.process_main();
      }
      if self.threads >= 2 {
        let mut merge_jobs_in = 0;
        let mut apply_jobs_in = 0;
        while merge_jobs_in < jobs_out {
          match self.result_rx.recv().unwrap() {
            WorldJob::Merge(merge_job) => {
              let mut apply_job = self.apply_job();
              apply_job.start = merge_job.start;
              apply_job.end = merge_job.end;
              apply_job.dt2 = dt2;
              apply_job.particles = particles.as_mut_ptr();
              apply_job.triggered = triggered.as_mut_ptr();
              apply_job.answer_positions = solutions.as_mut_ptr();
              apply_job.answer_triggers = triggers.as_mut_ptr();
              self.send_apply(apply_job);
              self.merge_jobs.push(merge_job);
              merge_jobs_in += 1;
            },
            WorldJob::Apply(apply_job) => {
              self.apply_jobs.push(apply_job);
              apply_jobs_in += 1;
            },
            _ => {
              panic!("Received unexpected result in WorldPool");
            },
          }
        }
        assert_eq!(merge_jobs_in, jobs_out);

        self.pool.process_main();

        while apply_jobs_in < jobs_out {
          let job = self.result_rx.recv().unwrap().unwrap_apply();
          self.apply_jobs.push(job);
          apply_jobs_in += 1;
        }
        assert_eq!(apply_jobs_in, jobs_out);
      }
      else {
        let mut j = 0;
        let mut jobs_out = 0;
        let len = particles.len();
        while j < len {
          let mut job = self.apply_job();
          job.start = j;
          job.end = if j + PARTICLES_PER_JOB > len {len} else {j + PARTICLES_PER_JOB};
          job.dt2 = dt2;
          job.particles = particles.as_mut_ptr();
          job.triggered = triggered.as_mut_ptr();
          job.answer_positions = solutions.as_mut_ptr();
          job.answer_triggers = triggers.as_mut_ptr();
          self.send_apply(job);
          jobs_out += 1;
          j += PARTICLES_PER_JOB;
        }

        let mut jobs_in = 0;

        self.pool.process_main();

        while jobs_in < jobs_out {
          let job = self.result_rx.recv().unwrap().unwrap_apply();
          self.apply_jobs.push(job);
          jobs_in += 1;
        }
      }

      self.positions.push((solutions, triggers));
    }
    for _ in 0..self.threads {
      let answers = self.positions.pop().unwrap();
      self.pool.send(WorldJob::TakeAnswers(answers)).unwrap();
    }

    self.pool.process_main();

    // self._send_all(WorldJob::ShowAnswers);
    // {
    //   let (mut solutions, mut triggers) = self.positions_rx.recv().unwrap();
    //   for _ in 0..(self.threads - 1) {
    //     let (mut answer_positions, mut answer_triggers) = self.positions_rx.recv().unwrap();
    //     World::merge_solutions(solutions.as_mut_ptr(), triggers.as_mut_ptr(), &mut answer_positions, &mut answer_triggers);
    //     self.positions.push((answer_positions, answer_triggers));
    //   }
    //   {
    //     let mut apply_job = ApplyJob {
    //       start: 0,
    //       end: particles.len(),
    //       dt2: dt2,
    //       particles: particles.as_mut_ptr(),
    //       triggered: triggered.as_mut_ptr(),
    //       answer_positions: solutions.as_mut_ptr(),
    //       answer_triggers: triggers.as_mut_ptr(),
    //     };
    //     World::apply_answers(&mut apply_job);
    //   }
    //   // handle(&mut solutions, &mut triggers);
    //   self.positions.push((solutions, triggers));
    // }
    // {
    //   let (solutions, triggers) = self.positions.pop().unwrap();
    //   self.maybe_solutions = Some(solutions);
    //   self.maybe_triggers = Some(triggers);
    // }
    // for i in 1..self.threads {
    //   let answers = self.positions.pop().unwrap();
    //   self.job_txs[i].send(WorldJob::TakeAnswers(answers)).unwrap();
    //   // self.positions_rx_txs[i].send().unwrap();
    // }

    // for i in 0..self.threads {
    //   self.job_txs[i].send(WorldJob::ShowAnswers).unwrap();
    // }
    // for _ in 0..self.threads {
    //   let (mut solutions, mut triggers) = self.positions_rx.recv().unwrap();
    //   handle(&mut solutions, &mut triggers);
    //   self.positions.push((solutions, triggers));
    // }
    // for i in 0..self.threads {
    //   self.positions_rx_txs[i].send(self.positions.pop().unwrap()).unwrap();
    // }
  }
}

impl Drop for WorldPool {
  fn drop(&mut self) {
    for _ in 0..self.threads {
      self.pool.send(WorldJob::ShutDown).unwrap();
      // self.consumers[i - 1].send(WorldJob::ShutDown).unwrap();
    }
  }
}
