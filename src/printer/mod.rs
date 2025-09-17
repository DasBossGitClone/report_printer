use ::std::{
    fmt::Display,
    io::{self, Write},
};

use ::token::*;

use crate::{Find, FindRev, builder::*};

mod builder;
mod carets;
mod underbar;
pub(super) use builder::*;
pub(crate) use carets::*;

impl Report {
    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        self.report_labels
            .write(writer, &self.input, self.display_range)
    }

    pub fn into_writer<'a, W: Write>(&'a self, writer: &'a mut W) -> ReportWriter<'a, W> {
        self.report_labels
            .into_writer(writer, &self.input, self.display_range)
    }
}
