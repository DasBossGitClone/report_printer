#![feature(trivial_bounds, iter_map_windows)]
#![deny(dead_code, unused)]

mod builder;
mod printer;
use printer::*;
/// A module for finding patterns in text
///
/// Provides a forward and backward iterator for finding substrings
/// in a given text.
///
/// Its just exported here, as it'd be a shame to have it, but not use it.
pub mod find_iter;
pub(crate) use find_iter::*;

pub use ::token::{AnsiStyle, Color, RgbColor, Style, impl_field};
pub use builder::{ChildLabel, Error, IntoRange, Label, RangeInclusive, ReportBuilder, Trim, TrimPadding};
pub mod config {
    use super::printer;
    pub use printer::set_arrow_label_padding;
    pub use printer::set_child_label_offset;
}

#[cfg(feature = "truncate_out_of_bounds")]
pub use builder::TruncateMode;