#[cfg(test)]
mod bench {

  struct DvhWorld {
    particles: Vec<Particle>,
    broadphase: Dvh,

    _particleBoxCache: Vec<BB>,
  }

  impl DvhWorld {
    fn particleBoxes(self) -> Vec<BB> {
      let boxes = Vec::<BB>::new();
      self.particleBoxesOver(boxes);
      boxes
    }

    fn particleBoxesOver(self, positions: &mut Vec<BB>) {
      positions.clear();
      for p in particles {
        positions.push(p.bb());
      }
    }
  }

  struct ParticlePosition {
    position: V2,
  }

  type ParticlePositionSlice = Vec<ParticlePosition>;

  impl ParticlePositionSlice {
    
  }

  struct BB {}

  struct Dvh {
    volumes: BTreeMap<Volume>,
    updateVolumeSetCache: Vec<Box<VirtualVolumeSet>>,
  }

  struct Volume {
    id: i32,
    bb: BB,
    particles: [Particle; 256],
  }

  struct VirtualVolumeSet {
    id: i32,
    index: usize,
    parentIndex: usize,
    childIndices: Vec<usize>,
    volumes: Vec<Box<VirtualVolume>>,
    particleLength: usize,
  }

  struct VirtualVolume {
    id: i32,
    particles: Vec<VirtualVolumeParticle>,
  }

  struct VirtualVolumeParticle {
    particleIndex: usize,
    volumeId: i32,
  }

  struct Pool {}

  impl DvhWorld {
    fn updateBroadphase(self) {
      // Copy particle bounding boxes
      let boxes = self._particleBoxCache;
      self.particleBoxesOver(&mut boxes);
      self.particlesClearDdvtIds();

      // Build virtual volumes
      let virtualVolumes = BTreeMap::<i32, Option<Box<VirtualVolume>>::new();
      let parentVolumes = Vec::<Option<Box<VirtualVolume>>::with(14);
      for leaf in self.broadphase.volumes {
        // let volume = nextFreeVolume();
        let leafIdMask = leaf.id.mask();
        let volumeDepth = leaf.id.length();
        parentVolumes[volumeDepth - 1] = virtualVolumes[leaf.id];
        for (let i = volumeDepth - 2; i >= 0; i--, mask /= 5) {
          parentVolumes[i] = None;
        }
        for particle in leaf.particles {
          let mask = leafIdMask;
          for (let i = volumeDepth - 1; i >= 0; i--, mask /= 5) {
            let vVolume = match parentVolumes[i] {
              Some(v) => {
                // Make sure the id is correct
                if v.id != 
                // Restore the volume to virtualVolumes and pull the appropriate one
              },
              None => {
                // Pull volume from virtual 
                parentVolumes[i]
              }
            };
            let bb = vVolume.bb;
            if bb.contains(boxes[particle.id]) {
              vVolume.particles.push(VirtualVolumeParticle { particleIndex: particle.id, volumeId: vVolume.id });
              break;
            }
          }
        }
      }
      // Restore parents to the virtualVolume set
      for (let i = 0; i < ) {
        match parentVolumes[i] {
          Some(v) => {
            virtualVolumes[v.id] = parentVolumes[i].take();
          },
          None => {},
        }
      }

      // Collect virtual volumes

      // Summarize virtual volumes
      // Loop non-leaf volumes, find leaves for particles without leaves
      // Loop leaf volumes, divide if too many particles, collapse if too few
      // Assign leaf ids
      // Update top ddvt id for particles

      // Update existing leaves
      // Add ddvt ids to particles (needed to merge positions after collision)

      // Delete old leaves
      // Add ddvt ids to particles

      // Create new leaves
      // Add ddvt ids to particles
    }
  }

}
