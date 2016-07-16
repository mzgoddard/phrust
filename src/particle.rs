use std::default::Default;
pub use super::math::*;

#[derive(Clone, Copy, Show)]
pub enum State {
  Dynamic,
  Asleep,
  Trigger,
  Static,
}

#[derive(Clone, Copy, Show)]
pub struct Particle {
  pub id: u32,
  pub position: V2,
  pub last_position: V2,
  pub acceleration: V2,
  pub radius: f64,
  pub radius2: f64,
  pub mass: f64,
  pub friction: f64,
  pub factor: f64,
  pub state: State,
  pub old_ddvt_position: V2,
}

impl Particle {
  pub fn at(p: V2) -> Particle {
    Particle {
      position: p,
      last_position: p,
      .. Default::default()
    }
  }

  pub fn bb(self) -> BB {
    BB::from_circle(self.position, self.radius)
  }
}

impl Default for Particle {
  fn default() -> Particle {
    Particle {
      id: Default::default(),
      position: Default::default(),
      last_position: Default::default(),
      acceleration: Default::default(),
      radius: 1f64,
      radius2: 1f64,
      mass: 1f64,
      friction: 0.99f64,
      factor: 0.5f64,
      state: State::Dynamic,
      old_ddvt_position: Default::default(),
    }
  }
}
