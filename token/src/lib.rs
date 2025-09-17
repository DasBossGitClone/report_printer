#![feature(iter_array_chunks)]
#![deny(dead_code, unused)]

use ::itertools::Itertools;
use ::misc_extensions::bool::*;
use ::std::{
    fmt::{Debug, Display},
    str::FromStr,
};

mod colors_chars;
pub use colors_chars::*;
mod token;
pub use token::*;
mod single_line_stream;
pub use single_line_stream::*;
mod multiline_stream;
pub use multiline_stream::*;
