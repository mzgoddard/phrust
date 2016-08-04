use std::f32::EPSILON;
// use std::num::Float;
use super::particle::Particle;
use super::math::*;

#[derive(Copy, Clone, Default)]
pub struct Collision {
  // pub a_ddvt_index: usize,
  // pub b_ddvt_index: usize,
  // a: Option<&'a Particle>,
  // b: Option<&'a Particle>,
  pub ingress: f32,
  pub pqt: f32,
}

impl Collision {
  // #[inline]
  // pub fn test(a: &Particle, b: &Particle,
  //   // a_index: usize, b_index: usize
  // ) -> Option<Collision> {
  //   let ingress = (a.position - b.position).mag2();
  //   if a.radius2 + b.radius2 > ingress {
  //   // if a.pressure_radius2 + b.pressure_radius2 > ingress {
  //     Some(Collision {
  //       // a_ddvt_index: a_index, b_ddvt_index: b_index,
  //       ingress: ingress, pqt: 0f32 })
  //   } else {
  //     None
  //   }
  // }

  // pub fn solve(self, a: &Particle, b: &Particle) -> (V2, V2, V2, V2) {
  //   // let a = self.a.unwrap();
  //   // let b = self.b.unwrap();
  //   let ingress = self.ingress.sqrt() + EPSILON;
  //   let factor = a.factor * b.factor;
  //   let pqt = ((a.radius + b.radius) / ingress - 1f32) * factor;
  //   // let pqt = ((a.pressure_radius + b.pressure_radius) / ingress - 1f32) * factor;
  //   let lamb = (a.position - b.position).scale(pqt);
  //   let total_mass = a.mass + b.mass;
  //   let a_mass_coeff = b.mass / total_mass;
  //   let b_mass_coeff = a.mass / total_mass;
  //   // let total_mass = a.pressure_mass + b.pressure_mass;
  //   // let a_mass_coeff = b.pressure_mass / total_mass;
  //   // let b_mass_coeff = a.pressure_mass / total_mass;
  //   let friction = 1.0 - a.friction * b.friction;
  //   let av = (a.last_position - a.position).scale(friction);
  //   let bv = (b.last_position - b.position).scale(friction);
  //
  //   (
  //     a.position + lamb.scale(a_mass_coeff),
  //     a.position + av,
  //     b.position - lamb.scale(b_mass_coeff),
  //     b.position + bv
  //   )
  //   // a.last_position = a.position + av;
  //   // a.position = a.position + lamb.scale(a_mass_coeff);
  //   //
  //   // b.last_position = b.position + bv;
  //   // b.position = b.position - lamb.scale(b_mass_coeff);
  // }

  // pub fn test2(a: &Particle, b: &Particle) -> Option<f32> {
  // }

  #[inline]
  pub fn test2(a: &Particle, b: &Particle) -> bool {
    (a.radius + b.radius) * (a.radius + b.radius) > a.position.dist2(b.position)
  }

  #[inline]
  pub fn solve2(a: &Particle, b: &Particle) -> (V2, V2, f32, f32) {
    let diff = a.position - b.position;
    let abr = a.radius + b.radius;
    let ingress_r = diff.rmag();
    let pqt = (abr * ingress_r + -1.0).min(1.0) * ingress_r;
    let ingress = -1.0 / (abr * ingress_r) + 1.0;
    let am = b.mass / (a.mass + b.mass);
    let bm = a.mass / (a.mass + b.mass);

    (
      // diff.scale(pqt),
      // diff.scale(-pqt),
      diff.scale(am * pqt),
      diff.scale(-bm * pqt),
      ingress * am,
      ingress * bm
    )
    // let diff = a.position - b.position;
    // let ingress_ep = a.position.dist(b.position) + EPSILON;
    // // Decrease ingress a little to increase the power of the spring a little.
    // let pqt = ((((a.radius + b.radius) / (ingress_ep)) - 1.0f32).min(1.0) / (ingress_ep));
    // let ingress = (a.radius + b.radius - ingress_ep) / (a.radius + b.radius);
    //
    // (
    //   // diff.scale(pqt),
    //   // diff.scale(-pqt),
    //   diff.scale(b.mass / (a.mass + b.mass) * pqt),
    //   diff.scale(-a.mass / (a.mass + b.mass) * pqt),
    //   ingress * b.mass / (a.mass + b.mass),
    //   ingress * a.mass / (a.mass + b.mass)
    // )
  }

  #[inline]
  pub fn test_solve(a: &Particle, b: &Particle) -> Option<(V2, V2, f32)> {
    if (a.radius + b.radius) * (a.radius + b.radius) - (a.position - b.position).mag2() > 0.0 {
      let pos_diff = a.position - b.position;
      let ingress_sq = (a.position - b.position).mag2();
      let ingress = ingress_sq.sqrt() + EPSILON + (a.radius + b.radius) / 50.0;
      let pqt = ((a.radius + b.radius) / ingress).powf(1.5) - 1.0f32;
      let total_mass = (a.mass + b.mass) / pqt;
      let a_mass_coeff = b.mass / total_mass;
      let b_mass_coeff = a.mass / total_mass;

      Some((
        pos_diff.scale(a_mass_coeff),
        pos_diff.scale(-b_mass_coeff),
        ingress_sq
      ))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod test {
  use super::Collision;
  use particle::Particle;
  use math::V2;
  use std::default::Default;

  #[test]
  fn fn_test() {
    // let mut c: Collision = Default::default();
    let a = Particle::at(Default::default());
    let b = Particle::at(V2 { x: 0.5f32, y: 0f32 });
    assert!(Collision::test2(&a, &b));
    // assert_eq!(c.a_ddvt_index, 0u32);
    // assert_eq!(c.b_ddvt_index, 1u32);
    // assert_eq!(c.ingress, 0.25f32);
    // assert!(c.a.is_some());
    // assert!(c.b.is_some());
    // let c = Collision::test(&a, &b);
    // assert!(c.is_some());
    // assert!(c.unwrap().a.is_some());
    // assert!(c.unwrap().b.is_some());
  }
}

#[cfg(bench)]
mod bench {
  extern crate test;
  use self::test::Bencher;
  use super::Collision;
  use particle::Particle;
  use math::V2;
  use std::default::Default;

  #[bench]
  fn b_fn_test(b: &mut Bencher) {
    let pa = Particle::at(Default::default());
    let pb = Particle::at(V2 { x: 0.5f32, y: 0f32 });
    let mut cols = Vec::<Collision>::with_capacity(330000);
    for i in 0..cols.capacity() {
      cols.push(Default::default());
    }
    b.iter(|| {
      for i in 0..cols.capacity() {
        cols[i].test(&pa, &pb, 0u32, 1u32);
      }
      cols.len()
    })
  }

  #[bench]
  fn b_fn_solve(b: &mut Bencher) {
    let mut pa = Particle::at(Default::default());
    let mut pb = Particle::at(V2 { x: 0.5f32, y: 0f32 });
    let mut cols = Vec::<Collision>::with_capacity(330000);
    for i in 0..cols.capacity() {
      cols.push(Default::default());
    }
    b.iter(|| {
      for i in 0..cols.capacity() {
        cols[i].test(&pa, &pb, 0, 1);
      }
      for i in 0..cols.capacity() {
        cols[i].solve(&pa, &pb);
        // pa.position = Default::default();
        // pb.position = V2 { x: 0.5f32, y: 0f32 };
      }
      cols.len()
    })
  }
}
