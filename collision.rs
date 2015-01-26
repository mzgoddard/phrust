use std::f64::EPSILON;
use std::num::Float;
use super::particle::Particle;
use super::math::*;

#[derive(Copy, Default, Show)]
pub struct Collision {
  pub a_ddvt_index: u32,
  pub b_ddvt_index: u32,
  // a: Option<&'a Particle>,
  // b: Option<&'a Particle>,
  pub ingress: f64,
  pub pqt: f64,
}

impl Collision {
  #[inline]
  pub fn test(&mut self, a: &Particle, b: &Particle, a_index: u32, b_index: u32) -> bool {
    let ingress = (a.position - b.position).mag2();
    if a.radius2 + b.radius2 > ingress {
      *self = Collision { a_ddvt_index: a_index, b_ddvt_index: b_index, ingress: ingress, pqt: 0f64 };
      true
    } else {
      false
    }
  }

  pub fn solve(self, a: &Particle, b: &Particle) -> (V2, V2, V2, V2) {
    // let a = self.a.unwrap();
    // let b = self.b.unwrap();
    let ingress = self.ingress.sqrt() + EPSILON;
    let factor = a.factor * b.factor;
    let pqt = ((a.radius + b.radius) / ingress - 1f64) * factor;
    let lamb = (a.position - b.position).scale(pqt);
    let total_mass = a.mass + b.mass;
    let a_mass_coeff = b.mass / total_mass;
    let b_mass_coeff = a.mass / total_mass;
    let friction = 1f64 - a.friction * b.friction;
    let av = (a.last_position - a.position).scale(friction);
    let bv = (b.last_position - b.position).scale(friction);

    (
      a.position + lamb.scale(a_mass_coeff),
      a.position + av,
      b.position + lamb.scale(b_mass_coeff),
      b.position + bv
    )
    // a.last_position = a.position + av;
    // a.position = a.position + lamb.scale(a_mass_coeff);
    //
    // b.last_position = b.position + bv;
    // b.position = b.position + lamb.scale(b_mass_coeff);
  }
}

#[cfg(test)]
mod bench {
  extern crate test;
  use self::test::Bencher;
  use super::Collision;
  use particle::Particle;
  use math::V2;
  use std::default::Default;

  #[test]
  fn fn_test() {
    let mut c: Collision = Default::default();
    let a = Particle::at(Default::default());
    let b = Particle::at(V2 { x: 0.5f64, y: 0f64 });
    assert!(c.test(&a, &b, 0u32, 1u32));
    assert_eq!(c.a_ddvt_index, 0u32);
    assert_eq!(c.b_ddvt_index, 1u32);
    assert_eq!(c.ingress, 0.25f64);
    // assert!(c.a.is_some());
    // assert!(c.b.is_some());
    // let c = Collision::test(&a, &b);
    // assert!(c.is_some());
    // assert!(c.unwrap().a.is_some());
    // assert!(c.unwrap().b.is_some());
  }

  #[bench]
  fn b_fn_test(b: &mut Bencher) {
    let pa = Particle::at(Default::default());
    let pb = Particle::at(V2 { x: 0.5f64, y: 0f64 });
    let mut cols = Vec::<Collision>::with_capacity(330000);
    for i in 0..cols.capacity() {
      cols.push(Default::default());
    }
    b.iter(|&mut:| {
      for i in 0..cols.capacity() {
        cols[i].test(&pa, &pb, 0u32, 1u32);
      }
      cols.len()
    })
  }

  #[bench]
  fn b_fn_solve(b: &mut Bencher) {
    let mut pa = Particle::at(Default::default());
    let mut pb = Particle::at(V2 { x: 0.5f64, y: 0f64 });
    let mut cols = Vec::<Collision>::with_capacity(330000);
    for i in 0..cols.capacity() {
      cols.push(Default::default());
    }
    b.iter(|&mut:| {
      for i in 0..cols.capacity() {
        cols[i].test(&pa, &pb, 0, 1);
      }
      for i in 0..cols.capacity() {
        cols[i].solve(&pa, &pb);
        // pa.position = Default::default();
        // pb.position = V2 { x: 0.5f64, y: 0f64 };
      }
      cols.len()
    })
  }
}
