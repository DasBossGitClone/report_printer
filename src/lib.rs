#![feature(trivial_bounds, iter_map_windows)]
#![deny(dead_code, unused)]

mod builder;
mod printer;
use printer::*;
mod find_iter;
pub(crate) use find_iter::*;

pub use ::token::{AnsiStyle, Color, RgbColor, Style};
pub use builder::{ChildLabel, Error, IntoRange, Label, RangeInclusive, ReportBuilder};
