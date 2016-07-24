#[macro_use]
pub mod v2;
#[macro_use]
pub mod bb;
mod base5;

pub type V2 = v2::V2;
pub type BB = bb::BB;
pub type b5 = base5::b5;

// #[macro_use]
// pub use bb_macro::*;

// impl V2 {
//   #[inline]
//   pub fn zero() -> V2 { v2::V2::zero() }
// }

// pub mod v2::V2 as V2;
// pub use self::v2::V2 as V2;
// pub struct v2::V2;
