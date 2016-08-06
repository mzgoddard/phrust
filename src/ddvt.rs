extern crate num_cpus;

use std::usize;
use std::f32;
use std::any::Any;
use std::fmt;
use std::iter;
use std::sync::mpsc::*;

use super::math::*;
use super::particle::Particle;
use super::quad_tree;
use super::quad_tree::{QuadTree, TreeSplitableWith, TreeJoinableWith, TreeSplitable, TreeJoinable};
use super::pool::*;

#[cfg(target_os="ios")]
const MAX_LEAF_VOLUME : usize = 128;
#[cfg(target_os="ios")]
const MIN_LEAF_VOLUME : usize = 96;

#[cfg(not(target_os="ios"))]
const MAX_LEAF_VOLUME : usize = 256;
#[cfg(not(target_os="ios"))]
const MIN_LEAF_VOLUME : usize = 224;

const PARTICLES_PER_JOB : usize = 4096;

#[derive(Default)]
pub struct VirtualVolume {
  bb: BB,
  center: V2,
  contained: Vec<usize>,
  contained_triggers: Vec<usize>,
  uncontained: Vec<usize>,
  uncontained_triggers: Vec<usize>,
}

#[derive(Default)]
struct VirtualRecord {
  bb: BB,
  contained: Vec<Vec<usize>>,
  contained_triggers: Vec<Vec<usize>>,
}

enum VolumeChange {
  Add(usize),
  Remove(usize),
}

#[derive(Default)]
struct OldNew {
  new: BB,
  old: BB,
  trigger: bool,
}

// #[derive(Default)]
pub struct VolumeRoot {
  root: QuadTree<VirtualVolume>,
  records: QuadTree<VirtualRecord>,
  changes: Vec<VolumeChange>,
  // triggers: Vec<bool>,
  bb_clone: Vec<OldNew>,

  threads: usize,
  pool: ConsumerPool<VolumeJob, VolumeResult>,
  result_rx: Receiver<VolumeResult>,
}

enum VolumeJob {
  CopyBB { start: usize, end: usize, old_new: *mut OldNew, particles: *const Particle },
  CopyOld { start: usize, end: usize, old_new: *mut OldNew },
  UpdateUncontained { done: usize, volume: *mut QuadTree<VirtualVolume>, old_new: *const Vec<OldNew> },
  MoveOutOfDate { done: usize, volume: *mut QuadTree<VirtualVolume>, record: *mut QuadTree<VirtualRecord>, old_new: *mut Vec<OldNew> },
  UpdateUncontainedCompleted(usize),
  MoveOutOfDateCompleted(usize),
}

unsafe impl Send for VolumeJob {}

impl fmt::Debug for VolumeJob {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &VolumeJob::CopyBB { start: _, end: _, old_new: _, particles: _ } => {
        write!(f, "CopyBB")
      },
      &VolumeJob::CopyOld { start: _, end: _, old_new: _ } => {
        write!(f, "CopyOld")
      },
      &VolumeJob::UpdateUncontained { done: _, volume: _, old_new: _ } => {
        write!(f, "UpdateUncontained")
      },
      &VolumeJob::MoveOutOfDate { done: _, volume: _, record: _, old_new: _ } => {
        write!(f, "MoveOutOfDate")
      },
      &VolumeJob::UpdateUncontainedCompleted(_) => {
        write!(f, "UpdateUncontainedCompleted")
      },
      &VolumeJob::MoveOutOfDateCompleted(_) => {
        write!(f, "MoveOutOfDateCompleted")
      },
    }
  }
}

enum VolumeResult {
  Done,
  BBDone,
}

unsafe impl Send for VolumeResult {}

impl Default for VolumeResult {
  fn default() -> VolumeResult {
    VolumeResult::Done
  }
}

impl fmt::Debug for VolumeResult {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &VolumeResult::Done => {
        write!(f, "Done")
      },
      &VolumeResult::BBDone => {
        write!(f, "BBDone")
      },
    }
  }
}

struct RecordStack<'a> {
  record: &'a mut QuadTree<VirtualRecord>,
}

struct RecordStackIter<'a> {
  #[allow(dead_code)]
  origin: &'a mut QuadTree<VirtualRecord>,
  record: Option<*mut QuadTree<VirtualRecord>>,
}

impl<'a> RecordStack<'a> {
  fn new(tree: &'a mut QuadTree<VirtualRecord>) -> RecordStack {
    RecordStack {
      record: tree,
    }
  }

  fn iter_mut(&mut self) -> RecordStackIter {
    RecordStackIter::new(self.record)
  }
}

impl<'a> RecordStackIter<'a> {
  fn new(tree: &'a mut QuadTree<VirtualRecord>) -> RecordStackIter<'a> {
    let record_evil = &mut *tree as *mut QuadTree<VirtualRecord>;
    RecordStackIter {
      origin: tree,
      record: Some(record_evil)
    }
  }
}

impl<'a> Iterator for RecordStackIter<'a> {
  type Item = &'a mut VirtualRecord;
  fn next(&mut self) -> Option<Self::Item> {
    let record = self.record.take();
    if let Some(tree) = record {
      self.record = unsafe { &mut *tree }.parent;
    }
    fn coerce<'a>(tree: *mut QuadTree<VirtualRecord>) -> &'a mut VirtualRecord {
      &mut unsafe { &mut *tree }.value
    }
    record.map(coerce)
  }
}

impl VirtualVolume {
  fn len(&self) -> usize {
    self.contained.len() + self.uncontained.len() + self.contained_triggers.len() + self.uncontained_triggers.len()
  }
}

impl QuadTree<VirtualVolume> {
  fn walk_rev_mut<F>(&mut self, mut handle: F) where F : FnMut(&mut QuadTree<VirtualVolume>) {
    self._walk_rev_mut(&mut handle);
  }

  fn _walk_rev_mut(&mut self, handle: &mut FnMut(&mut QuadTree<VirtualVolume>)) {
    handle(self);
    if let Some(ref mut children) = self.children {
      let c = children.as_mut_ptr();
      macro_rules! ptr_offset {
        ($p : expr, $i : expr) => {
          unsafe { &mut *$p.offset($i as isize) }
        }
      }
      ptr_offset!(c, 0)._walk_rev_mut(handle);
      ptr_offset!(c, 1)._walk_rev_mut(handle);
      ptr_offset!(c, 2)._walk_rev_mut(handle);
      ptr_offset!(c, 3)._walk_rev_mut(handle);
    }
  }

  fn walk_with_record_rev_mut<F>(&mut self, record: &mut QuadTree<VirtualRecord>, mut handle: F) where F : FnMut(&mut QuadTree<VirtualVolume>, &mut QuadTree<VirtualRecord>) {
    self._walk_with_record_rev_mut(record, &mut handle);
  }

  fn _walk_with_record_rev_mut<F>(&mut self, record: &mut QuadTree<VirtualRecord>, handle: &mut F) where F : FnMut(&mut QuadTree<VirtualVolume>, &mut QuadTree<VirtualRecord>) {
    handle(self, record);
    if let (&mut Some(ref mut volume_children), &mut Some(ref mut record_children)) = (&mut self.children, &mut record.children) {
      let vc = volume_children.as_mut_ptr();
      let rc = record_children.as_mut_ptr();
      macro_rules! ptr_offset {
        ($p : expr, $i : expr) => {
          unsafe { &mut *$p.offset($i as isize) }
        }
      }
      ptr_offset!(vc, 0)._walk_with_record_rev_mut(ptr_offset!(rc, 0), handle);
      ptr_offset!(vc, 1)._walk_with_record_rev_mut(ptr_offset!(rc, 1), handle);
      ptr_offset!(vc, 2)._walk_with_record_rev_mut(ptr_offset!(rc, 2), handle);
      ptr_offset!(vc, 3)._walk_with_record_rev_mut(ptr_offset!(rc, 3), handle);
    }
  }

  fn walk_bb_mut(&mut self, b: BB, handle: &Fn(&mut VirtualVolume)) {
    if let Some(ref mut children) = self.children {
      let c = children.as_mut_ptr();
      macro_rules! ptr_offset {
        ($p : expr, $i : expr) => {
          unsafe { &mut *$p.offset($i as isize) }
        }
      }

      // let cx = (self.value.bb.l + self.value.bb.r) / 2f32;
      // let cy = (self.value.bb.b + self.value.bb.t) / 2f32;
      let V2 { x: cx, y: cy } = self.value.center;
      if cy <= b.t {
        if cx >= b.l {
          ptr_offset!(c, 0).walk_bb_mut(b, handle);
          if cx <= b.r {
            ptr_offset!(c, 1).walk_bb_mut(b, handle);
            if cy >= b.b {
              ptr_offset!(c, 2).walk_bb_mut(b, handle);
              ptr_offset!(c, 3).walk_bb_mut(b, handle);
            }
          }
          else if cy >= b.b {
            ptr_offset!(c, 2).walk_bb_mut(b, handle);
          }
        }
        else {
          ptr_offset!(c, 1).walk_bb_mut(b, handle);
          if cy >= b.b {
            ptr_offset!(c, 3).walk_bb_mut(b, handle);
          }
        }
      }
      else {
        if cx >= b.l {
          ptr_offset!(c, 2).walk_bb_mut(b, handle);
          if cx <= b.r {
            ptr_offset!(c, 3).walk_bb_mut(b, handle);
          }
        }
        else {
          ptr_offset!(c, 3).walk_bb_mut(b, handle);
        }
      }
    }
    else {
      handle(&mut self.value);
    }
  }

  fn walk_contain_bb_search_mut<F>(&mut self, b: BB, handle: &mut F) where F : FnMut(&mut QuadTree<VirtualVolume>) {
    let found = if let Some(ref mut children) = self.children {
      let c = children.as_mut_ptr();
      macro_rules! ptr_offset {
        ($p : expr, $i : expr) => {
          unsafe { &mut *$p.offset($i as isize) }
        }
      }

      let cx = (self.value.bb.l + self.value.bb.r) / 2f32;
      let cy = (self.value.bb.b + self.value.bb.t) / 2f32;
      if cy <= b.b {
        if cx >= b.r {
          ptr_offset!(c, 0).walk_contain_bb_search_mut(b, handle);
          false
        }
        else if cx <= b.l {
          ptr_offset!(c, 1).walk_contain_bb_search_mut(b, handle);
          false
        }
        else {
          true
        }
      }
      else if cy >= b.t {
        if cx >= b.r {
          ptr_offset!(c, 2).walk_contain_bb_search_mut(b, handle);
          false
        }
        else if cx <= b.l {
          ptr_offset!(c, 3).walk_contain_bb_search_mut(b, handle);
          false
        }
        else {
          true
        }
      }
      else {
        true
      }
    }
    else {
      true
    };
    if found {
      handle(self);
    }
  }

  fn walk_with_record_mut<F>(&mut self, record: &mut QuadTree<VirtualRecord>, handle: &mut F) where F : FnMut(&mut QuadTree<VirtualVolume>, &mut QuadTree<VirtualRecord>) {
    if let (&mut Some(ref mut volume_children), &mut Some(ref mut record_children)) = (&mut self.children, &mut record.children) {
      let vc = volume_children.as_mut_ptr();
      let rc = record_children.as_mut_ptr();
      macro_rules! ptr_offset {
        ($p : expr, $i : expr) => {
          unsafe { &mut *$p.offset($i as isize) }
        }
      }
      ptr_offset!(vc, 0)._walk_with_record_rev_mut(ptr_offset!(rc, 0), handle);
      ptr_offset!(vc, 1)._walk_with_record_rev_mut(ptr_offset!(rc, 1), handle);
      ptr_offset!(vc, 2)._walk_with_record_rev_mut(ptr_offset!(rc, 2), handle);
      ptr_offset!(vc, 3)._walk_with_record_rev_mut(ptr_offset!(rc, 3), handle);
    }
    handle(self, record);
  }
}

impl TreeSplitableWith for VirtualVolume {
  fn tree_split_with(&mut self, children: &mut [&mut VirtualVolume], data: &mut Any) {
    let particles = data.downcast_mut::<Vec<Particle>>().unwrap();
    children[0].bb = self.bb.tl();
    children[0].center = children[0].bb.center();
    children[1].bb = self.bb.tr();
    children[1].center = children[1].bb.center();
    children[2].bb = self.bb.bl();
    children[2].center = children[2].bb.center();
    children[3].bb = self.bb.br();
    children[3].center = children[3].bb.center();

    self.contained.retain(|particle_id| {
      let particle = &mut particles[*particle_id];
      if children[0].bb.contains(particle.bbox) {
        children[0].contained.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[1].bb.contains(particle.bbox) {
        children[1].contained.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[2].bb.contains(particle.bbox) {
        children[2].contained.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[3].bb.contains(particle.bbox) {
        children[3].contained.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else {
        true
      }
    });

    self.contained_triggers.retain(|particle_id| {
      let particle = &mut particles[*particle_id];
      if children[0].bb.contains(particle.bbox) {
        children[0].contained_triggers.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[1].bb.contains(particle.bbox) {
        children[1].contained_triggers.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[2].bb.contains(particle.bbox) {
        children[2].contained_triggers.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[3].bb.contains(particle.bbox) {
        children[3].contained_triggers.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else {
        true
      }
    });

    for particle_id in self.contained.iter() {
      let particle = &mut particles[*particle_id];
      particle.uncontained = true;
      if children[0].bb.overlaps(particle.bbox) {
        children[0].uncontained.push(*particle_id);
      }
      if children[1].bb.overlaps(particle.bbox) {
        children[1].uncontained.push(*particle_id);
      }
      if children[2].bb.overlaps(particle.bbox) {
        children[2].uncontained.push(*particle_id);
      }
      if children[3].bb.overlaps(particle.bbox) {
        children[3].uncontained.push(*particle_id);
      }
    }
    for particle_id in self.uncontained.iter() {
      let particle = &mut particles[*particle_id];
      particle.uncontained = true;
      if children[0].bb.overlaps(particle.bbox) {
        children[0].uncontained.push(*particle_id);
      }
      if children[1].bb.overlaps(particle.bbox) {
        children[1].uncontained.push(*particle_id);
      }
      if children[2].bb.overlaps(particle.bbox) {
        children[2].uncontained.push(*particle_id);
      }
      if children[3].bb.overlaps(particle.bbox) {
        children[3].uncontained.push(*particle_id);
      }
    }
    self.uncontained.clear();

    for particle_id in self.contained_triggers.iter() {
      let particle = &mut particles[*particle_id];
      particle.uncontained = true;
      if children[0].bb.overlaps(particle.bbox) {
        children[0].uncontained_triggers.push(*particle_id);
      }
      if children[1].bb.overlaps(particle.bbox) {
        children[1].uncontained_triggers.push(*particle_id);
      }
      if children[2].bb.overlaps(particle.bbox) {
        children[2].uncontained_triggers.push(*particle_id);
      }
      if children[3].bb.overlaps(particle.bbox) {
        children[3].uncontained_triggers.push(*particle_id);
      }
    }
    for particle_id in self.uncontained_triggers.iter() {
      let particle = &mut particles[*particle_id];
      particle.uncontained = true;
      if children[0].bb.overlaps(particle.bbox) {
        children[0].uncontained_triggers.push(*particle_id);
      }
      if children[1].bb.overlaps(particle.bbox) {
        children[1].uncontained_triggers.push(*particle_id);
      }
      if children[2].bb.overlaps(particle.bbox) {
        children[2].uncontained_triggers.push(*particle_id);
      }
      if children[3].bb.overlaps(particle.bbox) {
        children[3].uncontained_triggers.push(*particle_id);
      }
    }
    self.uncontained_triggers.clear();

    // assert!(
    //   uncontained_len > MAX_LEAF_VOLUME && (
    //     children[0].uncontained.len() < uncontained_len ||
    //     children[1].uncontained.len() < uncontained_len ||
    //     children[2].uncontained.len() < uncontained_len ||
    //     children[3].uncontained.len() < uncontained_len
    //   ) ||
    //   uncontained_len <= MAX_LEAF_VOLUME
    // );
  }
}

impl TreeJoinableWith for VirtualVolume {
  fn tree_join_with(&mut self, children: &[&VirtualVolume], data: &mut Any) {
    let particles = data.downcast_mut::<Vec<Particle>>().unwrap();

    for &particle_id in children[0].uncontained.iter()
    .chain(children[1].uncontained.iter())
    .chain(children[2].uncontained.iter())
    .chain(children[3].uncontained.iter()) {
      if index_in_uncontained(particle_id, &self.contained) != usize::MAX {
        particles[particle_id].uncontained = false;
      }
      else if index_in_uncontained(particle_id, &self.uncontained) == usize::MAX {
        self.uncontained.push(particle_id);
      }
    }

    for particle_id in children[0].contained.iter()
    .chain(children[1].contained.iter())
    .chain(children[2].contained.iter())
    .chain(children[3].contained.iter()) {
      self.contained.push(*particle_id);
    }

    for &particle_id in children[0].uncontained_triggers.iter()
    .chain(children[1].uncontained_triggers.iter())
    .chain(children[2].uncontained_triggers.iter())
    .chain(children[3].uncontained_triggers.iter()) {
      if index_in_uncontained(particle_id, &self.contained_triggers) != usize::MAX {
        particles[particle_id].uncontained = false;
      }
      else if index_in_uncontained(particle_id, &self.uncontained_triggers) == usize::MAX {
        self.uncontained_triggers.push(particle_id);
      }
    }

    for particle_id in children[0].contained_triggers.iter()
    .chain(children[1].contained_triggers.iter())
    .chain(children[2].contained_triggers.iter())
    .chain(children[3].contained_triggers.iter()) {
      self.contained_triggers.push(*particle_id);
    }
  }
}

impl TreeSplitable for VirtualRecord {
  fn tree_split(&mut self, _: &mut [&mut VirtualRecord]) {
  }
}

impl TreeJoinable for VirtualRecord {
  fn tree_join(&mut self, _: &[&VirtualRecord]) {
  }
}

fn index_in_uncontained(id: usize, uncontained: &Vec<usize>) -> usize {
  let mut index = usize::MAX;
  let mut i = uncontained.len();
  for &leafpid in uncontained.iter().rev() {
    i -= 1;
    if leafpid == id {
      index = i;
      break;
    }
  }
  index
}

fn remove_uncontained_from_leaf(id: usize, uncontained: &mut Vec<usize>) {
  let index = index_in_uncontained(id, uncontained);
  if index != usize::MAX {
    uncontained.swap_remove(index);
  }
}

fn add_uncontained_to_leaf(id: usize, uncontained: &mut Vec<usize>) {
  // let index = index_in_uncontained(id, uncontained);
  // if index == usize::MAX {
    uncontained.push(id);
  // }
}

impl VolumeRoot {
  pub fn new(bb: BB) -> VolumeRoot {
    let (result_tx, result_rx) = channel();
    let threads = num_cpus::get();
    // let threads = 1;

    let pool = ConsumerPool::new(result_tx, |thread_id, mut producer| {
      let mut removed = Vec::<usize>::new();
      let mut removed_triggers = Vec::<usize>::new();
      let mut deeper = Vec::<usize>::new();
      let mut deeper_triggers = Vec::<usize>::new();

      Consumer::new(move |job| {
        match job {
          VolumeJob::CopyBB { start, end, particles, old_new } => {
            let mut i = start;
            while i < end {
              unsafe {
                (&mut *old_new.offset(i as isize)).new =
                  (&*particles.offset(i as isize)).bbox;
              }
              i += 1;
            }
            Some(VolumeResult::BBDone)
          },
          VolumeJob::CopyOld { start, end, old_new } => {
            let mut i = start;
            while i < end {
              unsafe {
                (&mut *old_new.offset(i as isize)).old =
                  (&*old_new.offset(i as isize)).new;
              }
              i += 1;
            }
            Some(VolumeResult::BBDone)
          },
          VolumeJob::UpdateUncontained { mut done, old_new, volume, } => {
            if let Some(ref mut children) = unsafe { &mut *volume }.children {
              VolumeRoot::_update_leaves_uncontained(unsafe { &mut *volume }, unsafe { &*old_new });
              done += 1;
              let c = children.as_mut_ptr();
              for i in 0..4 {
                if children[i].children.is_none() {
                  continue;
                }
                producer.send(VolumeJob::UpdateUncontained {
                  done: done,
                  volume: unsafe { c.offset(i as isize) },
                  old_new: old_new,
                }).unwrap();
                done = 0;
              }
            }
            if done > 0 {
              producer.send_main(VolumeJob::UpdateUncontainedCompleted(done)).unwrap();
            }
            None
          },
          VolumeJob::MoveOutOfDate { mut done, volume, record, old_new } => {
            VolumeRoot::_update_remove_out_of_date_contained(thread_id, unsafe { &mut *volume }, unsafe { &mut *record }, unsafe { &mut *old_new }, &mut removed, &mut deeper, &mut removed_triggers, &mut deeper_triggers);
            done += 1;
            if let (&mut Some(ref mut volume_children), &mut Some(ref mut record_children)) = (&mut unsafe { &mut *volume }.children, &mut unsafe { &mut *record }.children) {
              let v = volume_children.as_mut_ptr();
              let r = record_children.as_mut_ptr();
              for i in 0..4 {
                producer.send(VolumeJob::MoveOutOfDate {
                  done: done,
                  volume: unsafe { v.offset(i as isize) },
                  record: unsafe { r.offset(i as isize) },
                  old_new: old_new,
                }).unwrap();
                done = 0;
              }
            }
            if done > 0 {
              producer.send_main(VolumeJob::MoveOutOfDateCompleted(done)).unwrap();
            }
            None
          },
          _ => { None },
        }
      })
    });

    let mut record_thread_vec = Vec::new();
    for _ in 0..threads {
      record_thread_vec.push(Vec::new());
    }

    VolumeRoot {
      root: QuadTree::new(VirtualVolume {
        bb: bb,
        center: bb.center(),
        .. Default::default()
      }),
      records: QuadTree::new(VirtualRecord {
        bb: bb,
        contained: record_thread_vec.clone(),
        contained_triggers: record_thread_vec.clone(),
        .. Default::default()
      }),
      changes: Vec::new(),
      bb_clone: Vec::new(),

      threads: threads,
      pool: pool,
      result_rx: result_rx,
    }
  }

  pub fn add(&mut self, particleid: usize) {
    self.changes.push(VolumeChange::Add(particleid));
  }

  pub fn remove(&mut self, particleid: usize) {
    self.changes.push(VolumeChange::Remove(particleid));
  }

  #[inline(never)]
  pub fn update(&mut self, particles: &mut Vec<Particle>) {
    while self.bb_clone.len() < particles.len() {
      self.bb_clone.push(Default::default());
    }
    while self.bb_clone.len() > particles.len() {
      self.bb_clone.pop();
    }
    let b = self.bb_clone.as_mut_ptr();
    let p = particles.as_ptr();
    let mut i = 0;
    let l = particles.len();
    let mut jobs_out = 0;
    while i < l {
      self.pool.send(VolumeJob::CopyBB {
        start: i,
        end: if i + PARTICLES_PER_JOB > l {l} else {i + PARTICLES_PER_JOB},
        particles: p,
        old_new: b,
      }).unwrap();
      jobs_out += 1;
      i += PARTICLES_PER_JOB;
    }
    self.pool.process_main();
    let mut jobs_in = 0;
    while jobs_in < jobs_out {
      match self.result_rx.recv().unwrap() {
        VolumeResult::BBDone => {},
        result => {
          panic!("Unexpected result {:?}", result);
        },
      }
      jobs_in += 1;
    }
    // while i < l {
    //   unsafe { (&mut *b.offset(i as isize)).new = (&*p.offset(i as isize)).bbox; }
    //   i += 1;
    // }

    self.update_apply_changes(particles);
    self.update_remove_out_of_date_contained(particles);
    self.update_add_newly_contained(particles);
    self.update_leaves_uncontained(particles);
    self.update_split_and_join(particles);

    i = 0;
    jobs_out = 0;
    while i < l {
      self.pool.send(VolumeJob::CopyOld {
        start: i,
        end: if i + PARTICLES_PER_JOB > l {l} else {i + PARTICLES_PER_JOB},
        old_new: b,
      }).unwrap();
      jobs_out += 1;
      i += PARTICLES_PER_JOB;
    }
    self.pool.process_main();
    jobs_in = 0;
    while jobs_in < jobs_out {
      match self.result_rx.recv().unwrap() {
        VolumeResult::BBDone => {},
        result => {
          panic!("Unexpected result {:?}", result);
        },
      }
      jobs_in += 1;
    }
    // while i < l {
    //   unsafe {
    //     let bsub = &mut *b.offset(i as isize);
    //     bsub.old = bsub.new;
    //   }
    //   i += 1;
    // }
  }

  fn update_apply_changes(&mut self, particles: &mut Vec<Particle>) {
    let evil = particles.as_mut_ptr();
    let mut bb_clones = unsafe { &mut *(&mut self.bb_clone as *mut Vec<OldNew>) as &mut Vec<OldNew> };
    for change in unsafe { &*(&self.changes as *const Vec<VolumeChange>) as &Vec<VolumeChange> }.iter() {
      match change {
        &VolumeChange::Add(particleid) => {
          while bb_clones.len() < particles.len() {
            bb_clones.push(Default::default());
          }
          let is_trigger = particles[particleid].is_trigger();
          let new_bb = particles[particleid].bbox;
          self.root.walk_contain_bb_search_mut(new_bb, &mut |vvolumebelow| {
            if is_trigger {
              vvolumebelow.value.contained_triggers.push(particleid);
            }
            else {
              vvolumebelow.value.contained.push(particleid);
            }
            unsafe { &mut *evil.offset(particleid as isize) }.uncontained = vvolumebelow.children.is_some();
            bb_clones[particleid] = OldNew {
              new: new_bb,
              old: BB::infinity(),
              trigger: is_trigger,
            };
          });
        },
        &VolumeChange::Remove(particleid) => {
          // println!("remove particle {} from bvt", particleid);
          let is_trigger = bb_clones[particleid].trigger;
          let old_bb = bb_clones[particleid].old;
          self.root.walk_contain_bb_search_mut(old_bb, &mut |vvolumebelow| {
            if is_trigger {
              remove_uncontained_from_leaf(particleid, &mut vvolumebelow.value.contained_triggers);
              vvolumebelow.walk_bb_mut(old_bb, &|vleaf| {
                remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained_triggers);
              });
            }
            else {
              remove_uncontained_from_leaf(particleid, &mut vvolumebelow.value.contained);
              vvolumebelow.walk_bb_mut(old_bb, &|vleaf| {
                remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained);
              });
            }
          });
        },
      }
    }
    self.changes.clear();
  }

  fn _update_remove_out_of_date_contained(thread: usize, vvolume: &mut QuadTree<VirtualVolume>, record: &mut QuadTree<VirtualRecord>, bb_clones: &mut Vec<OldNew>, removed: &mut Vec<usize>, deeper: &mut Vec<usize>, removed_triggers: &mut Vec<usize>, deeper_triggers: &mut Vec<usize>) {
    let bb = vvolume.value.bb;
    let branch = vvolume.children.is_some();
    vvolume.value.contained.retain(|&particleid| {
      let new_bb = bb_clones[particleid].new;
      if !bb.contains(new_bb) {
        removed.push(particleid);
        for vparent in RecordStack::new(record).iter_mut().skip(1) {
          if vparent.bb.contains(new_bb) {
            vparent.contained[thread].push(particleid);
            break;
          }
        }
        false
      }
      else if branch && bb.child_contains(&new_bb) {
        deeper.push(particleid);
        // let record = recordset.last_mut();
        record.value.contained[thread].push(particleid);
        false
      }
      else {
        true
      }
    });

    vvolume.value.contained_triggers.retain(|&particleid| {
      let new_bb = bb_clones[particleid].new;
      if !bb.contains(new_bb) {
        removed_triggers.push(particleid);
        for vparent in RecordStack::new(record).iter_mut().skip(1) {
          if vparent.bb.contains(new_bb) {
            vparent.contained_triggers[thread].push(particleid);
            break;
          }
        }
        false
      }
      else if branch && bb.child_contains(&new_bb) {
        deeper_triggers.push(particleid);
        // let record = record.last_mut();
        record.value.contained_triggers[thread].push(particleid);
        false
      }
      else {
        true
      }
    });

    if removed.len() > 0 {
      for &particleid in removed.iter() {
        let old_bb = bb_clones[particleid].old;
        vvolume.walk_bb_mut(old_bb, &|vleaf| {
          remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained);
        });
        bb_clones[particleid].old = BB::infinity();
      }
      removed.clear();
    }
    if deeper.len() > 0 {
      for &particleid in deeper.iter() {
        let OldNew { new: new_bb, old: old_bb, trigger: _ } = bb_clones[particleid];
        vvolume.walk_bb_mut(old_bb, &|vleaf| {
          if !vleaf.bb.overlaps(new_bb) || vleaf.bb.contains(new_bb) {
            remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained);
          }
        });
        bb_clones[particleid].old = if old_bb.overlaps(new_bb) {
          old_bb.intersect(new_bb)
        }
        else {
          BB::infinity()
        };
      }
      deeper.clear();
    }

    if removed_triggers.len() > 0 {
      for &particleid in removed_triggers.iter() {
        let old_bb = bb_clones[particleid].old;
        vvolume.walk_bb_mut(old_bb, &|vleaf| {
          remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained_triggers);
        });
        bb_clones[particleid].old = BB::infinity();
      }
      removed_triggers.clear();
    }
    if deeper_triggers.len() > 0 {
      for &particleid in deeper_triggers.iter() {
        let OldNew { new: new_bb, old: old_bb, trigger: _ } = bb_clones[particleid];
        vvolume.walk_bb_mut(old_bb, &|vleaf| {
          if !vleaf.bb.overlaps(new_bb) || vleaf.bb.contains(new_bb) {
            remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained_triggers);
          }
        });
        bb_clones[particleid].old = if old_bb.overlaps(new_bb) {
          old_bb.intersect(new_bb)
        }
        else {
          BB::infinity()
        };
      }
      deeper_triggers.clear();
    }
  }

  #[inline(never)]
  fn update_remove_out_of_date_contained(&mut self, _: &mut Vec<Particle>) {
    let mut bb_clones = unsafe { &mut *(&mut self.bb_clone as *mut Vec<OldNew>) as &mut Vec<OldNew> };

    let mut todo = 0;
    self.root.walk_rev_mut(|_| {todo += 1;});

    let mut update_completed = 0;

    self.pool.send(VolumeJob::MoveOutOfDate {
      done: 0,
      volume: &mut self.root,
      record: &mut self.records,
      old_new: &mut *bb_clones,
    }).unwrap();

    self.pool.process_until(|job| {
      match job {
        &VolumeJob::MoveOutOfDateCompleted(done) => {
          update_completed += done;
        },
        &VolumeJob::MoveOutOfDate { done: _, volume: _, record: _, old_new: _ } => {},
        job => {
          panic!("Unexpected job {:?}", job);
        },
      }
      update_completed < todo
    });
  }

  fn update_add_newly_contained(&mut self, particles: &mut Vec<Particle>) {
    self.walk_with_record_mut(|vvolume, records| {
      for contained in records.value.contained.iter_mut() {
        for particleid in contained.iter() {
          let evil = particles.as_mut_ptr();
          let new_bb = particles[*particleid].bbox;
          vvolume.walk_contain_bb_search_mut(new_bb, &mut |vvolumebelow| {
            vvolumebelow.value.contained.push(*particleid);
            unsafe { &mut *evil.offset(*particleid as isize) }.uncontained = vvolumebelow.children.is_some();
          });
        }
        contained.clear();
      }

      for contained_triggers in records.value.contained_triggers.iter_mut() {
        for particleid in contained_triggers.iter() {
          let evil = particles.as_mut_ptr();
          let new_bb = particles[*particleid].bbox;
          vvolume.walk_contain_bb_search_mut(new_bb, &mut |vvolumebelow| {
            vvolumebelow.value.contained_triggers.push(*particleid);
            unsafe { &mut *evil.offset(*particleid as isize) }.uncontained = vvolumebelow.children.is_some();
          });
        }
        contained_triggers.clear();
      }
    });
  }

  fn _update_leaves_uncontained(vvolume: &mut QuadTree<VirtualVolume>, bb_clones: &Vec<OldNew>) {
    if vvolume.children.is_none() {
      return;
    }
    let contained = unsafe { &*(&vvolume.value.contained as *const Vec<usize>) as &Vec<usize> };
    for &particleid in contained.iter() {
      let OldNew { new: new_bb, old: old_bb, trigger: _ } = bb_clones[particleid];
      let joined_bb = if old_bb.t == f32::INFINITY {
        new_bb
      }
      else {
        old_bb.join(new_bb)
      };
      vvolume.walk_bb_mut(joined_bb, &|vleaf| {
        if !vleaf.bb.overlaps(new_bb) {
          remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained);
        }
        else if !vleaf.bb.overlaps(old_bb) {
          add_uncontained_to_leaf(particleid, &mut vleaf.uncontained);
        }
      });
    }

    let contained_triggers = unsafe { &*(&vvolume.value.contained_triggers as *const Vec<usize>) as &Vec<usize> };
    for &particleid in contained_triggers.iter() {
      let OldNew { new: new_bb, old: old_bb, trigger: _ } = bb_clones[particleid];
      let joined_bb = if old_bb.t == f32::INFINITY {
        new_bb
      }
      else {
        old_bb.join(new_bb)
      };
      vvolume.walk_bb_mut(joined_bb, &|vleaf| {
        if !vleaf.bb.overlaps(new_bb) {
          remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained_triggers);
        }
        else if !vleaf.bb.overlaps(old_bb) {
          add_uncontained_to_leaf(particleid, &mut vleaf.uncontained_triggers);
        }
      });
    }
  }

  #[inline(never)]
  fn update_leaves_uncontained(&mut self, _: &mut Vec<Particle>) {
    let bb_clones = unsafe { &*(&self.bb_clone as *const Vec<OldNew>) as &Vec<OldNew> };

    let mut todo = 0;
    self.root.walk_rev_mut(|v| {if v.children.is_some() {todo += 1;}});

    let mut update_completed = 0;
    if todo > 0 {
      self.pool.send(VolumeJob::UpdateUncontained {
        done: 0,
        volume: &mut self.root,
        old_new: &*bb_clones,
      }).unwrap();

      self.pool.process_until(|job| {
        match job {
          &VolumeJob::UpdateUncontainedCompleted(done) => {
            update_completed += done;
          },
          &VolumeJob::UpdateUncontained { done: _, volume: _, old_new: _ } => {},
          job => {
            panic!("Unexpected job {:?}", job);
          },
        }
        update_completed < todo
      });
    }
  }

  fn update_split_and_join(&mut self, particles: &mut Vec<Particle>) {
    let threads = self.threads;
    let bb_clones = unsafe { &*(&self.bb_clone as *const Vec<OldNew>) as &Vec<OldNew> };
    self.root.walk_with_record_rev_mut(&mut self.records, |vvolume, record| {
    // for (vvolume, record) in self.root.iter_mut().rev()
    // .zip(self.records.iter_mut().rev()) {
      if vvolume.is_leaf() {
        let center = vvolume.value.center;
        if vvolume.value.len() >= MAX_LEAF_VOLUME &&
          // If all held particles overlap the center of this volume, every
          // child will contain every particle resulting in no improvement by
          // subdividing.
          !vvolume.value.contained.iter()
          .chain(vvolume.value.uncontained.iter())
          .chain(vvolume.value.contained_triggers.iter())
          .chain(vvolume.value.uncontained_triggers.iter())
          .map(|&particle_id| bb_clones[particle_id].new)
          .all(|bb| bb.overlaps_point(center)) {
          println!("split {}", vvolume.value.bb);
          vvolume.split_with(particles);
          record.split();
          for record_child in record.children.as_mut().unwrap().iter_mut() {
            for _ in 0..threads {
              record_child.value.contained.push(Vec::new());
              record_child.value.contained_triggers.push(Vec::new());
            }
          }
        }
        // assert!(vvolume.children.is_none() && vvolume.value.len() <= 256 || vvolume.children.is_some());
      }
      // let vvolume_contains = vvolume.value.contained.len();
      let should_join = match vvolume.children {
        Some(ref children) => {
          children.iter().fold(true, |carry, child| carry && child.is_leaf()) &&
          children.iter().fold(0, |carry, child| carry + child.value.len()) <= MIN_LEAF_VOLUME
        },
        _ => {false},
      };
      if should_join {
        println!("join {}", vvolume.value.bb);
        vvolume.join_with(particles);
        record.join();
      }
    });
  }

  fn walk_with_record_mut<F>(&mut self, mut handle: F) where F : FnMut(&mut QuadTree<VirtualVolume>, &mut QuadTree<VirtualRecord>) {
    self.root.walk_with_record_mut(&mut self.records, &mut handle);
  }

  pub fn iter_volumes<'a>(&'a mut self) -> iter::FilterMap<
    quad_tree::Iter<'a, VirtualVolume>,
    fn(& QuadTree<VirtualVolume>) -> Option<(&Vec<usize>, &Vec<usize>, &Vec<usize>, &Vec<usize>)>
  > {
    fn into_volume(vvolume: &QuadTree<VirtualVolume>) -> Option<(&Vec<usize>, &Vec<usize>, &Vec<usize>, &Vec<usize>)> {
      if vvolume.is_leaf() {
        Some((
          &vvolume.value.contained, &vvolume.value.uncontained,
          &vvolume.value.contained_triggers, &vvolume.value.uncontained_triggers,
        ))
      }
      else {
        None
      }
    }
    self.root.iter().filter_map(into_volume)
  }
}
