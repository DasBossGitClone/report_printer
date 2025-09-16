#![feature(iter_array_chunks)]
#![allow(dead_code)]
#![deny(unused)]

use ::itertools::Itertools;
use ::std::{fmt::Display, str::FromStr};
mod colors_chars;
pub use colors_chars::*;

mod token;
pub use token::*;
mod single_line_stream;
pub use single_line_stream::*;
mod multiline_stream;
pub use multiline_stream::*;
