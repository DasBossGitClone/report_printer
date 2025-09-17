#![feature(trivial_bounds)]
#![deny(unused)]
#![allow(dead_code)]

mod builder;
mod printer;
pub use builder::{ReportBuilder, ChildLabel, IntoRange, Label, RangeInclusive, Error};
use printer::*;
mod find_iter;
pub use ::token::{AnsiStyle, Color, RgbColor, Style};
pub(crate) use find_iter::*;
