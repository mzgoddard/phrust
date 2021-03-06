use std::ops::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct B5 { value: u32 }

impl B5 {
  pub fn from(value: u32) -> B5 {
    B5 { value: value }
  }

  pub fn value(self) -> u32 {
    self.value
  }

  pub fn length(self) -> usize {
    if self.value != 0 {
      (((self.value as f32 * 5f32).log(5f32)).floor() as usize)
    }
    else {
      0
    }
  }

  pub fn highest_bit(self) -> B5 {
    self >> (self.length() - 1)
  }
}

#[test]
fn length_test() {
  assert_eq!(B5::from(0).length(), 0);
  assert_eq!(B5::from(1).length(), 1);
  assert_eq!(B5::from(4).length(), 1);
  assert_eq!(B5::from(5).length(), 2);
  assert_eq!(B5::from(24).length(), 2);
  assert_eq!(B5::from(25).length(), 3);
}

#[test]
fn highest_bit_test() {
  assert_eq!(B5::from(1).highest_bit(), B5::from(1));
  assert_eq!(B5::from(2).highest_bit(), B5::from(2));
  assert_eq!(B5::from(3).highest_bit(), B5::from(3));
  assert_eq!(B5::from(4).highest_bit(), B5::from(4));
  assert_eq!(B5::from(5).highest_bit(), B5::from(1));
}

impl Add for B5 {
  type Output = B5;

  fn add(self, other: B5) -> B5 {
    B5{value: self.value + other.value}
  }
}

#[test]
fn add_test() {
  assert_eq!(B5::from(1) + B5::from(2), B5::from(3))
}

const MULTIPLES: [u32; 14] = [
  1, 5, 25, 125, 625, 3125, 15625, 78125, 390625, 1953125, 9765625, 48828125, 244140625, 1220703125
];

impl Shl<usize> for B5 {
  type Output = B5;

  fn shl(self, other: usize) -> B5 {
    B5{value: self.value * MULTIPLES[other]}
  }
}

#[test]
fn shl_test() {
  assert_eq!(B5::from(1) << 1, B5::from(5));
  assert_eq!(B5::from(5) << 2, B5::from(125))
}

impl Shr<usize> for B5 {
  type Output = B5;

  fn shr(self, other: usize) -> B5 {
    B5{value: self.value / MULTIPLES[other]}
  }
}

#[test]
fn shr_test() {
  assert_eq!(B5::from(5) >> 1, B5::from(1));
  assert_eq!(B5::from(25) >> 2, B5::from(1))
}
