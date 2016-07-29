extern crate num_cpus;

use std::slice;
use std::iter;
use std::usize;
use std::f32;
use std::f32::consts;
use std::ptr;
// use std::boxed::FnBox;

use std::mem;
// use std::marker::Reflect;
// use std::raw::TraitObject;
use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::borrow::{Borrow, BorrowMut};
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
  used: bool,
  triggered: Vec<usize>,
}

#[derive(Default)]
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
  AddParticle(Particle),
  RemoveParticle(usize),
  AddEffect(Box<WorldEffect + Sized + 'static>),
  RemoveEffect(*mut WorldEffect),
}

pub struct WorldEditor<'a> {
  world: &'a mut World,
  world_evil: *mut World,
  // changes: Vec<WorldChange>,
}

pub struct WorldEffectBox<T : ?Sized> where T : WorldEffect {
  world_ptr: *mut World,
  effect: Box<T>,
}

pub struct WorldEffectRef {
  ptr: *mut WorldEffect,
}

impl WorldEffectRef {
  pub fn as_mut<'a>(&'a mut self, world: &'a mut World) -> &'a mut WorldEffect {
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
  fn len(&self) -> usize {
    self.triggered.len()
  }

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
    let world_evil = unsafe { &mut *(&mut *world as &mut World) as *mut World };
    WorldEditor {
      world: world,
      world_evil: world_evil,
      // changes: Vec::<WorldChange>::new(),
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

// impl<'a> Drop for WorldEditor<'a> {
//   fn drop(&mut self) {
//     for effect_ptr in self.remove_effects.split_off(0).into_iter() {
//       unsafe { &mut *self.world_evil }.remove_effect(unsafe { &mut *effect_ptr })
//     }
//       match change {
//         WorldChange::AddParticle(mut particle) => {
//           unsafe { &mut *self.world_evil }.add_particle(&mut particle);
//         },
//         WorldChange::RemoveParticle(id) => {
//           unsafe { &mut *self.world_evil }.remove_particle(&mut Particle {
//             id: id,
//             .. Default::default()
//           });
//         },
//         WorldChange::AddEffect(effect) => {
//           unsafe { &mut *self.world_evil }.add_effect(effect);
//         },
//         WorldChange::RemoveEffect(effect_ptr) => {
//           unsafe { &mut *self.world_evil }.remove_effect(unsafe { &mut *effect_ptr });
//         },
//       }
//     }
//   }
// }

// impl<T> WorldEffectBox<T> where T : WorldEffect {
//   fn new(world: &mut World, effect: Box<T>) -> WorldEffectBox<T> {
//     WorldEffectBox {
//       world_ptr: &mut *world as *mut World,
//       effect: effect,
//     }
//   }
//   //
//   // fn remove(&mut self) {
//   //   unsafe { &mut *self.world_ptr }.remove_effect(&*self.effect as *const WorldEffect);
//   // }
// }
//
// impl<T> Deref for WorldEffectBox<T> where T : WorldEffect {
//   type Target = T;
//   fn deref(&self) -> &T {
//     self.effect.deref()
//   }
// }
//
// impl<T> DerefMut for WorldEffectBox<T> where T : WorldEffect {
//   fn deref_mut(&mut self) -> &mut T {
//     self.effect.deref_mut()
//   }
// }
//
// impl<T> Borrow<T> for WorldEffectBox<T> where T : WorldEffect {
//   fn borrow(&self) -> &T {
//     self.effect.borrow()
//   }
// }
//
// impl<T> BorrowMut<T> for WorldEffectBox<T> where T : WorldEffect {
//   fn borrow_mut(&mut self) -> &mut T {
//     self.effect.borrow_mut()
//   }
// }

impl World {
  pub fn new(bb: BB) -> World {
    World {
      bb: bb,
      volume_tree: VolumeRoot::new(bb!(-2621440, -2621440, 2621440, 2621440)),
      pool: Some(WorldPool::new()),
      .. Default::default()
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
          World::merge_solutions(self.solve_positions.as_mut_ptr(), self.solve_triggered.as_mut_ptr(), positions, triggers);
        });
      }

      // println!("iter solutions");
      self.apply_solutions();
    }
  }

  fn apply_effects(&mut self) {
    {
      let mut editor = self.edit();
      for effect in editor.iter_effects() {
        effect.apply(&editor);
      }
    }

    for change in self.changes.split_off(0).into_iter() {
      match change {
        WorldChange::RemoveEffect(effect_ptr) => {
          self._remove_effect(unsafe { &mut *effect_ptr });
        },
        _ => {},
      }
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
      let a = unsafe { &*p.offset(i as isize) };
      // let a_p = &mut a;
      let sa = unsafe { &mut *s.offset(i as isize) };
      j = i + 1;
      while j < triggers_start {
        let b = unsafe { &*p.offset(j as isize) };
        if Collision::test2(a, b) {
          let (ap, bp, ain, bin) = Collision::solve2(a, b);
          let sb = unsafe { &mut *s.offset(j as isize) };
          // a.position = a.position + ap;
          // b.position = b.position + bp;
          sa.add(ap, ain);
          sb.add(bp, bin);
        }
        j += 1;
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
          let (ap, bp, ain, bin) = Collision::solve2(a, b);
          let sb = unsafe { &mut *s.offset(j as isize) };
          // a.position = a.position + ap.scale(0.5);
          // b.position = b.position + bp.scale(0.5);
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

  fn merge_solutions(s: *mut Positions, t: *mut Triggers, positions: &mut Vec<Positions>, triggers: &mut Vec<Triggers>) {
    let mut i : isize = 0;
    let l : isize = positions.len() as isize;
    let p = positions.as_mut_ptr();
    let pt = triggers.as_mut_ptr();
    // let s = solve_positions.as_mut_ptr();
    // let t = solve_triggered.as_mut_ptr();
    while i < l {
    // for (position, total) in positions.iter_mut().zip(self.solve_positions.iter_mut()) {
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
                        1.0 - particle.friction2 * solutions,
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
                    1.0 - particle.friction2 * solutions,
                    particle.position
                  )
              );
          }
          // particle.last_position =
          //   particle.last_position - positions.position;
          particle.position =
            // positions.position + particle.position;
            // particle.position.scale_add(solutions, positions.position).scale(inv_solutions);
            // particle.position + positions.position.project(particle.acceleration);
            // particle.position + positions.position;
            positions.position.scale_add(inv_solutions, particle.position);
          particle.acceleration = V2::zero();
          positions.clear();
        }
        else {
          particle.acceleration = V2::zero();
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
            WorldJob::Particles(mut particles) => {
              // println!("rx particles job");
              if let (Some(solutions), Some(triggers)) = (maybe_solutions.as_mut(), maybe_triggers.as_mut()) {
                while solutions.len() < particles.max {
                  solutions.push(Default::default());
                }
                while triggers.len() < particles.max {
                  triggers.push(Default::default());
                }
                let len = particles.len;
                let contained = particles.contained;
                let triggers_len = particles.triggers;
                World::test_solve_particles(&particles.particles, len, contained, triggers_len, solutions, &mut scratch, triggers);
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
