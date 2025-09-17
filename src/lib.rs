#![feature(trivial_bounds)]
#![deny(dead_code, unused)]

mod builder;
mod printer;
pub use builder::{ChildLabel, Error, IntoRange, Label, RangeInclusive, ReportBuilder};
use printer::*;
mod find_iter;
pub use ::token::{AnsiStyle, Color, RgbColor, Style};
pub(crate) use find_iter::*;
