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
pub use carets::{set_arrow_label_padding, set_child_label_offset};

impl Report {
    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        self.report_labels
            .write(writer, &self.input, self.colored_input, self.display_range)
    }

    pub fn into_writer<'a, W: Write>(&'a self, writer: &'a mut W) -> ReportWriter<'a, W> {
        self.report_labels
            .into_writer(writer, &self.input, self.display_range)
    }
    pub fn into_writer_with<
        'a,
        W: Write,
        D: Display,
        F: FnMut(io::Result<()>, ReportWriterMeta) -> io::Result<Option<D>>,
    >(
        &'a self,
        writer: &'a mut W,
        callback: F,
    ) -> ReportWriterWith<'a, W, D, F> {
        self.report_labels
            .into_writer_with(writer, &self.input, self.display_range, callback)
    }
}
