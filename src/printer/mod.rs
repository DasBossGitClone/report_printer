use ::std::{
    fmt::Display,
    io::{self, Write},
};

use ::itertools::Itertools;
use ::token::*;

use crate::{Find, FindRev, builder::*};

mod builder;
mod carets;
mod underbar;
pub(super) use builder::*;
pub(crate) use carets::*;

impl ArgumentErrorReport {
    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        self.write_internal(writer)
    }

    fn write_internal<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let caret_segments = self.generate_underbar()?;

        caret_segments.write(writer, &self.input, self.display_range)?;

        Ok(())
    }
}
