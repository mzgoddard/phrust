use std::usize;
use std::f32;
use std::any::Any;
use std::collections::LinkedList;
use std::iter;
use std::slice;

use super::math::BB;
use super::particle::Particle;
use super::quad_tree;
use super::quad_tree::{QuadTree, TreeSplitableWith, TreeJoinableWith, TreeSplitable, TreeJoinable};

const MAX_LEAF_VOLUME : usize = 256;
const MIN_LEAF_VOLUME : usize = 128;

struct Volume {
  bb: BB,
  len: usize,
  particles: [Particle; 256],
}

#[derive(Default)]
struct VirtualVolume {
  bb: BB,
  contained: Vec<usize>,
  uncontained: Vec<usize>,
}

#[derive(Default)]
struct VirtualRecord {
  bb: BB,
  contained: Vec<usize>,
}

enum VolumeChange {
  Add(usize),
  Remove(usize),
}

#[derive(Default)]
struct OldNew {
  new: BB,
  old: BB,
}

#[derive(Default)]
pub struct VolumeRoot {
  root: QuadTree<VirtualVolume>,
  records: QuadTree<VirtualRecord>,
  changes: Vec<VolumeChange>,
  bb_clone: Vec<OldNew>,
}

impl VirtualVolume {
  fn contains(&self, particle: &Particle) -> bool {
    self.bb.contains(&particle.bbox)
  }

  fn len(&self) -> usize {
    self.contained.len() + self.uncontained.len()
  }

  fn append_uncontained(&mut self, other: &VirtualVolume) {
    for &particle_id in other.uncontained.iter() {
      if index_in_uncontained(particle_id, &other.uncontained) == usize::MAX &&
        index_in_uncontained(particle_id, &other.contained) == usize::MAX
      {
        self.uncontained.push(particle_id);
      }
    }
  }
}

impl QuadTree<VirtualVolume> {
  fn walk_mut<F>(&mut self, mut handle: F) where F : FnMut(&mut QuadTree<VirtualVolume>) {
    self._walk_mut(&mut handle);
  }

  fn _walk_mut<F>(&mut self, handle: &mut F) where F : FnMut(&mut QuadTree<VirtualVolume>) {
    if let Some(ref mut children) = self.children {
      children[0]._walk_mut(handle);
      children[1]._walk_mut(handle);
      children[2]._walk_mut(handle);
      children[3]._walk_mut(handle);
    }
    handle(self);
  }

  fn walk_rev_mut<F>(&mut self, mut handle: F) where F : FnMut(&mut QuadTree<VirtualVolume>) {
    self._walk_rev_mut(&mut handle);
  }

  fn _walk_rev_mut<F>(&mut self, handle: &mut F) where F : FnMut(&mut QuadTree<VirtualVolume>) {
    handle(self);
    if let Some(ref mut children) = self.children {
      children[0]._walk_rev_mut(handle);
      children[1]._walk_rev_mut(handle);
      children[2]._walk_rev_mut(handle);
      children[3]._walk_rev_mut(handle);
    }
  }

  fn walk_bb_mut(&mut self, b: BB, handle: &Fn(&mut VirtualVolume)) {
    if let Some(ref mut children) = self.children {
      let cx = (self.value.bb.l + self.value.bb.r) / 2f32;
      let cy = (self.value.bb.b + self.value.bb.t) / 2f32;
      if cy < b.t {
        if cx > b.l {
          children[0].walk_bb_mut(b, handle);
        }
        if cx < b.r {
          children[1].walk_bb_mut(b, handle);
        }
      }
      if cy > b.b {
        if cx > b.l {
          children[2].walk_bb_mut(b, handle);
        }
        if cx < b.r {
          children[3].walk_bb_mut(b, handle);
        }
      }
    }
    else {
      handle(&mut self.value);
    }
  }

  fn walk_contain_bb_search_mut(&mut self, b: BB, handle: &Fn(&mut QuadTree<VirtualVolume>)) {
    let found = if let Some(ref mut children) = self.children {
      let cx = (self.value.bb.l + self.value.bb.r) / 2f32;
      let cy = (self.value.bb.b + self.value.bb.t) / 2f32;
      if cy <= b.b {
        if cx >= b.r {
          children[0].walk_contain_bb_search_mut(b, handle);
          false
        }
        else if cx <= b.l {
          children[1].walk_contain_bb_search_mut(b, handle);
          false
        }
        else {
          true
        }
      }
      else if cy >= b.t {
        if cx >= b.r {
          children[2].walk_contain_bb_search_mut(b, handle);
          false
        }
        else if cx <= b.l {
          children[3].walk_contain_bb_search_mut(b, handle);
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

  fn walk_with_record_stack_rev_mut<F>(&mut self, record: &mut QuadTree<VirtualRecord>, record_stack: &mut Vec<&mut VirtualRecord>, handle: &mut F) where F : FnMut(&mut QuadTree<VirtualVolume>, &mut Vec<&mut VirtualRecord>) {
    let l = record_stack.len();
    record_stack.push(unsafe { &mut *(&mut record.value as *mut VirtualRecord) as &mut VirtualRecord });
    // unsafe {
    //   if l == record_stack.capacity() {
    //     record_stack.reserve(l * 2);
    //   }
    //   record_stack.set_len(l + 1);
    //   *(*record_stack).as_mut_ptr().offset(l as isize) = &mut *(&mut record.value as *mut VirtualRecord) as &mut VirtualRecord;
    // }
    handle(self, record_stack);
    if let (&mut Some(ref mut volume_children), &mut Some(ref mut record_children)) = (&mut self.children, &mut record.children) {
      volume_children[0].walk_with_record_stack_rev_mut(&mut record_children[0], record_stack, handle);
      volume_children[1].walk_with_record_stack_rev_mut(&mut record_children[1], record_stack, handle);
      volume_children[2].walk_with_record_stack_rev_mut(&mut record_children[2], record_stack, handle);
      volume_children[3].walk_with_record_stack_rev_mut(&mut record_children[3], record_stack, handle);
    }
    unsafe {
      record_stack.set_len(l);
    }
    // record_stack.pop();
  }

  fn walk_with_record_mut<F>(&mut self, record: &mut QuadTree<VirtualRecord>, handle: &mut F) where F : FnMut(&mut QuadTree<VirtualVolume>, &mut QuadTree<VirtualRecord>) {
    if let Some(ref mut volume_children) = self.children {
      if let Some(ref mut record_children) = record.children {
        volume_children[0].walk_with_record_mut(&mut record_children[0], handle);
        volume_children[1].walk_with_record_mut(&mut record_children[1], handle);
        volume_children[2].walk_with_record_mut(&mut record_children[2], handle);
        volume_children[3].walk_with_record_mut(&mut record_children[3], handle);
      }
    }
    handle(self, record);
  }
}

impl TreeSplitableWith for VirtualVolume {
  fn tree_split_with(&mut self, children: &mut [&mut VirtualVolume], data: &mut Any) {
    let particles = data.downcast_mut::<Vec<Particle>>().unwrap();
    children[0].bb = self.bb.tl();
    children[1].bb = self.bb.tr();
    children[2].bb = self.bb.bl();
    children[3].bb = self.bb.br();

    self.contained.retain(|particle_id| {
      let particle = &mut particles[*particle_id];
      if children[0].bb.contains(&particle.bbox) {
        children[0].contained.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[1].bb.contains(&particle.bbox) {
        children[1].contained.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[2].bb.contains(&particle.bbox) {
        children[2].contained.push(*particle_id);
        particle.uncontained = false;
        false
      }
      else if children[3].bb.contains(&particle.bbox) {
        children[3].contained.push(*particle_id);
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
  }
}

impl TreeJoinableWith for VirtualVolume {
  fn tree_join_with(&mut self, children: &[&VirtualVolume], data: &mut Any) {
    let particles = data.downcast_mut::<Vec<Particle>>().unwrap();

    for particle_id in children[0].contained.iter() {
      self.contained.push(*particle_id);
    }
    for particle_id in children[1].contained.iter() {
      self.contained.push(*particle_id);
    }
    for particle_id in children[2].contained.iter() {
      self.contained.push(*particle_id);
    }
    for particle_id in children[3].contained.iter() {
      self.contained.push(*particle_id);
    }

    // for particle_id in children[0].uncontained.iter() {
    //   self.uncontained.push(*particle_id);
    // }
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
    // self.append_uncontained(children[0]);
    // self.append_uncontained(children[1]);
    // self.append_uncontained(children[2]);
    // self.append_uncontained(children[3]);
  }
}

impl VirtualRecord {
  fn contains(&self, particle: &Particle) -> bool {
    self.bb.contains(&particle.bbox)
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

fn is_leaf(node: &&mut QuadTree<VirtualVolume>) -> bool {
  node.is_leaf()
}

fn is_not_leaf(node: &&mut QuadTree<VirtualVolume>) -> bool {
  !node.is_leaf()
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
    VolumeRoot {
      root: QuadTree::new(VirtualVolume {
        bb: bb,
        .. Default::default()
      }),
      records: QuadTree::new(VirtualRecord {
        bb: bb,
        .. Default::default()
      }),
      .. Default::default()
    }
  }

  pub fn add(&mut self, particleid: usize) {
    self.changes.push(VolumeChange::Add(particleid));
  }

  pub fn remove(&mut self, particleid: usize) {
    self.changes.push(VolumeChange::Remove(particleid));
  }

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
    while i < l {
      unsafe { (&mut *b.offset(i as isize)).new = (&*p.offset(i as isize)).bbox; }
      i += 1;
    }
    // for (i, particle) in particles.iter().enumerate() {
    //   self.bb_clone[i].new = particle.bbox;
    // }

    self.update_apply_changes(particles);
    self.update_remove_out_of_date_contained(particles);
    self.update_add_newly_contained(particles);
    self.update_leaves_uncontained(particles);
    self.update_split_and_join(particles);

    i = 0;
    while i < l {
      unsafe {
        let bsub = &mut *b.offset(i as isize);
        bsub.old = bsub.new;
      }
      i += 1;
    }
    // for old_new in self.bb_clone.iter_mut() {
    //   old_new.old = old_new.new;
    // }
  }

  fn update_apply_changes(&mut self, particles: &mut Vec<Particle>) {
    for change in self.changes.iter() {
      match change {
        &VolumeChange::Add(particleid) => {
          // println!("add {} {}", particleid, particle.bbox);
          while self.bb_clone.len() < particles.len() {
            self.bb_clone.push(Default::default());
          }
          let particle = &mut particles[particleid];
          let mut uncontained = false;
          for vvolumebelow in self.root.iter_mut().rev().filter(|vvolume| vvolume.value.contains(particle)) {
            // println!("under? {}", vvolumebelow.value.bb);
            match vvolumebelow.children {
              Some(ref mut children) => {
                if
                  !children[0].value.contains(particle) ||
                  !children[1].value.contains(particle) ||
                  !children[2].value.contains(particle) ||
                  !children[3].value.contains(particle)
                {
                  // println!("add {} under {}", particleid, vvolumebelow.value.bb);
                  vvolumebelow.value.contained.push(particleid);
                  uncontained = true;
                  self.bb_clone[particleid] = OldNew {
                    new: particle.bbox,
                    old: BB::infinity(),
                  };
                  break;
                }
              },
              None => {
                // println!("add {} under {}", particleid, vvolumebelow.value.bb);
                vvolumebelow.value.contained.push(particleid);
                self.bb_clone[particleid] = OldNew {
                  new: particle.bbox,
                  old: BB::infinity(),
                };
                break;
              },
            }
          }
          particle.uncontained = uncontained;
        },
        &VolumeChange::Remove(particleid) => {
          // println!("remove {}", particleid);
          // let particle = &particles[particleid];
          let old_bbox = &self.bb_clone[particleid].old;
          for vleaf in self.root.iter_mut()
            .filter(|node| node.is_leaf())
            .filter(|node| node.value.bb.contains(old_bbox)) {
            let mut index = usize::MAX;
            for (i, leafpid) in vleaf.value.uncontained.iter().enumerate() {
              if leafpid == &particleid {
                index = i;
                break;
              }
            }
            if index != usize::MAX {
              vleaf.value.uncontained.swap_remove(index);
            }
          }
        },
      }
    }
    self.changes.clear();
  }

  fn update_remove_out_of_date_contained(&mut self, particles: &mut Vec<Particle>) {
    let mut removed = Vec::<usize>::new();
    let mut deeper = Vec::<usize>::new();
    let mut bb_clones = unsafe { &mut *(&mut self.bb_clone as *mut Vec<OldNew>) as &mut Vec<OldNew> };
    self.walk_with_record_stack_rev_mut(|vvolume, recordset| {
    // for (vvolume, recordset) in self.root.iter_mut().rev()
    // .zip(self.records.iter_stack_mut().rev()) {
      let bb = vvolume.value.bb;
      let branch = vvolume.children.is_some();
      vvolume.value.contained.retain(|&particleid| {
        // let particle = &particles[particleid];
        let new_bb = bb_clones[particleid].new;
        if !bb.contains(&new_bb) {
          removed.push(particleid);
          for vparent in recordset.iter_mut().rev().skip(1) {
            if vparent.bb.contains(&new_bb) {
              vparent.contained.push(particleid);
              break;
            }
          }
          false
        }
        else if branch && bb.child_contains(&new_bb) {
          // println!("contained in child");
          // removed.push(particleid);
          deeper.push(particleid);
          if let Some(record) = recordset.last_mut() {
            record.contained.push(particleid);
          }
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

          // let OldNew { new: new_bb, old: old_bb } = bb_clones[particleid];
          // if old_bb.overlaps(new_bb) {
          //   vvolume.walk_bb_mut(old_bb, &|vleaf| {
          //     if !vleaf.bb.overlaps(new_bb) {
          //       remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained);
          //     }
          //   });
          //   bb_clones[particleid].old = old_bb.intersect(new_bb);
          // }
          // else {
          //   vvolume.walk_bb_mut(old_bb, &|vleaf| {
          //     remove_uncontained_from_leaf(particleid, &mut vleaf.uncontained);
          //   });
          //   bb_clones[particleid].old = BB::infinity();
          // }
        }
        removed.clear();
      }
      if deeper.len() > 0 {
        for &particleid in deeper.iter() {
          let OldNew { new: new_bb, old: old_bb } = bb_clones[particleid];
          vvolume.walk_bb_mut(old_bb, &|vleaf| {
            if !vleaf.bb.overlaps(new_bb) || vleaf.bb.contains(&new_bb) {
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
    // }
    });
  }

  fn update_add_newly_contained(&mut self, particles: &mut Vec<Particle>) {
    self.walk_with_record_mut(|vvolume, records| {
    // for (records, vvolume) in self.records.iter().zip(self.root.iter_mut()) {
      for particleid in records.value.contained.iter() {
        // let particle = &mut particles[*particleid];
        let evil = particles.as_mut_ptr();
        let new_bb = particles[*particleid].bbox;
        let mut uncontained = false;
        vvolume.walk_contain_bb_search_mut(new_bb, &|vvolumebelow| {
        // for vvolumebelow in vvolume.iter_mut().rev().filter(|vvolume| vvolume.value.contains(particle)) {
          vvolumebelow.value.contained.push(*particleid);
          unsafe { &mut *evil.offset(*particleid as isize) }.uncontained = vvolumebelow.children.is_some();
          // match vvolumebelow.children {
          //   Some(_) => {
          //     if !vvolumebelow.value.bb.child_contains(&new_bb) {
          //       vvolumebelow.value.contained.push(*particleid);
          //       unsafe { &mut *evil.offset(*particleid as isize) }.uncontained = true;
          //       // unsafe { uncontained = true; }
          //       true
          //     }
          //     else {
          //       false
          //     }
          //   },
          //   None => {
          //     vvolumebelow.value.contained.push(*particleid);
          //     unsafe { &mut *evil.offset(*particleid as isize) }.uncontained = false;
          //     true
          //   },
          // }

          // if added_to_branch {
          //   for vleaf in vvolumebelow.iter_mut().filter(is_leaf) {
          //     if vleaf.value.bb.overlaps(particle.bbox) {
          //       add_uncontained_to_leaf(*particleid, &mut vleaf.value.uncontained);
          //     }
          //   }
          // }
        // }
        });
        // particles[*particleid].uncontained = uncontained;
      }
      records.value.contained.clear();
    // }
    });
    // for records in self.records.iter_mut() {
    //   records.value.contained.clear();
    // }
  }

  fn update_leaves_uncontained(&mut self, particles: &mut Vec<Particle>) {
    // let mut contained_copy = Vec::<usize>::new();
    let mut bb_clones = unsafe { &mut *(&mut self.bb_clone as *mut Vec<OldNew>) as &mut Vec<OldNew> };
    self.root.walk_rev_mut(|vvolume| {
      if vvolume.children.is_none() {
        return;
      }
    // for vvolume in self.root.iter_mut().rev().filter(is_not_leaf) {
      // contained_copy.clear();
      // for particleid in vvolume.value.contained.iter() {
      //   // contained_copy.push((*particleid, particles[*particleid].bbox, self.old_particle_bb[*particleid]));
      //   contained_copy.push(*particleid);
      // }
      // for vleaf in vvolume.iter_mut()
      // .filter(is_leaf) {
      //   for &(particleid, new_bb, old_bb) in contained_copy.iter() {
      //     if vleaf.value.bb.overlaps(old_bb) {
      //       if !vleaf.value.bb.overlaps(new_bb) {
      //         remove_uncontained_from_leaf(particleid, &mut vleaf.value.uncontained);
      //       }
      //     }
      //     else {
      //       if vleaf.value.bb.overlaps(new_bb) {
      //         add_uncontained_to_leaf(particleid, &mut vleaf.value.uncontained);
      //       }
      //     }
      //   }
      // }
      // for &(particleid, new_bb, old_bb) in contained_copy.iter() {
      // for &particleid in contained_copy.iter() {
      let contained = unsafe { &*(&vvolume.value.contained as *const Vec<usize>) as &Vec<usize> };
      for &particleid in contained.iter() {
        let OldNew { new: new_bb, old: old_bb } = bb_clones[particleid];
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
        // vvolume.walk_mut(&|node| node.bb.overlaps(joined_bb), &|vleaf| {
        //   if vleaf.is_leaf() {
        //     if !vleaf.value.bb.overlaps(new_bb) {
        //       remove_uncontained_from_leaf(particleid, &mut vleaf.value.uncontained);
        //     }
        //     else if !vleaf.value.bb.overlaps(old_bb) {
        //       add_uncontained_to_leaf(particleid, &mut vleaf.value.uncontained);
        //     }
        //   }
        // });
        // vvolume.walk_mut(&|node| node.bb.overlaps(new_bb), &|vleaf| {
        //   if vleaf.is_leaf() && !vleaf.value.bb.overlaps(old_bb) {
        //     add_uncontained_to_leaf(particleid, &mut vleaf.value.uncontained);
        //   }
        // });
        // for vleaf in vvolume.iter_filter_mut(&|node| node.value.bb.overlaps(old_bb))
        //   .filter(|node| !node.value.bb.overlaps(new_bb))
        //   .filter(is_leaf) {
        //   remove_uncontained_from_leaf(particleid, &mut vleaf.value.uncontained);
        // }
        // for vleaf in vvolume.iter_filter_mut(&|node| node.value.bb.overlaps(new_bb))
        //   .filter(|node| !node.value.bb.overlaps(old_bb))
        //   .filter(is_leaf) {
        //   add_uncontained_to_leaf(particleid, &mut vleaf.value.uncontained);
        // }
      }
    // }
    });
  }

  fn update_split_and_join(&mut self, particles: &mut Vec<Particle>) {
    for (vvolume, record) in self.root.iter_mut().rev()
    .zip(self.records.iter_mut().rev()) {
      // println!("{}", vvolume.value.bb);
      if vvolume.is_leaf() {
        if vvolume.value.len() >= MAX_LEAF_VOLUME {
          println!("split {}", vvolume.value.bb);
          vvolume.split_with(particles);
          // if let Some(ref children) = vvolume.children {
          //   for child in children.iter() {
          //     println!("{} {} {}", child.value.contained.len(), child.value.uncontained.len(), child.value.bb);
          //   }
          // }
          // println!("{} {}", vvolume.value.contained.len(), vvolume.value.bb);
          record.split();
        }
      }
      let vvolume_contains = vvolume.value.contained.len() * 2;
      let should_join = match vvolume.children {
        Some(ref children) => {
          children.iter().fold(true, |carry, child| carry && child.is_leaf()) &&
          children.iter().fold(0, |carry, child| carry + child.value.len()) - vvolume_contains <= MIN_LEAF_VOLUME
        },
        _ => {false},
      };
      if should_join {
        println!("join {}", vvolume.value.bb);
        vvolume.join_with(particles);
        record.join();
      }
    }
  }

  fn walk_with_record_stack_rev_mut<F>(&mut self, mut handle: F) where F : FnMut(&mut QuadTree<VirtualVolume>, &mut Vec<&mut VirtualRecord>) {
    self.root.walk_with_record_stack_rev_mut(&mut self.records, &mut Vec::<&mut VirtualRecord>::with_capacity(16), &mut handle);
  }

  fn walk_with_record_mut<F>(&mut self, mut handle: F) where F : FnMut(&mut QuadTree<VirtualVolume>, &mut QuadTree<VirtualRecord>) {
    self.root.walk_with_record_mut(&mut self.records, &mut handle);
  }

  pub fn iter_volumes<'a>(&'a mut self) -> iter::FilterMap<
    quad_tree::Iter<'a, VirtualVolume>,
    fn(& QuadTree<VirtualVolume>) -> Option<(&Vec<usize>, &Vec<usize>)>
  > {
    fn into_volume(vvolume: &QuadTree<VirtualVolume>) -> Option<(&Vec<usize>, &Vec<usize>)> {
      if vvolume.is_leaf() {
        // println!("{}", vvolume.value.bb);
        // let mut ids = Vec::<usize>::new();
        // ids.extend(&vvolume.value.contained);
        // ids.extend(&vvolume.value.uncontained);
        // ids.extend(vvolume.value.contained.iter().cloned());
        // ids.extend(vvolume.value.uncontained.iter().cloned());
        // for particleid in vvolume.value.contained.iter() {
        //   ids.push(*particleid);
        // }
        // for particleid in vvolume.value.uncontained.iter() {
        //   ids.push(*particleid);
        // }
        Some((&vvolume.value.contained, &vvolume.value.uncontained))
      }
      else {
        None
      }
    }
    self.root.iter().filter_map(into_volume)
  }
}
