use std::f32;
use std::fmt;

use super::v2::V2;
use super::base5::b5;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct BB {
  pub l: f32,
  pub b: f32,
  pub r: f32,
  pub t: f32,
}

#[macro_export]
macro_rules! bb {
  ($l:expr, $b:expr, $r:expr, $t:expr) => { {
    BB {
      l: $l as f32,
      b: $b as f32,
      r: $r as f32,
      t: $t as f32,
    }
  } }
}


impl BB {
  pub fn from_circle(center: V2, radius: f32) -> BB {
    BB {
      l: center.x - radius,
      b: center.y - radius,
      r: center.x + radius,
      t: center.y + radius,
    }
  }

  pub fn infinity() -> BB {
    BB {
      l: f32::INFINITY,
      b: f32::INFINITY,
      r: f32::INFINITY,
      t: f32::INFINITY,
    }
  }

  pub fn width(&self) -> f32 {
    self.r - self.l
  }

  pub fn height(&self) -> f32 {
    self.t - self.b
  }

  pub fn tl(&self) -> BB {
    BB {
      b: self.t - self.height() / 2f32,
      r: self.l + self.width() / 2f32,
      .. *self
    }
  }

  pub fn tr(&self) -> BB {
    BB {
      b: self.t - self.height() / 2f32,
      l: self.r - self.width() / 2f32,
      .. *self
    }
  }

  pub fn bl(&self) -> BB {
    BB {
      t: self.b + self.height() / 2f32,
      r: self.l + self.width() / 2f32,
      .. *self
    }
  }

  pub fn br(&self) -> BB {
    BB {
      t: self.b + self.height() / 2f32,
      l: self.r - self.width() / 2f32,
      .. *self
    }
  }

  pub fn as_v2_vec(&self) -> Vec<V2> {
    vec![V2{ x: self.l, y: self.b }, V2{ x: self.r, y: self.t }]
  }

  #[inline]
  pub fn intersect(&self, b: BB) -> BB {
    BB {
      l: f32::max(self.l, b.l) + f32::EPSILON,
      b: f32::max(self.b, b.b) + f32::EPSILON,
      r: f32::min(self.r, b.r) - f32::EPSILON,
      t: f32::min(self.t, b.t) - f32::EPSILON,
    }
  }

  #[inline]
  pub fn join(&self, b: BB) -> BB {
    BB {
      l: f32::min(self.l, b.l),
      b: f32::min(self.b, b.b),
      r: f32::max(self.r, b.r),
      t: f32::max(self.t, b.t),
    }
  }

  #[inline]
  pub fn contains(&self, b: &BB) -> bool {
    self.l <= b.l && self.b <= b.b && self.r >= b.r && self.t >= b.t
  }

  #[inline]
  pub fn child_contains(&self, b: &BB) -> bool {
    let cx = (self.l + self.r) / 2.0;
    let cy = (self.b + self.t) / 2.0;
    cy < b.b && (cx > b.r || cx < b.l) ||
    cy > b.t && (cx > b.r || cx < b.l)
  }

  #[inline]
  pub fn overlaps(self, b: BB) -> bool {
    self.l <= b.r && self.b <= b.t && self.r >= b.l && self.t >= b.b
  }

  pub fn overlap_children(self, i: usize, b: BB) -> bool {
    self.b < b.t && (i == 0 && self.r > b.l || i == 1 && self.l < b.r) ||
    self.t > b.t && (i == 2 && self.r > b.l || i == 3 && self.l < b.r)
  }

  // pub fn overlaps_either(self, a: BB, b: BB) -> bool {
  //   a.l < b.l && b.l < self.r;
  //   b.l < a.l && a.l < self.r;
  //   self.l < a.r && self.l < b.r
  //   a.l < self.r && b.l < self.r
  //   self.r > a.l && self.r > b.
  //   self.l < a.r && self.r > a.l
  //   self.l < b.r && self.r > b.l
  //   self.l < a.r && self.b < a.t && self.r > a.l && self.t > a.b
  //   self.l < b.r && self.b < b.t && self.r > b.l && self.t > b.b
  // }
}

impl fmt::Display for BB {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "bb!({}, {}, {}, {})", self.l, self.b, self.r, self.t)
  }
}

#[test]
fn bb_contains() {
  assert!(BB { t: 0f32, l: 0f32, b: -4f32, r: 4f32 }.contains(&BB { t: -1f32, l: 1f32, b: -3f32, r: 3f32 }))
}

pub struct BBId {
  id: u32,
  bb: BB,
}

impl BBId {
  pub fn parent(self) -> BBId {
    let bit = b5::from(self.id).highest_bit().value();
    let parentId = self.id - (b5::from(bit) << (b5::from(self.id).length() - 1)).value();
    BBId {
      id: parentId,
      bb: match bit {
        // top left
        1 => BB { l: self.bb.l, b: self.bb.b * 2.0 - self.bb.t, r: self.bb.r * 2.0 - self.bb.l, t: self.bb.t },
        2 => BB { l: self.bb.l * 2.0 - self.bb.r, b: self.bb.b * 2.0 - self.bb.t, r: self.bb.r, t: self.bb.t },
        3 => BB { l: self.bb.l * 2.0 - self.bb.r, b: self.bb.b, r: self.bb.r, t: self.bb.t * 2.0 - self.bb.b },
        4 => BB { l: self.bb.l, b: self.bb.b, r: self.bb.r * 2.0 - self.bb.l, t: self.bb.t * 2.0 - self.bb.b },
        _ => BB { .. Default::default() }
      },
    }
  }

  pub fn tl(self) -> BBId {
    let id = self.id + (b5::from(1) << b5::from(self.id).length()).value();
    BBId {
      id: id,
      bb: BB { l: self.bb.l, b: self.bb.b + (self.bb.t - self.bb.b) / 2.0, r: self.bb.r - (self.bb.r - self.bb.l) / 2.0, t: self.bb.t },
    }
  }
}
