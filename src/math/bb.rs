use super::v2::V2;
use super::base5::b5;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct BB {
  pub l: f64,
  pub b: f64,
  pub r: f64,
  pub t: f64,
}

impl BB {
  pub fn from_circle(center: V2, radius: f64) -> BB {
    BB {
      l: center.x - radius,
      b: center.y - radius,
      r: center.x + radius,
      t: center.y + radius,
    }
  }

  pub fn as_v2_vec(self) -> Vec<V2> {
    vec![V2{ x: self.l, y: self.b }, V2{ x: self.r, y: self.t }]
  }

  pub fn contains(self, b: &BB) -> bool {
     self.l < b.l && self.b < b.b && self.r > b.r && self.t > b.t
  }

  pub fn overlaps(self, b: &BB) -> bool {
    self.l < b.r && self.b < b.t && self.r > b.l && self.t > b.b
  }
}

#[test]
fn bb_contains() {
  assert!(BB { t: 0f64, l: 0f64, b: -4f64, r: 4f64 }.contains(&BB { t: -1f64, l: 1f64, b: -3f64, r: 3f64 }))
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
