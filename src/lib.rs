// extern crate time;
// extern crate rand;
//
// use time::get_time;
// use rand::{random, Open01};

#[macro_export]
#[macro_use]
pub mod math;
pub mod particle;
pub mod collision;
mod ddvt;
mod ddvt_bench;
mod quad_tree;
pub mod world;
pub mod world_renderer;

use std::f32;
use std::f64;

// #[macro_use]
// use math;
// #[macro_use]
// use math::*;
