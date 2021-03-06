use std::ops::*;
use std::f32;
use std::f32::EPSILON;
use std::fmt;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct V2 {
  pub x: f32,
  pub y: f32
}

#[macro_export]
macro_rules! v2 {
  ($x:expr, $y:expr) => { {
    V2 {x: ($x) as f32, y: ($y) as f32}
  } }
}


macro_rules! impl_binary_op {
    ($t:ty, $n:ident) => (
        // #[stable]
        impl $t for V2 {
            type Output = V2;

            #[inline]
            fn $n(self, other: V2) -> V2 {
              macro_rules! impl_op_op {
                (add) => ( V2 { x: self.x + other.x, y: self.y + other.y } );
                (sub) => ( V2 { x: self.x - other.x, y: self.y - other.y } );
                (mul) => ( V2 { x: self.x * other.x, y: self.y * other.y } );
                (div) => ( V2 { x: self.x / other.x, y: self.y / other.y } );
                (rem) => ( V2 { x: self.x % other.x, y: self.y % other.y } );
              }
              impl_op_op!($n)
            }
        }
    )
}

impl_binary_op!{ Add, add }
impl_binary_op!{ Sub, sub }
impl_binary_op!{ Mul, mul }
impl_binary_op!{ Div, div }
impl_binary_op!{ Rem, rem }

macro_rules! impl_assign_op {
    ($t:ty, $n:ident) => (
        // #[stable]
        impl $t for V2 {
            #[inline]
            fn $n(&mut self, other: V2) {
              macro_rules! impl_op_op {
                (add_assign) => ( { self.x += other.x; self.y += other.y; } );
                (sub_assign) => ( { self.x -= other.x; self.y -= other.y; } );
                (mul_assign) => ( { self.x *= other.x; self.y *= other.y; } );
                (div_assign) => ( { self.x /= other.x; self.y /= other.y; } );
                (rem_assign) => ( { self.x %= other.x; self.y %= other.y; } );
              }
              impl_op_op!($n)
            }
        }
    )
}

impl_assign_op!{ AddAssign, add_assign }
impl_assign_op!{ SubAssign, sub_assign }
impl_assign_op!{ MulAssign, mul_assign }
impl_assign_op!{ DivAssign, div_assign }
impl_assign_op!{ RemAssign, rem_assign }

macro_rules! impl_unary_op {
    ($t:ty, $n:ident) => (
        // #[stable]
        impl $t for V2 {
            type Output = V2;

            #[inline]
            fn $n(self: V2) -> V2 {
              macro_rules! impl_op_op {
                (neg) => ( V2 { x: -self.x, y: -self.y } );
              }
              impl_op_op!($n)
            }
        }
    )
}

impl_unary_op!{ Neg, neg }

pub const V2_ZERO : V2 = V2 { x: 0f32, y: 0f32 };

pub fn v2_zero() -> V2 {
  V2::zero()
}

impl V2 {
  // pub const ZERO : V2 = V2 { x: 0f32, y: 0f32 };

  #[inline]
  pub fn zero() -> V2 {
    V2 { x: 0f32, y: 0f32 }
  }

  pub fn infinity() -> V2 {
    V2 { x: f32::INFINITY, y: f32::INFINITY }
  }

  #[inline]
  pub fn scale(self: V2, other: f32) -> V2 {
    V2 { x: self.x * other, y: self.y * other }
  }

  #[inline(always)]
  pub fn scale_add(self: V2, scale: f32, add: V2) -> V2 {
    V2 { x: self.x * scale + add.x, y: self.y * scale + add.y }
    // V2 { x: self.x.mul_add(scale, add.x), y: self.y.mul_add(scale, add.y) }
  }

  #[inline]
  pub fn sq(self: V2) -> V2 {
    V2 { x: self.x * self.x, y: self.y * self.y }
  }

  #[inline]
  pub fn powi(self: V2, i: i32) -> V2 {
    V2 { x: self.x.powi(i), y: self.y.powi(i) }
  }

  pub fn div_scale(self, other: f32) -> V2 {
    V2 { x: self.x / other, y: self.y / other }
  }

  #[inline]
  pub fn dot(self, other: V2) -> f32 {
    self.x * other.x + self.y * other.y
  }

  #[inline]
  pub fn cross(self, other: V2) -> f32 {
    self.x * other.y - self.y * other.x
  }

  #[inline]
  pub fn project(self, other: V2) -> V2 {
    other.scale(self.dot(other) / other.mag2())
  }

  #[inline]
  pub fn project_scale_add(self, other: V2, scale: f32, add: V2) -> V2 {
    other.scale_add(self.dot(other) / other.mag2() * scale, add)
  }

  #[inline]
  pub fn mag2(self: V2) -> f32 {
    self.x * self.x + self.y * self.y
    // let x = self.x;
    // let y = self.y;
    // x * x + y * y
  }

  #[inline]
  pub fn mag(self: V2) -> f32 {
    self.x.hypot(self.y)
    // (self.x * self.x + self.y * self.y).sqrt()
    // self.mag2().sqrt()
  }

  #[inline]
  pub fn rmag(self: V2) -> f32 {
    1.0 / (self.x * self.x + self.y * self.y + EPSILON).sqrt()
    // 1.0 / self.x.hypot(self.y)
  }

  #[inline]
  pub fn dist2(self, other: V2) -> f32 {
    (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    // let x = self.x - other.x;
    // let y = self.y - other.y;
    // x * x + y * y
  }

  #[inline]
  pub fn dist(self, other: V2) -> f32 {
    (self.x - other.x).hypot(self.y - other.y)
    // self.dist2(other).sqrt()
  }

  #[inline]
  pub fn rdist(self, other: V2) -> f32 {
    1.0 / ((self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y) + EPSILON).sqrt()
  }

  #[inline]
  pub fn unit(self: V2) -> V2 {
    self.scale(self.rmag())
  }
}

impl fmt::Display for V2 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "v2!({}, {})", self.x, self.y)
  }
}

#[cfg(test)]
mod test {
  use super::V2;

  #[test]
  fn add() {
    assert_eq!(V2 { x: 1f32, y: 2f32 } + V2 { x: 2f32, y: 1f32 }, V2 { x: 3f32, y: 3f32 })
  }

  #[test]
  fn sub() {
    assert_eq!(V2 { x: 1f32, y: 2f32 } - V2 { x: 2f32, y: 1f32 }, V2 { x: -1f32, y: 1f32 })
  }

  #[test]
  fn mul() {
    assert_eq!(V2 { x: 1f32, y: 2f32 } * V2 { x: 2f32, y: 1f32 }, V2 { x: 2f32, y: 2f32 })
  }

  #[test]
  fn div() {
    assert_eq!(V2 { x: 1f32, y: 2f32 } / V2 { x: 2f32, y: 1f32 }, V2 { x: 0.5f32, y: 2f32 })
  }

  #[test]
  fn rem() {
    assert_eq!(V2 { x: 1f32, y: 2f32 } % V2 { x: 2f32, y: 1f32 }, V2 { x: 1f32, y: 0f32 })
  }

  #[test]
  fn neg() {
    assert_eq!(-V2 { x: 1f32, y: 2f32 }, V2 { x: -1f32, y: -2f32 })
  }
}

#[cfg(bench)]
mod bench {
  extern crate test;
  use self::test::Bencher;
  use super::V2;

  #[bench]
  fn b_add(b: &mut Bencher) {
    let a1 = V2 { x: 1f32, y: 2f32 };
    let a2 = V2 { x: 2f32, y: 1f32 };
    b.iter(|| {
      let mut s = a1;
      for i in 0..1000000 {
        s = s + V2 { x: 2f32, y: 1f32 };
      }
      s
    });
  }

  #[bench]
  fn b_mul_add_op(b: &mut Bencher) {
    let a1 = V2 { x: 1f32, y: 2f32 };
    let a2 = V2 { x: 2f32, y: 1f32 };
    let a3 = V2 { x: 1f32, y: 1f32 };
    b.iter(|| {
      let mut s = a1;
      for i in 0..1000000 {
        s = s * a2 + a3;
      }
      s
    });
  }

  #[bench]
  fn b_dot(b: &mut Bencher) {
    let a1 = V2 { x: 1f32, y: 2f32 };
    let a2 = V2 { x: 2f32, y: 1f32 };
    b.iter(|| {
      let mut s = a1;
      for i in 0..1000000 {
        s.x = s.dot(a2);
      }
      s
    });
  }
}
