#![feature(test)]

use std::f32::EPSILON;
// use std::num::Float;
use super::particle::Particle;
use super::collision::Collision;
use super::math::*;

// struct VolumeNode {
//   bb: BB,
//   children: Option<Box<[VirtualVolumeIds; 4]>>,
//   // in maintenance tracks particles moving between parents to reduce extra work
//   adds: [ParticleId; 256],
//   lastAdd: u32,
//   // a parent holds the particle
//   overlaps: [ParticleId; 256],
//   lastOverlap: u32,
//   // fully holds the particle
//   contains: Vec<ParticleId>,
//   containAdds: Vec<ParticleId>,
// }
//
// struct VolumeRoot {
//   top: VolumeNode,
//   meta: Vec<ParticleMeta>,
// }
//
// struct ParticleMeta {
//   // position particle was at last frame
//   old_position: V2,
// }
//
// struct ParticleCopy {
//   id: usize,
//   position: V2,
//   last_position: V2,
//   acceleration: V2,
//   radius: f32,
//   mass: f32,
//   friction: f32,
//   bb: BB,
// }
//
// struct ParticleBox {
//   id: usize,
//   bb: BB,
// }
//
// struct VolumeCopy {
//   overlaps: [ParticleCopy; 256],
// }
//
// struct VolumeNodeChildrenMutIter {
//   index: usize,
//   node: &mut VolumeNode,
// }
//
// impl Iterator for VolumeNodeChildrenMutIter {
//   type Item = &mut VolumeNode;
//
//   fn next(&mut self) -> Option<Self::Item> {
//     if (self.index < 4) {
//       Some(&mut self.children.unwrap()[self.index].borrow_mut())
//     }
//     None
//   }
// }
//
// type VolumeNodeIter = Iterator<&mut VolumeNode>;
// type VolumeNodeIterVec = Vec<VolumeNodeIter>;
//
// struct VolumeNodeWalkIter {
//   stack: VolumeNodeIterVec,
// }
//
// impl VolumeNodeWalkIter {
//   fn new() -> VolumeNodeWalkIter {
//     VolumeNodeWalkIter {
//       stack: VolumeNodeIterVec::new(),
//     }
//   }
// }
//
// impl Iterator for VolumeNodeWalkUpMutIter {
//   type Item = &mut VolumeNode;
//
//   fn next(&mut self) -> Option<Self::Item> {
//     if (self.stack.len()) {
//     }
//   }
// }
//
// impl VolumeNode {
//   fn split(self) {
//   }
//
//   fn join(self) {
//   }
//
//   fn apply_containees(self) {
//   }
//
//   fn apply_adds(self) {
//   }
//
//   fn add_to_smallest_container(self, partilceId: usize, bb: BB) {
//   }
//
//   fn has_children(self) -> bool {
//     self.children.is_some()
//   }
//
//   fn has_deep_children(self) -> bool {
//     match (self.children) {
//       Some(children) => children.borrow()[0].has_children(),
//       None => false
//     }
//   }
//
//   fn children_mut(self) -> IterMut<VolumeNode> {
//     self.children.iter_mut()
//   }
//
//   fn walk_up_mut(self) -> IterMut<VolumeNode> {
//     let stack = Vec::<&mut VolumeNode, IterMut<VolumeNode>>::new();
//     let maybeNode = Some(self);
//     while (match (maybeNode) {
//       Some(node) => true,
//       None => false,
//     }) {
//       let iter = node.children_mut();
//       maybeNode = iter.next();
//       stack.push((node, iterx));
//     }
//   }
//
//   fn walk_down_mut(self) -> IterMut<VolumeNode> {
//
//   }
//
//   fn nodes_mut(self) -> IterMut<VolumeNode> {
//     self.walk_down_mut().filter(|&mut node| node.has_children())
//   }
//
//   fn leaves_mut(self) -> IterMut<VolumeNode> {
//     self.walk_down_mut().filter(|&mut node| !node.has_children())
//   }
// }
//
// impl VolumeRoot {
//   fn update(self, particles: &Vec<Particle>) {
//     // determine particles moving to different containing nodes
//     let uncontained = Vec::<ParticleBox>::new();
//     for node in self.top.walk_up() {
//       uncontained.retain(|&mut particle_box| {
//         if (node.bb.contains(particle_box.bb)) {
//           node.add_to_smallest_container(particle_id, bb);
//           false
//         }
//         true
//       });
//       for particle_id in node.contains.iter() {
//         let bb = particles[particle_id].bb;
//         let old_bb = self.meta[particle_id].old_bb;
//         for leaf in node.leaves().filter(|&leaf| leaf.bb.overlaps(old_bb) && !leaf.bb.overlaps(particle.bb)) {
//           leaf.remove(particle);
//         }
//         if (node.bb.contains(bb)) {
//           node.add_to_smallest_container(particle_id, bb);
//         }
//         else {
//           uncontained.push(ParticleBox {id: particle_id, bb: bb});
//         }
//       }
//     }
//     // apply changes
//     for node in self.top.nodes_mut() {
//       node.apply_containees();
//     }
//     // from containing node
//     for node in self.top.nodes() {
//       for particle in node.iter_particles() {
//         // determine leaves that overlap particles that didn't before
//         let old_bb = self.meta[particle_id].old_bb;
//         for leaf in node.leaves().filter(|leaf| leaf.bb.overlaps(particle.bb) && !leaf.bb.overlaps(old_bb)) {
//           leaf.add(particle);
//         }
//           // if child has too many new particles, split it
//         // update meta for particle
//         self.meta[particle_id].old_bb = particle.bb;
//       }
//     }
//     // check leaves
//     for node in self.top.leaves() {
//       node.apply_adds();
//       // if too many new particles and old particles, split it
//       // else update overlaps with new particles
//     }
//     // merge leaves with too few overlaps
//     for node in self.top.walk_down_mut().filter(|node| match (node.children) { Some(children) => children.borrow()[0].children.is_none(), None => false}) {
//       // if too particles, join children
//     }
//   }
//
//   fn remove(self, particle: &Particle) {
//   }
//
//   fn add(self, particle: &Particle) {
//   }
// }

#[cfg(bench)]
mod bench {
  extern crate test;
  use std::f32::EPSILON;
  // use std::num::Float;
  use self::test::Bencher;
  use particle::Particle;
  use collision::Collision;
  use math::V2;
  use std::collections::vec_deque::VecDeque;
  use std::rc::Rc;
  use std::cell::{Ref, RefMut, RefCell};
  use std::sync::{Arc, Mutex};
  use std::sync::mpsc::{channel, Sender};
  use std::thread;
  extern crate num_cpus;
  // use self::num_cpus;
  // use std::os::cpu_count;
  use std::default::Default;

  struct VirtualDdvt {
    particles: [Particle; 256],
    collisions: Vec<Collision>,
    last_col: usize,
  }

  struct VirtualRingDdvt {
    particles: [Particle; 256],
    collisions: VecDeque<VecDeque<Collision>>,
  }

  #[derive(Copy, Clone)]
  struct FutureParticle {
    position: V2,
    last_position: V2,
    count: usize,
  }

  impl Default for FutureParticle {
    fn default() -> FutureParticle {
      FutureParticle { position: V2::zero(), last_position: V2::zero(), count: 0 }
    }
  }

  struct VirtualMergeDdvt {
    particles: [Particle; 256],
    future_positions: [FutureParticle; 256],
  }

  struct RefDdvt {
    particles: Vec<Rc<RefCell<Particle>>>,
    collisions: Vec<Collision>,
  }

  // struct PtrDdvt {
  //   particles: Vec<Box<Particle>>,
  //   collisions: Vec<Collision>,
  // }

  fn init_virtual_ddvts(ddvts: &mut Vec<VirtualDdvt>) {
    for d in 0..ddvts.capacity() {
      ddvts.push(VirtualDdvt {
        particles: [Particle { .. Default::default() }; 256],
        collisions: Vec::<Collision>::with_capacity(2048),
        last_col: 0,
      });
      for i in 0..256 {
        ddvts[d].particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
      }
    }
  }

  fn init_ref_ddvts(ddvts: &mut Vec<RefDdvt>) {
    for d in 0..ddvts.capacity() {
      ddvts.push(RefDdvt {
        particles: Vec::<Rc<RefCell<Particle>>>::with_capacity(256),
        collisions: Vec::<Collision>::with_capacity(256),
      });
      for i in 0..256 {
        if d % 4 == 0 || i > 16 && i % 16 != 0 {
          ddvts[d].particles.push(Rc::new(RefCell::new(Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 }))));
        } else {
          let particle = ddvts[d / 4 * 4].particles[i].clone();
          ddvts[d].particles.push(particle);
        }
      }
    }
  }

  fn init_virtual_ring_ddvts(ddvts: &mut Vec<VirtualRingDdvt>) {
    for d in 0..ddvts.capacity() {
      ddvts.push(VirtualRingDdvt {
        particles: [Particle { .. Default::default() }; 256],
        collisions: VecDeque::<VecDeque<Collision>>::with_capacity(256),
      });
      for i in 0..256 {
        ddvts[d].particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
      }
    }
  }

  fn init_virtual_future_ddvts(ddvts: &mut Vec<VirtualMergeDdvt>) {
    for d in 0..ddvts.capacity() {
      ddvts.push(VirtualMergeDdvt {
        particles: [Particle { .. Default::default() }; 256],
        future_positions: [Default::default(); 256],
      });
      for i in 0..256 {
        ddvts[d].particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
      }
    }
  }

  fn init_mutex_virtual_ddvts(ddvts: &mut Vec<Arc<Mutex<VirtualDdvt>>>) {
    for d in 0..ddvts.capacity() {
      ddvts.push(Arc::new(Mutex::new(VirtualDdvt {
        particles: [Particle { .. Default::default() }; 256],
        collisions: Vec::<Collision>::with_capacity(0),
        last_col: 0,
      })));
      let mut ddvt = ddvts[d].lock().unwrap();
      for i in 0..256 {
        ddvt.particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
      }
    }
  }

  fn init_boxed_virtual_ddvts(ddvts: &mut Vec<Option<Box<VirtualDdvt>>>) {
    for d in 0..ddvts.capacity() {
      ddvts.push(Some(Box::new(VirtualDdvt {
        particles: [Particle { .. Default::default() }; 256],
        collisions: Vec::<Collision>::with_capacity(0),
        last_col: 0,
      })));
      let mut ddvt = ddvts[d].as_mut().unwrap();
      for i in 0..256 {
        ddvt.particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
      }
    }
  }

  fn reset_virtual_ddvts(ddvts: &mut Vec<VirtualDdvt>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.capacity() {
      for i in 0..256 {
        let p = V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 };
        ddvts[d].particles[i].position = p;
        ddvts[d].particles[i].last_position = p;
        // ddvts[d].particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
        n += 1;
      }
    }
    n
  }

  fn reset_ref_ddvts(ddvts: &mut Vec<RefDdvt>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.capacity() {
      let ddvt = &mut ddvts[d];
      let particles = &mut ddvt.particles;
      for i in 0..256 {
        let mut particle = particles[i].borrow_mut();
        // let particleRef = particles[i].clone();
        let p = V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 };
        particle.position = p;
        particle.last_position = p;
        // *particle = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
        n += 1;
      }
    }
    n
  }

  fn reset_virtual_future_ddvts(ddvts: &mut Vec<VirtualMergeDdvt>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.capacity() {
      for i in 0..256 {
        let p = V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 };
        ddvts[d].particles[i].position = p;
        ddvts[d].particles[i].last_position = p;
        // ddvts[d].particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
        n += 1;
      }
    }
    n
  }

  fn reset_mutex_virtual_ddvts(ddvts: &mut Vec<Arc<Mutex<VirtualDdvt>>>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.capacity() {
      let mut ddvt = ddvts[d].lock().unwrap();
      for i in 0..256 {
        let p = V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 };
        ddvt.particles[i].position = p;
        ddvt.particles[i].last_position = p;
        // ddvts[d].particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
        n += 1;
      }
    }
    n
  }

  fn reset_boxed_virtual_ddvts(ddvts: &mut Vec<Option<Box<VirtualDdvt>>>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.capacity() {
      let mut ddvt = ddvts[d].as_mut().unwrap();
      for i in 0..256 {
        let p = V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 };
        ddvt.particles[i].position = p;
        ddvt.particles[i].last_position = p;
        // ddvts[d].particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
        n += 1;
      }
    }
    n
  }

  fn reset_virtual_ring_ddvts(ddvts: &mut Vec<VirtualRingDdvt>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.capacity() {
      for i in 0..256 {
        ddvts[d].particles[i] = Particle::at(V2 { x: (i as f32) % 16f32, y: (i as f32) / 16f32 });
        n += 1;
      }
    }
    n
  }

  fn test_virtual_ddvts(ddvts: &mut Vec<VirtualDdvt>) -> usize {
    let mut col_index = 0;
    for d in 0..ddvts.len() {
      col_index = 0;
      let mut ddvt = &mut ddvts[d];
      let mut particles = &mut ddvt.particles;
      for i in 0..255 {
        let particle_i = &particles[i];
        while col_index - i + 256 >= ddvt.collisions.len() {
          ddvt.collisions.push(Collision { .. Default::default() });
        }
        for j in (i + 1)..256 {
          // if col_index >= ddvt.collisions.len() {
          //   ddvt.collisions.push(Collision { .. Default::default() });
          // }
          if ddvt.collisions[col_index].test(particle_i, &particles[j], i as u32, j as u32) {
            col_index += 1;
          }
        }
      }
      if col_index >= ddvt.collisions.len() {
        ddvt.collisions.push(Collision { .. Default::default() });
      }
      ddvt.collisions[col_index] = Collision { .. Default:: default() };
      ddvt.last_col = col_index;
    }
    col_index
  }

  fn ref_particles<'a, 'b>(particles: &'a mut Vec<Rc<RefCell<Particle>>>, borrowed_particles: &'b mut Vec<Ref<'a, Particle>>) {
    for i in 0..256 {
      borrowed_particles.push(particles[i].borrow::<'a>());
    }
  }

  fn ref_mut_particles<'a, 'b>(particles: &'a mut Vec<Rc<RefCell<Particle>>>, borrowed_particles: &'b mut Vec<RefMut<'a, Particle>>) {
    for i in 0..256 {
      borrowed_particles.push(particles[i].borrow_mut::<'a>());
    }
  }

  fn test_ref_ddvt_single<'a>(ddvt: &'a mut RefDdvt) -> usize {
    let mut col_index = 0;
    let mut borrowed_particles = Vec::<Ref<'a, Particle>>::with_capacity(256);
    ref_particles(&mut ddvt.particles, &mut borrowed_particles);

    for i in 0..255 {
      for j in (i + 1)..256 {
        if col_index >= ddvt.collisions.len() {
          ddvt.collisions.push(Collision { .. Default::default() });
        }
        if ddvt.collisions[col_index].test(&*borrowed_particles[i], &*borrowed_particles[j], i as u32, j as u32) {
          col_index += 1;
        }
      }
    }

    if col_index >= ddvt.collisions.len() {
      ddvt.collisions.push(Collision { .. Default::default() });
    }
    ddvt.collisions[col_index] = Collision { .. Default::default() };
    col_index
  }

  fn test_ref_ddvts(ddvts: &mut Vec<RefDdvt>) -> usize {
    let mut col_index = 0;
    for d in 0..ddvts.len() {
      col_index = test_ref_ddvt_single(&mut ddvts[d]);
    }
    col_index
  }

  fn solve_virtual_ddvts(ddvts: &mut Vec<VirtualDdvt>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.len() {
      let mut ddvt = &mut ddvts[d];
      let mut particles = &mut ddvt.particles;
      for collision in ddvt.collisions.iter().take(ddvt.last_col) {
        let a_index = collision.a_ddvt_index as usize;
        let b_index = collision.b_ddvt_index as usize;

        let (ap, alp, bp, blp) = collision.solve(&particles[a_index], &particles[b_index]);
        {
          let particle_a = &mut particles[a_index];
          particle_a.position = ap;
          particle_a.last_position = alp;
        }
        {
          let particle_b = &mut particles[b_index];
          particle_b.position = bp;
          particle_b.last_position = blp;
        }
        n += 1;
      }
    }
    n
  }

  fn solve_ref_ddvt_single<'a>(ddvt: &'a mut RefDdvt) -> usize {
    let mut n = 0;
    let mut particles = Vec::<RefMut<'a, Particle>>::with_capacity(256);
    ref_mut_particles(&mut ddvt.particles, &mut particles);
    for i in 0..ddvt.collisions.len() {
      let a_index = ddvt.collisions[i].a_ddvt_index as usize;
      let b_index = ddvt.collisions[i].b_ddvt_index as usize;
      if a_index == b_index { break; }

      let (ap, alp, bp, blp) = ddvt.collisions[i].solve(&*particles[a_index], &*particles[b_index]);
      particles[a_index].position = ap;
      particles[a_index].last_position = alp;
      particles[b_index].position = bp;
      particles[b_index].last_position = blp;
      n += 1;
    }
    n
  }

  fn solve_ref_ddvts(ddvts: &mut Vec<RefDdvt>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.len() {
      n += solve_ref_ddvt_single(&mut ddvts[d]);
    }
    n
  }

  fn merge_virtual_ddvts(ddvts: &mut Vec<VirtualDdvt>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.len() {
      if d % 4 != 0 { continue; }
      for i in 0..16 {
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        for k in 0..4 {
          sum = sum + ddvts[d + k].particles[i].position;
          lpsum = lpsum + ddvts[d + k].particles[i].last_position;
        }
        sum = sum.scale(0.25f32);
        lpsum = lpsum.scale(0.25f32);
        for k in 0..4 {
          ddvts[d + k].particles[i].position = sum;
          ddvts[d + k].particles[i].last_position = lpsum;
          n += 1;
        }
      }
      for i in 16..256 {
        if i % 16 != 0 { continue; }
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        for k in 0..4 {
          sum = sum + ddvts[d + k].particles[i].position;
          lpsum = lpsum + ddvts[d + k].particles[i].last_position;
        }
        sum = sum.scale(0.25f32);
        lpsum = lpsum.scale(0.25f32);
        for k in 0..4 {
          ddvts[d + k].particles[i].position = sum;
          ddvts[d + k].particles[i].last_position = lpsum;
          n += 1;
        }
      }
    }
    n
  }

  // Theory for smaller runtime memory footprint for collision objects.
  // Test collisions for current a index.
  // Solve b side of collisions before the col_index if index b is the current a index.
  // Solve a side of collisions collisions for a index.
  fn test_solve_virtual_ddvts(ddvts: &mut Vec<VirtualDdvt>) -> usize {
    let mut col_index = 0;
    for d in 0..ddvts.len() {
      col_index = 0;
      let mut ddvt = &mut ddvts[d];
      let mut particles = &mut ddvt.particles;
      for i in 0..255 {
        let particle_i = &particles[i];
        while col_index - i + 256 >= ddvt.collisions.len() {
          ddvt.collisions.push(Collision { .. Default::default() });
        }
        for j in (i + 1)..256 {
          // if col_index >= ddvt.collisions.len() {
          //   ddvt.collisions.push(Collision { .. Default::default() });
          // }
          if ddvt.collisions[col_index].test(particle_i, &particles[j], i as u32, j as u32) {
            col_index += 1;
          }
        }
      }

      if col_index >= ddvt.collisions.len() {
        ddvt.collisions.push(Collision { .. Default::default() });
      }
      ddvt.collisions[col_index] = Collision { .. Default:: default() };

      for col in ddvt.collisions.iter().take(col_index) {
        let a_index = col.a_ddvt_index as usize;
        let b_index = col.b_ddvt_index as usize;
        let (ap, alp, bp, blp) = col.solve(&particles[a_index], &particles[b_index]);
        {
          let mut_particle = &mut particles[a_index];
          mut_particle.position = ap;
          mut_particle.last_position = alp;
        }
        {
          let mut_particle = &mut particles[b_index];
          mut_particle.position = bp;
          mut_particle.last_position = blp;
        }
      }
    }
    col_index
  }

  fn new_mutex_virtual_ddvt_test_pool(size: usize) -> Sender<(Arc<Mutex<VirtualDdvt>>, Sender<usize>)> {
    let (sender, rx) = channel::<(Arc<Mutex<VirtualDdvt>>, Sender<usize>)>();
    let rx_arc = Arc::new(Mutex::new(rx));
    for _ in 0..(size) {
      let rx_arc = rx_arc.clone();
      thread::spawn(move || {
        let mut collisions = Vec::<Collision>::with_capacity(33000);
        for i in 0..33000 {
          collisions.push(Default::default());
        }
        loop {
          let (ddvt_arc, tresponse) = {
            let rx = rx_arc.lock().unwrap();
            match rx.recv() {
              Ok(job) => job,
              Err(..) => break
            }
          };

          let mut ddvt: &mut VirtualDdvt = &mut *ddvt_arc.lock().unwrap();
          let mut particles = &mut ddvt.particles;
          let mut col_index = 0;
          for i in 0..255 {
            let particle_i = &particles[i];
            // while col_index - i + 256 >= collisions.len() {
            //   collisions.push(Collision { .. Default::default() });
            // }
            for j in (i + 1)..256 {
              // if col_index >= ddvt.collisions.len() {
              //   ddvt.collisions.push(Collision { .. Default::default() });
              // }
              if collisions[col_index].test(particle_i, &particles[j], i as u32, j as u32) {
                col_index += 1;
              }
            }
          }
          // if col_index >= collisions.len() {
          //   collisions.push(Collision { .. Default::default() });
          // }
          // collisions[col_index] = Collision { .. Default:: default() };
          // ddvt.last_col = col_index;

          for collision in collisions.iter().take(col_index) {
            let a_index = collision.a_ddvt_index as usize;
            let b_index = collision.b_ddvt_index as usize;

            let (ap, alp, bp, blp) = collision.solve(&particles[a_index], &particles[b_index]);
            {
              let particle_a = &mut particles[a_index];
              particle_a.position = ap;
              particle_a.last_position = alp;
            }
            {
              let particle_b = &mut particles[b_index];
              particle_b.position = bp;
              particle_b.last_position = blp;
            }
          }

          tresponse.send(1);
        }
      });
    }
    sender
  }

  fn test_solve_mutex_virtual_ddvts(ddvts: &mut Vec<Arc<Mutex<VirtualDdvt>>>, pool: &mut Sender<(Arc<Mutex<VirtualDdvt>>, Sender<usize>)>) -> usize {
    let (tresponse, rresponse) = channel::<usize>();
    let mut responses = 0;
    for d in 0..ddvts.len() {
      let ddvt_arc: Arc<Mutex<VirtualDdvt>> = ddvts[d].clone();
      let tresponse: Sender<usize> = tresponse.clone();
      pool.send((ddvt_arc, tresponse));
    }

    loop {
      if responses == 32 {
        break;
      }
      rresponse.recv();
      responses += 1;
    }
    responses
  }

  fn new_boxed_virtual_ddvt_test_pool(size: usize) -> Sender<(Box<VirtualDdvt>, Sender<(Box<VirtualDdvt>, usize)>, usize)> {
    let (sender, rx) = channel::<(Box<VirtualDdvt>, Sender<(Box<VirtualDdvt>, usize)>, usize)>();
    let rx_arc = Arc::new(Mutex::new(rx));
    for _ in 0..(size) {
      let rx_arc = rx_arc.clone();
      thread::spawn(move || {
        let mut collisions = Vec::<Collision>::with_capacity(33000);
        for i in 0..33000 {
          collisions.push(Default::default());
        }
        loop {
          let (mut ddvt_box, tresponse, index) = {
            let rx = rx_arc.lock().unwrap();
            match rx.recv() {
              Ok(job) => job,
              Err(..) => break
            }
          };

          {
            let mut ddvt: &mut VirtualDdvt = &mut *ddvt_box;
            let mut particles = &mut ddvt.particles;
            let mut col_index = 0;
            for i in 0..255 {
              let particle_i = &particles[i];
              // while col_index - i + 256 >= collisions.len() {
              //   collisions.push(Collision { .. Default::default() });
              // }
              for j in (i + 1)..256 {
                // if col_index >= ddvt.collisions.len() {
                //   ddvt.collisions.push(Collision { .. Default::default() });
                // }
                if collisions[col_index].test(particle_i, &particles[j], i as u32, j as u32) {
                  col_index += 1;
                }
              }
            }
            // if col_index >= collisions.len() {
            //   collisions.push(Collision { .. Default::default() });
            // }
            // collisions[col_index] = Collision { .. Default:: default() };
            // ddvt.last_col = col_index;

            for collision in collisions.iter().take(col_index) {
              let a_index = collision.a_ddvt_index as usize;
              let b_index = collision.b_ddvt_index as usize;

              let (ap, alp, bp, blp) = collision.solve(&particles[a_index], &particles[b_index]);
              {
                let particle_a = &mut particles[a_index];
                particle_a.position = ap;
                particle_a.last_position = alp;
              }
              {
                let particle_b = &mut particles[b_index];
                particle_b.position = bp;
                particle_b.last_position = blp;
              }
            }
          }

          tresponse.send((ddvt_box, index));
        }
      });
    }
    sender
  }

  fn test_solve_boxed_virtual_ddvts(ddvts: &mut Vec<Option<Box<VirtualDdvt>>>, pool: &mut Sender<(Box<VirtualDdvt>, Sender<(Box<VirtualDdvt>, usize)>, usize)>) -> usize {
    let (tresponse, rresponse) = channel::<(Box<VirtualDdvt>, usize)>();
    let mut responses = 0;
    for d in 0..ddvts.len() {
      let ddvt_box: Box<VirtualDdvt> = ddvts[d].take().unwrap();
      let tresponse: Sender<(Box<VirtualDdvt>, usize)> = tresponse.clone();
      pool.send((ddvt_box, tresponse, d));
    }

    loop {
      if responses == 32 {
        break;
      }
      let (ddvt_box, d) = rresponse.recv().unwrap();
      ddvts[d] = Some(ddvt_box);
      responses += 1;
    }
    responses
  }

  fn merge_boxed_virtual_ddvts(ddvts: &mut Vec<Option<Box<VirtualDdvt>>>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.len() {
      if d % 4 != 0 { continue; }
      for i in 0..16 {
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        for k in 0..4 {
          let ddvt: &VirtualDdvt = &**ddvts[d + k].as_ref().unwrap();
          sum = sum + ddvt.particles[i].position;
          lpsum = lpsum + ddvt.particles[i].last_position;
        }
        sum = sum.scale(0.25f32);
        lpsum = lpsum.scale(0.25f32);
        for k in 0..4 {
          let mut ddvt = &mut **ddvts[d + k].as_mut().unwrap();
          ddvt.particles[i].position = sum;
          ddvt.particles[i].last_position = lpsum;
          n += 1;
        }
      }
      for i in 16..256 {
        if i % 16 != 0 { continue; }
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        for k in 0..4 {
          let ddvt = &**ddvts[d + k].as_ref().unwrap();
          sum = sum + ddvt.particles[i].position;
          lpsum = lpsum + ddvt.particles[i].last_position;
        }
        sum = sum.scale(0.25f32);
        lpsum = lpsum.scale(0.25f32);
        for k in 0..4 {
          let mut ddvt = &mut **ddvts[d + k].as_mut().unwrap();
          ddvt.particles[i].position = sum;
          ddvt.particles[i].last_position = lpsum;
          n += 1;
        }
      }
    }
    n
  }

  fn merge_mutex_virtual_ddvts(ddvts: &mut Vec<Arc<Mutex<VirtualDdvt>>>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.len() {
      if d % 4 != 0 { continue; }
      for i in 0..16 {
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        for k in 0..4 {
          let mut ddvt = ddvts[d + k].lock().unwrap();
          sum = sum + ddvt.particles[i].position;
          lpsum = lpsum + ddvt.particles[i].last_position;
        }
        sum = sum.scale(0.25f32);
        lpsum = lpsum.scale(0.25f32);
        for k in 0..4 {
          let mut ddvt = ddvts[d + k].lock().unwrap();
          ddvt.particles[i].position = sum;
          ddvt.particles[i].last_position = lpsum;
          n += 1;
        }
      }
      for i in 16..256 {
        if i % 16 != 0 { continue; }
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        for k in 0..4 {
          let mut ddvt = ddvts[d + k].lock().unwrap();
          sum = sum + ddvt.particles[i].position;
          lpsum = lpsum + ddvt.particles[i].last_position;
        }
        sum = sum.scale(0.25f32);
        lpsum = lpsum.scale(0.25f32);
        for k in 0..4 {
          let mut ddvt = ddvts[d + k].lock().unwrap();
          ddvt.particles[i].position = sum;
          ddvt.particles[i].last_position = lpsum;
          n += 1;
        }
      }
    }
    n
  }

  fn merge_virtual_future_ddvts(ddvts: &mut Vec<VirtualMergeDdvt>) -> usize {
    let mut n = 0;
    for d in 0..ddvts.len() {
      if d % 4 != 0 { continue; }
      for i in 0..16 {
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        for k in 0..4 {
          sum = sum + ddvts[d + k].particles[i].position;
          lpsum = lpsum + ddvts[d + k].particles[i].last_position;
        }
        sum = sum.scale(0.25f32);
        lpsum = lpsum.scale(0.25f32);
        for k in 0..4 {
          ddvts[d + k].particles[i].position = sum;
          ddvts[d + k].particles[i].last_position = lpsum;
          n += 1;
        }
      }
      for i in 16..256 {
        if i % 16 != 0 { continue; }
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        for k in 0..4 {
          sum = sum + ddvts[d + k].particles[i].position;
          lpsum = lpsum + ddvts[d + k].particles[i].last_position;
        }
        sum = sum.scale(0.25f32);
        lpsum = lpsum.scale(0.25f32);
        for k in 0..4 {
          ddvts[d + k].particles[i].position = sum;
          ddvts[d + k].particles[i].last_position = lpsum;
          n += 1;
        }
      }
    }
    n
  }

  pub fn col_solve_a(col: &mut Collision, a: &Particle, b: &Particle) -> (V2, V2) {
    let ingress = col.ingress.sqrt() + EPSILON;
    let factor = a.factor * b.factor;
    col.pqt = ((a.radius + b.radius) / ingress - 1f32) * factor;
    let lamb = (a.position - b.position).scale(col.pqt);
    let total_mass = a.mass + b.mass;
    let a_mass_coeff = b.mass / total_mass;
    let friction = 1f32 - a.friction * b.friction;
    let av = (a.last_position - a.position).scale(friction);

    (
      a.position + lamb.scale(a_mass_coeff),
      a.position + av
    )
  }

  pub fn col_solve_b(col: &Collision, a: &Particle, b: &Particle) -> (V2, V2) {
    let pqt = col.pqt;
    let lamb = (a.position - b.position).scale(pqt);
    let total_mass = a.mass + b.mass;
    let b_mass_coeff = a.mass / total_mass;
    let friction = 1f32 - a.friction * b.friction;
    let bv = (b.last_position - b.position).scale(friction);

    (
      b.position + lamb.scale(b_mass_coeff),
      b.position + bv
    )
  }

  fn test_solve_virtual_ring_ddvts(ddvts: &mut Vec<VirtualRingDdvt>) -> usize {
    let mut col_group_index = 0;
    let mut col_index = 0;
    for d in 0..ddvts.len() {
      col_group_index = 0;
      let mut ddvt = &mut ddvts[d];
      let mut particles = &mut ddvt.particles;
      for i in 0..255 {
        col_index = 0;
        for j in (i + 1)..256 {
          if col_group_index >= ddvt.collisions.len() {
            ddvt.collisions.push_back(VecDeque::<Collision>::with_capacity(32));
          }
          if col_index >= ddvt.collisions[col_group_index].len() {
            ddvt.collisions[col_group_index].push_back(Collision { .. Default::default() });
          }
          if ddvt.collisions[col_group_index][col_index].test(&particles[i], &particles[j], i as u32, j as u32) {
            col_index += 1;
          }
        }

        {
          // Solve b side of collisions.
          for col_group in ddvt.collisions.iter_mut() {
            if col_group.len() > 0 && col_group[0].b_ddvt_index == i as u32 {
              {
                let col = &col_group[0];
                let (bp, blp) = col_solve_b(col, &particles[col.a_ddvt_index as usize], &particles[i]);
                let mut_particle_a = &mut particles[i];
                mut_particle_a.position = bp;
                mut_particle_a.last_position = blp;
              }
              col_group.pop_front();
            }
          }
          // for _ in 0..(col_group_index + 1) {
          //   if ddvt.collisions[0].len() == 0 {
          //     ddvt.collisions.pop_front();
          //     col_group_index -= 1;
          //   }
          // }

          // Solve a side of new collisions.
          for col in ddvt.collisions[col_group_index].iter_mut() {
            let a_index = col.a_ddvt_index as usize;
            let b_index = col.b_ddvt_index as usize;
            let (ap, alp) = col_solve_a(col, &particles[a_index], &particles[b_index]);
            let mut_particle_a = &mut particles[i];
            mut_particle_a.position = ap;
            mut_particle_a.last_position = alp;
          }
        }

        if col_index > 0 {
          col_group_index += 1;
        }
      }
      // if col_index >= ddvt.collisions.len() {
      //   ddvt.collisions.push_back(Collision { .. Default::default() });
      // }
      // ddvt.collisions[col_index] = Collision { .. Default:: default() };

      // for col in ddvt.collisions.as_slice(0, col_index).iter() {
      //   let (ap, alp, bp, blp) = col.solve(&particles[col.a_ddvt_index as usize], &particles[col.b_ddvt_index as usize]);
      //   {
      //     let mut_particle = &mut particles[col.a_ddvt_index as usize];
      //     mut_particle.position = ap;
      //     mut_particle.last_position = alp;
      //   }
      //   {
      //     let mut_particle = &mut particles[col.b_ddvt_index as usize];
      //     mut_particle.position = bp;
      //     mut_particle.last_position = blp;
      //   }
      // }
    }
    col_index
  }

  fn test_solve_virtual_future_ddvts(ddvts: &mut Vec<VirtualMergeDdvt>) -> usize {
    let mut collision = Collision { .. Default::default() };
    let mut n = 0;
    for d in 0..ddvts.len() {
      let mut ddvt = &mut ddvts[d];
      let mut particles = &mut ddvt.particles;
      let mut future = &mut ddvt.future_positions;
      for i in 0..255 {
        let particle_i = &particles[i];
        for j in (i + 1)..256 {
          if collision.test(particle_i, &particles[j], i as u32, j as u32) {
            let (ip, ilp, jp, jlp) = collision.solve(particle_i, &particles[j]);
            {
              future[i].position = future[i].position + ip;
              future[i].last_position = future[i].last_position + ilp;
              future[i].count += 1;
            }
            {
              future[j].position = future[j].position + jp;
              future[j].last_position = future[j].last_position + jlp;
              future[j].count += 1;
            }
            n += 1;
          }
        }
      }

      for i in 0..256 {
        let fcol = future[i].count;
        if fcol > 0 {
          particles[i].position = future[i].position.div_scale(fcol as f32);
          particles[i].last_position = future[i].last_position.div_scale(fcol as f32);
          future[i] = Default::default();
        }
      }
    }
    n
  }

  fn test_solve_merge_virtual_future_ddvts(ddvts: &mut Vec<VirtualMergeDdvt>) -> usize {
    let mut collision = Collision { .. Default::default() };
    let mut n = 0;
    for d in 0..ddvts.len() {
      let mut ddvt = &mut ddvts[d];
      let mut particles = &mut ddvt.particles;
      let mut future = &mut ddvt.future_positions;
      for i in 0..255 {
        let particle_i = &particles[i];
        for j in (i + 1)..256 {
          if collision.test(particle_i, &particles[j], i as u32, j as u32) {
            let (ip, ilp, jp, jlp) = collision.solve(particle_i, &particles[j]);
            {
              future[i].position = future[i].position + ip;
              future[i].last_position = future[i].last_position + ilp;
              future[i].count += 1;
            }
            {
              future[j].position = future[j].position + jp;
              future[j].last_position = future[j].last_position + jlp;
              future[j].count += 1;
            }
            n += 1;
          }
        }
      }

      // for i in 0..256 {
      //   let fcol = future[i].count;
      //   if fcol > 0 {
      //     particles[i].position = future[i].position.div_scale(fcol as f32);
      //     particles[i].last_position = future[i].last_position.div_scale(fcol as f32);
      //     future[i] = Default::default();
      //   }
      // }
    }

    let mut n = 0;
    for d in 0..ddvts.len() {
      if d % 4 != 0 { continue; }
      for i in 0..16 {
        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        let mut count = 0;
        for k in 0..4 {
          let fcol = ddvts[d + k].future_positions[i].count;
          if fcol > 0 {
            sum = sum + ddvts[d + k].future_positions[i].position;
            lpsum = lpsum + ddvts[d + k].future_positions[i].last_position;
            count += fcol;
            ddvts[d + k].future_positions[i] = Default::default();
          }
        }
        if count > 0 {
          sum = sum.div_scale(count as f32);
          lpsum = lpsum.div_scale(count as f32);
          for k in 0..4 {
            ddvts[d + k].particles[i].position = sum;
            ddvts[d + k].particles[i].last_position = lpsum;
            n += 1;
          }
        }
      }
      for i in 16..256 {
        if i % 16 != 0 {
          let fcol = ddvts[d].future_positions[i].count;
          if fcol > 0 {
            ddvts[d].future_positions[i].position = ddvts[d].future_positions[i].position.div_scale(fcol as f32);
            ddvts[d].future_positions[i].last_position = ddvts[d].future_positions[i].last_position.div_scale(fcol as f32);
            ddvts[d].future_positions[i] = Default::default();
          }
          continue;
        }

        let mut sum = V2 { .. Default::default() };
        let mut lpsum = V2 { .. Default::default() };
        let mut count = 0;
        for k in 0..4 {
          let fcol = ddvts[d + k].future_positions[i].count;
          if fcol > 0 {
            sum = sum + ddvts[d + k].future_positions[i].position;
            lpsum = lpsum + ddvts[d + k].future_positions[i].last_position;
            count += fcol;
            ddvts[d + k].future_positions[i] = Default::default();
          }
        }
        if count > 0 {
          sum = sum.div_scale(count as f32);
          lpsum = lpsum.div_scale(count as f32);
          for k in 0..4 {
            ddvts[d + k].particles[i].position = sum;
            ddvts[d + k].particles[i].last_position = lpsum;
            n += 1;
          }
        }
      }
    }

    n
  }

  #[bench]
  fn virtual_ddvt_test(b: &mut Bencher) {
    let mut ddvts = Vec::<VirtualDdvt>::with_capacity(32);
    init_virtual_ddvts(&mut ddvts);
    test_virtual_ddvts(&mut ddvts);

    b.iter(|| {
      test_virtual_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn virtual_ddvt_test_merge(b: &mut Bencher) {
    let mut ddvts = Vec::<VirtualDdvt>::with_capacity(32);
    init_virtual_ddvts(&mut ddvts);
    reset_virtual_ddvts(&mut ddvts) +
      test_virtual_ddvts(&mut ddvts) +
      merge_virtual_ddvts(&mut ddvts);

    b.iter(|| {
      reset_virtual_ddvts(&mut ddvts) +
        test_virtual_ddvts(&mut ddvts) +
        merge_virtual_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn virtual_ddvt_test_solve_merge(b: &mut Bencher) {
    let mut ddvts = Vec::<VirtualDdvt>::with_capacity(32);
    init_virtual_ddvts(&mut ddvts);
    reset_virtual_ddvts(&mut ddvts) +
      test_virtual_ddvts(&mut ddvts) +
      solve_virtual_ddvts(&mut ddvts) +
      merge_virtual_ddvts(&mut ddvts);

    b.iter(|| {
      reset_virtual_ddvts(&mut ddvts) +
        test_virtual_ddvts(&mut ddvts) +
        solve_virtual_ddvts(&mut ddvts) +
        merge_virtual_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn virtual_ddvt_test_solve2(b: &mut Bencher) {
    let mut ddvts = Vec::<VirtualDdvt>::with_capacity(32);
    init_virtual_ddvts(&mut ddvts);

    b.iter(|| {
      reset_virtual_ddvts(&mut ddvts) +
        test_solve_virtual_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn virtual_ring_ddvt_test_solve(b: &mut Bencher) {
    let mut ddvts = Vec::<VirtualRingDdvt>::with_capacity(32);
    init_virtual_ring_ddvts(&mut ddvts);

    b.iter(|| {
      reset_virtual_ring_ddvts(&mut ddvts) +
        test_solve_virtual_ring_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn virtual_future_ddvt_test_solve(b: &mut Bencher) {
    let mut ddvts = Vec::<VirtualMergeDdvt>::with_capacity(32);
    init_virtual_future_ddvts(&mut ddvts);
    test_solve_virtual_future_ddvts(&mut ddvts);

    b.iter(|| {
      reset_virtual_future_ddvts(&mut ddvts) +
        test_solve_virtual_future_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn virtual_future_ddvt_test_solve_merge(b: &mut Bencher) {
    let mut ddvts = Vec::<VirtualMergeDdvt>::with_capacity(32);
    init_virtual_future_ddvts(&mut ddvts);
    test_solve_virtual_future_ddvts(&mut ddvts);

    b.iter(|| {
      reset_virtual_future_ddvts(&mut ddvts) +
        test_solve_virtual_future_ddvts(&mut ddvts) +
        merge_virtual_future_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn virtual_future_ddvt_test_solve_merge_inline(b: &mut Bencher) {
    let mut ddvts = Vec::<VirtualMergeDdvt>::with_capacity(32);
    init_virtual_future_ddvts(&mut ddvts);
    test_solve_virtual_future_ddvts(&mut ddvts);

    b.iter(|| {
      reset_virtual_future_ddvts(&mut ddvts) +
        test_solve_merge_virtual_future_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn mutex_virtual_ddvt_test_solve(b: &mut Bencher) {
    let mut pool = new_mutex_virtual_ddvt_test_pool(num_cpus::get());
    let mut ddvts = Vec::<Arc<Mutex<VirtualDdvt>>>::with_capacity(32);
    init_mutex_virtual_ddvts(&mut ddvts);
    test_solve_mutex_virtual_ddvts(&mut ddvts, &mut pool);

    b.iter(|| {
      reset_mutex_virtual_ddvts(&mut ddvts) +
        test_solve_mutex_virtual_ddvts(&mut ddvts, &mut pool)
    })
  }

  #[bench]
  fn mutex_virtual_ddvt_test_solve_merge(b: &mut Bencher) {
    let mut pool = new_mutex_virtual_ddvt_test_pool(num_cpus::get());
    let mut ddvts = Vec::<Arc<Mutex<VirtualDdvt>>>::with_capacity(32);
    init_mutex_virtual_ddvts(&mut ddvts);
    test_solve_mutex_virtual_ddvts(&mut ddvts, &mut pool);

    b.iter(|| {
      reset_mutex_virtual_ddvts(&mut ddvts) +
        test_solve_mutex_virtual_ddvts(&mut ddvts, &mut pool) +
        merge_mutex_virtual_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn boxed_virtual_ddvt_test_solve(b: &mut Bencher) {
    let mut pool = new_boxed_virtual_ddvt_test_pool(num_cpus::get());
    let mut ddvts = Vec::<Option<Box<VirtualDdvt>>>::with_capacity(32);
    init_boxed_virtual_ddvts(&mut ddvts);
    test_solve_boxed_virtual_ddvts(&mut ddvts, &mut pool);

    b.iter(|| {
      reset_boxed_virtual_ddvts(&mut ddvts) +
        test_solve_boxed_virtual_ddvts(&mut ddvts, &mut pool)
    })
  }

  #[bench]
  fn boxed_virtual_ddvt_test_solve_merge(b: &mut Bencher) {
    let mut pool = new_boxed_virtual_ddvt_test_pool(num_cpus::get());
    let mut ddvts = Vec::<Option<Box<VirtualDdvt>>>::with_capacity(32);
    init_boxed_virtual_ddvts(&mut ddvts);
    test_solve_boxed_virtual_ddvts(&mut ddvts, &mut pool);

    b.iter(|| {
      reset_boxed_virtual_ddvts(&mut ddvts) +
        test_solve_boxed_virtual_ddvts(&mut ddvts, &mut pool) +
        merge_boxed_virtual_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn ref_ddvt_test(b: &mut Bencher) {
    let mut ddvts = Vec::<RefDdvt>::with_capacity(32);
    init_ref_ddvts(&mut ddvts);

    b.iter(|| {
      test_ref_ddvts(&mut ddvts)
    })
  }

  #[bench]
  fn ref_ddvt_test_solve(b: &mut Bencher) {
    let mut ddvts = Vec::<RefDdvt>::with_capacity(32);
    init_ref_ddvts(&mut ddvts);

    b.iter(|| {
      test_ref_ddvts(&mut ddvts) +
        solve_ref_ddvts(&mut ddvts)
    })
  }
}
