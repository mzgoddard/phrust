use std::default::Default;
pub use super::math::*;

#[derive(Clone, Copy)]
pub enum State {
  Dynamic,
  Asleep,
  Trigger,
  Static,
  Dead,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct ParticleId {
  id: usize,
}

#[derive(Clone, Copy)]
pub struct Particle {
  pub id: usize,
  pub position: V2,
  pub radius: f32,
  pub mass: f32,
  pub last_position: V2,
  pub acceleration: V2,
  pub drag: f32,
  pub friction: f32,
  pub friction2: f32,
  pub state: State,
  pub bbox: BB,
  pub uncontained: bool,
  // pub old_ddvt_position: V2,
}

// impl Deref for ParticleId {
//
// }
//
// impl Borrow<ParticleId> for Particle {
//   fn borrow(&self) -> &ParticleId {
//
//   }
// }

impl Particle {
  pub fn at(p: V2) -> Particle {
    Particle {
      position: p,
      last_position: p,
      .. Default::default()
    }
  }

  pub fn bb(&self) -> BB {
    BB::from_circle(self.position, self.radius)
    // BB::from_circle(self.position, self.pressure_radius)
  }

  #[inline]
  pub fn integrate(&mut self, dt2: f32) {
    // let lastpos = self.last_position * LAST_POS_MUL;
    let position = self.position;
    // p = a * dt2 + (p - lp) * drag + p
    self.position = self.acceleration.scale_add(dt2,
      (self.position - self.last_position).scale_add(self.drag,
        self.position
      )
    );
    // self.position =
    //   self.position.scale_add(
    //     self.drag + 1.0,
    //     self.last_position.scale_add(
    //       -self.drag,
    //       self.acceleration.scale(dt2)
    //     )
    //   );
    self.last_position = position;
    // self.acceleration = V2::zero();
    // self.bbox = self.bb();
  }

  pub fn is_dynamic(&self) -> bool {
    match self.state {
      State::Dynamic => true,
      _ => false,
    }
  }

  pub fn is_trigger(&self) -> bool {
    match self.state {
      State::Trigger => true,
      _ => false,
    }
  }

  pub fn is_dead(&self) -> bool {
    match self.state {
      State::Dead => true,
      _ => false,
    }
  }
}

impl Default for Particle {
  fn default() -> Particle {
    Particle {
      id: Default::default(),
      // worldid: Default::default(),
      position: Default::default(),
      last_position: V2::infinity(),
      acceleration: Default::default(),
      radius: 1f32,
      mass: 1f32,
      drag: 0.999,
      friction: 0.01f32,
      friction2: 0.0001f32,
      state: State::Dynamic,
      bbox: Default::default(),
      uncontained: false,
      // old_ddvt_position: Default::default(),
    }
  }
}
