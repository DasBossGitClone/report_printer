#![feature(trivial_bounds, iter_map_windows)]
#![deny(dead_code, unused)]

mod builder;
mod printer;
use printer::*;
mod find_iter;
pub(crate) use find_iter::*;

pub use ::token::{AnsiStyle, Color, RgbColor, Style, impl_field};
pub use builder::{ChildLabel, Error, IntoRange, Label, RangeInclusive, ReportBuilder};
pub mod config {
    use super::printer;
    pub use printer::set_arrow_label_padding;
    pub use printer::set_child_label_offset;
}
