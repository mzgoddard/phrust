use std::default::Default;
pub use super::math::*;

#[derive(Clone, Copy)]
pub enum State {
  Dynamic,
  Asleep,
  Trigger,
  Static,
}

#[derive(Clone, Copy)]
pub struct Particle {
  pub id: usize,
  pub position: V2,
  pub last_position: V2,
  pub acceleration: V2,
  pub radius: f32,
  pub radius2: f32,
  pub mass: f32,
  pub friction: f32,
  pub factor: f32,
  pub state: State,
  pub bbox: BB,
  pub uncontained: bool,
  // pub old_ddvt_position: V2,
}

const LAST_POS_MUL : f32 = -0.99;
const POS_MUL : f32 = 1.99;

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
    self.position =
      self.position.scale_add(
        POS_MUL,
        self.last_position.scale_add(
          LAST_POS_MUL,
          self.acceleration.scale(dt2)
        )
      );
    self.last_position = position;
    // self.acceleration = V2::zero();
    // self.bbox = self.bb();
  }
}

impl Default for Particle {
  fn default() -> Particle {
    Particle {
      id: Default::default(),
      position: Default::default(),
      last_position: Default::default(),
      acceleration: Default::default(),
      radius: 1f32,
      radius2: 1f32,
      mass: 1f32,
      friction: 0.01f32,
      factor: 0.5f32,
      state: State::Dynamic,
      bbox: Default::default(),
      uncontained: false,
      // old_ddvt_position: Default::default(),
    }
  }
}
