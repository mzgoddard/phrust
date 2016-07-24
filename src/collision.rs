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

fn bad_rt(v: f32) -> f32 {
  let mut top_rt = v;
  let mut bot_rt = 0.0;
  let mut rt = v;
  if v == 0.0 {}
  else if v < 1.0 {
    bot_rt = v;
    top_rt = 1.0;
    while rt * rt > v * 1.25 || rt * rt < v * 0.75 {
      if rt * rt < v {
        bot_rt = rt;
      }
      else {
        top_rt = rt;
      }
      rt = (top_rt + bot_rt) / 2.0;
    }
  }
  else {
    while rt * rt > v * 1.25 || rt * rt < v * 0.75 {
      if rt * rt < v {
        bot_rt = rt;
      }
      else {
        top_rt = rt;
      }
      rt = (top_rt + bot_rt) / 2.0;
    }
  }
  rt
}

impl Collision {
  #[inline]
  pub fn test(a: &Particle, b: &Particle,
    // a_index: usize, b_index: usize
  ) -> Option<Collision> {
    let ingress = (a.position - b.position).mag2();
    if a.radius2 + b.radius2 > ingress {
    // if a.pressure_radius2 + b.pressure_radius2 > ingress {
      Some(Collision {
        // a_ddvt_index: a_index, b_ddvt_index: b_index,
        ingress: ingress, pqt: 0f32 })
    } else {
      None
    }
  }

  pub fn solve(self, a: &Particle, b: &Particle) -> (V2, V2, V2, V2) {
    // let a = self.a.unwrap();
    // let b = self.b.unwrap();
    let ingress = self.ingress.sqrt() + EPSILON;
    let factor = a.factor * b.factor;
    let pqt = ((a.radius + b.radius) / ingress - 1f32) * factor;
    // let pqt = ((a.pressure_radius + b.pressure_radius) / ingress - 1f32) * factor;
    let lamb = (a.position - b.position).scale(pqt);
    let total_mass = a.mass + b.mass;
    let a_mass_coeff = b.mass / total_mass;
    let b_mass_coeff = a.mass / total_mass;
    // let total_mass = a.pressure_mass + b.pressure_mass;
    // let a_mass_coeff = b.pressure_mass / total_mass;
    // let b_mass_coeff = a.pressure_mass / total_mass;
    let friction = 1.0 - a.friction * b.friction;
    let av = (a.last_position - a.position).scale(friction);
    let bv = (b.last_position - b.position).scale(friction);

    (
      a.position + lamb.scale(a_mass_coeff),
      a.position + av,
      b.position - lamb.scale(b_mass_coeff),
      b.position + bv
    )
    // a.last_position = a.position + av;
    // a.position = a.position + lamb.scale(a_mass_coeff);
    //
    // b.last_position = b.position + bv;
    // b.position = b.position - lamb.scale(b_mass_coeff);
  }

  // pub fn test2(a: &Particle, b: &Particle) -> Option<f32> {
  // }

  #[inline]
  pub fn test2(a: &Particle, b: &Particle) -> bool {
    // (a.radius + b.radius) * (a.radius + b.radius) > a.position.dist2(b.position)
    (a.pressure_radius + b.pressure_radius) * (a.pressure_radius + b.pressure_radius) > a.position.dist2(b.position)
  }

  #[inline]
  pub fn solve2(a: &Particle, b: &Particle) -> (V2, V2, f32, f32) {
    let diff = a.position - b.position;
    let ingress_sq = diff.mag2();
    // let pqt = ((
    //   (a.radius + b.radius) /
    //   (ingress_sq + EPSILON + (a.radius + b.radius) / 4.0).sqrt()
    // ) - 1.0f32);
    //
    // (
    //   diff.scale(a.mass / (a.mass + b.mass) * pqt),
    //   diff.scale(-b.mass / (a.mass + b.mass) * pqt),
    //   ingress_sq
    // )

    let pqt = ((
      (a.pressure_radius + b.pressure_radius) / (ingress_sq + EPSILON).sqrt()
    ) - 1.0f32);
    let ingress = (a.pressure_radius + b.pressure_radius - (ingress_sq + EPSILON).sqrt()) / (a.pressure_radius + b.pressure_radius);

    (
      diff.scale(b.pressure_mass / (a.pressure_mass + b.pressure_mass) * pqt),
      diff.scale(-a.pressure_mass / (a.pressure_mass + b.pressure_mass) * pqt),
      ingress * b.pressure_mass / (a.pressure_mass + b.pressure_mass),
      ingress * a.pressure_mass / (a.pressure_mass + b.pressure_mass)
    )
  }

  #[inline]
  pub fn test_solve(a: &Particle, b: &Particle,
    // a_index: usize, b_index: usize
  ) -> Option<(V2, V2, f32)> {
    // let pos_diff = a.position - b.position;
    // (ax - bx) * (ax - bx) + (ay - by) * (ay - by)
    // ax * ax - 2 * bx * ax + bx * bx + ay * ay - 2 * by * ay + by * by
    // ax ^ 2 + ay ^ 2 + bx ^ 2 + by ^ 2 - 2 * ax * bx - 2 * ay * by
    // ax ^ 2 + ay ^ 2 + bx ^ 2 + by ^ 2 - 2 * (ax * bx + ay * by)
    // let ingress_sq = a.position_pre_calc + b.position_pre_calc - a.position_double.x * b.position.x - a.position_double.y * b.position.y;
    // let ingress_sq = (a.position - b.position).mag2();
    // let abr = a.radius + b.radius;
    // let radius_sq = (a.radius + b.radius) * (a.radius + b.radius);
    // let true_ingress = (a.radius + b.radius) * (a.radius + b.radius) - ingress_sq;
    // let true_ingress = a.radius2 + b.radius2 - ingress_sq;
    if (a.radius + b.radius) * (a.radius + b.radius) - (a.position - b.position).mag2() > 0.0 {
    // if a.radius2 + b.radius2 > ingress_sq {
    // if a.pressure_radius2 + b.pressure_radius2 > ingress_sq {
      // let a = self.a.unwrap();
      // let b = self.b.unwrap();
      // let pos_diff = a.position - b.position;
      let pos_diff = a.position - b.position;
      let ingress_sq = (a.position - b.position).mag2();
      let ingress = ingress_sq.sqrt() + EPSILON + (a.radius + b.radius) / 50.0;
      // let factor = a.factor * b.factor;
      let pqt = (((a.radius + b.radius) / ingress).powf(1.5) - 1.0f32);
      // let pqt = (radius_sq / ingress_sq - 1.0f32);
      // let pqt = (a.radius + b.radius - ingress);
      // let pqt = true_ingress.sqrt() + EPSILON;
      // let pqt = ((a.pressure_radius + b.pressure_radius) / ingress - 1f32) * factor;
      // let lamb = pos_diff.scale(pqt);
      // let lamb = pos_diff.div_scale(pqt);
      let total_mass = (a.mass + b.mass) / pqt;
      // (am + bm) * 1 / ((ar + br) / i - 1) * 0.25
      //             1 / (ar + br - 1 / i) * 1 / i
      // (am + bm) * (1 / ar + 1 / br + -i) * 1 / i
      // (am / ar + bm / ar + am / br + bm / br - am * i - bm * i) / i
      // (amr + bmr + bm / ar + am / br - (am + bm) * i) / i
      let a_mass_coeff = b.mass / total_mass;
      let b_mass_coeff = a.mass / total_mass;
      // let total_mass = a.pressure_mass + b.pressure_mass;
      // let a_mass_coeff = b.pressure_mass / total_mass;
      // let b_mass_coeff = a.pressure_mass / total_mass;
      // let friction = 1.0 - a.friction * b.friction;
      // let av = (a.last_position - a.position).scale(friction);
      // let bv = (b.last_position - b.position).scale(friction);
      // let av = a.velocity.scale(friction);
      // let bv = b.velocity.scale(friction);
      // let av = a.velocity;
      // let bv = b.velocity;

      // ax * amc
      // ax * (am / (am + bm) * 1 / pqt)
      // ax * (1 / pqt) * (am / tm)
      // ax * (am / (tm * pqt))
      // ax * amc, ay * amc

      // let lamb = pos_diff.div_scale();

      Some((
        pos_diff.scale(a_mass_coeff),
        // a.position + av,
        pos_diff.scale(-b_mass_coeff),
        // b.position + bv,
        ingress_sq
      ))
      // Some(Collision {
      //   // a_ddvt_index: a_index, b_ddvt_index: b_index,
      //   ingress: ingress, pqt: 0f32 })
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
    let mut c: Collision = Default::default();
    let a = Particle::at(Default::default());
    let b = Particle::at(V2 { x: 0.5f32, y: 0f32 });
    assert!(c.test(&a, &b, 0u32, 1u32));
    assert_eq!(c.a_ddvt_index, 0u32);
    assert_eq!(c.b_ddvt_index, 1u32);
    assert_eq!(c.ingress, 0.25f32);
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
