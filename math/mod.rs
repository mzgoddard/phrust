mod v2;

pub type V2 = v2::V2;

impl V2 {
  #[inline]
  pub fn zero() -> V2 { v2::V2::zero() }
}

// pub mod v2::V2 as V2;
// pub use self::v2::V2 as V2;
// pub struct v2::V2;
