use ::std::{
    fmt::Display,
    io::{self, Write},
    str::FromStr,
};

use ::itertools::Itertools;
use ::token::*;

use crate::{Find, FindRev, builder::*};

mod builder;
mod carets;
mod segments;
mod underbar;
pub(super) use builder::*;
pub(crate) use carets::*;
pub(crate) use segments::*;
pub(crate) use underbar::*;

#[allow(non_snake_case)]
pub fn SPACE(c: usize) -> String {
    " ".repeat(c)
}
#[allow(non_snake_case)]
pub fn REP(c: usize, s: &str) -> String {
    s.repeat(c)
}

/* #[derive(Debug, Clone)]
struct ChildLine {
    indent: usize,
    /// Arrow
    arrow: String,
    /// The raw uncolored characters of the line
    msg: String,
    /// The ranges of colors to apply to the line
    colors: Vec<(RangeInclusive, Color)>,
}

impl Display for ChildLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ChildLine {
            indent,
            arrow,
            msg,
            colors,
        } = self;

        let mut current_pos = 0;

        write!(f, "{}", " ".repeat(*indent))?;
        write!(f, "{}", arrow)?;

        for (range, color) in colors {
            // Write the text before the colored range
            if range.start() > current_pos {
                write!(f, "{}", &msg[current_pos..range.start()])?;
            }
            // Write the colored range
            write!(
                f,
                "{color}{msg}{COLOR_RESET}",
                color = color.to_ansi_fg(),
                msg = &msg[range.start()..=(msg.len() - 1.min(range.end()))],
            )?;
            current_pos = range.end().saturating_add(1);
        }
        // Write any remaining text after the last colored range
        if current_pos < msg.len() {
            write!(f, "{}", &msg[current_pos..])?;
        }

        Ok(())
    }
} */

#[derive(Debug, Clone, thiserror::Error)]
pub enum ColorizationError {
    #[error("Color range {attempted:?} is out of bounds for string of length {valid_length}.")]
    OutOfBounds {
        valid_length: usize,
        attempted: RangeInclusive,
    },
}

#[derive(Debug, Clone, derive_more::From, derive_more::Into, derive_more::IntoIterator)]
pub struct ColorRanges {
    pub ranges: Vec<(RangeInclusive, Color)>,
}
impl ColorRanges {
    pub fn new() -> Self {
        Self { ranges: vec![] }
    }
    pub fn single_color<I: IntoRange>(range: I, color: Color) -> Self {
        Self {
            ranges: vec![(range.into_range(), color)],
        }
    }
    pub fn push<I: IntoRange>(&mut self, range: I, color: Color) {
        self.ranges.push((range.into_range(), color));
    }
}

/* #[derive(Debug, Clone)]
pub struct ColoredString<A: AsRef<str>> {
    pub raw: A,
    pub colors: ColorRanges,
}
impl<A: AsRef<str>> ColoredString<A> {
    pub fn colorize(&self) -> Result<String, ColorizationError> {
        let source = self.raw.as_ref();
        let mut current_pos = 0;
        let mut out_string = String::new();
        let source_len = source.len();

        for (range, color) in &self.colors.ranges {
            if range.start() >= source_len || range.end() >= source_len {
                return Err(ColorizationError::OutOfBounds {
                    valid_length: source_len,
                    attempted: range.clone(),
                });
            }
            // Write the text before the colored range
            if range.start() > current_pos {
                out_string.push_str(&source[current_pos..range.start()]);
            }
            // Write the colored range
            out_string.push_str(&format!(
                "{color}{msg}{COLOR_RESET}",
                msg = &source[range.start()..=(range.end()).min(source_len - 1)],
            ));
            current_pos = range.end().saturating_add(1);
        }
        // Write any remaining text after the last colored range
        if current_pos < source_len {
            out_string.push_str(&source[current_pos..]);
        }

        Ok(out_string)
    }
} */

#[derive(Debug, Clone)]
struct LabelLine {
    /// The absolut (irrelevant of offset) starting position of the underbar (inclusive)
    underbar_start: usize,
    /// The range of the underbar
    underbar_range: RangeInclusive,
    /// The text of the label
    msg: LineTokenStream,
    /// The ranges of colors to apply to the line
    colors: ColorRanges,
    /// The child lines to print below this label
    child_lines: Vec<TokenizedChildLabel>,
}

impl ArgumentErrorReport {
    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        self.write_internal(writer)
    }

    fn write_internal<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let ArgumentErrorReport {
            input_label_offset,
            raw_input,
            input,
            // Labels already sorted in the builder
            mut labels,
        } = self;

        let label_lines: Vec<LabelLine> =
            Self::gather_lines(labels.len(), labels.into_iter(), input_label_offset);

        let caret_segments = Self::generate_underbar(label_lines)?;

        caret_segments.into_iter().try_for_each(|cs| {
            if let Some(formatted) = cs.format() {
                write!(writer, "{}", formatted.underbar_lines())?;
                formatted.into_iter().try_for_each(|cl| {
                    writeln!(writer, "{}", cl.main())?;
                    if let Some(sep) = cl.separator() {
                        return writeln!(writer, "{}", sep);
                    }
                    Ok(())
                })
            } else {
                Ok(())
            }
        })?;

        Ok(())
    }
}
impl ArgumentErrorReport {
    fn gather_lines(
        size_hint: usize,
        labels: impl Iterator<Item = TokenizedLabel>,
        offset: usize,
    ) -> Vec<LabelLine> {
        let mut colors = colors();

        let mut label_lines = Vec::with_capacity(size_hint);

        for label in labels {
            let TokenizedLabel {
                range,
                message,
                child_labels,
            } = label;

            let start = range.start().saturating_sub(offset);
            let end = range.end().saturating_sub(offset);

            let underbar_range = (start..=end).into();
            let underbar_len = end.saturating_sub(start + 1).max(1);

            // The underbar tree split (┬) usually starts at the 3rd character to the right of the start
            // Check if that is possible, otherwise just start in the middle
            let down_start = if start + 3 < end {
                2
            } else {
                (underbar_len / 2).saturating_sub(1)
            };

            let forward_label_indent = down_start + start;

            let child_labels_len = child_labels.len();

            let mut child_lines = Vec::with_capacity(child_labels_len);

            child_labels
                .into_iter()
                .enumerate()
                .map(|(i, child_label)| {
                    let is_last = i + 1 == child_labels_len;

                    TokenizedChildLabel {
                        //indent: forward_label_indent + 2,
                        /* arrow: format!(
                            "{transition}{H_CARET}{H_CARET}{H_CARET}{L_ARROW}",
                            transition = if is_last { UP_RIGHT } else { V_RIGHT },
                        ), */
                        message: child_label.message,
                    }
                })
                .for_each(|child_line| child_lines.push(child_line));

            let label_line = LabelLine {
                underbar_start: if offset != 0 { start + 4 } else { start },
                underbar_range,
                colors: ColorRanges::single_color(0..=message.len(), color),
                msg: LineTokenStream::from_str(&message).expect("Failed to parse label message"),
                child_lines,
            };

            label_lines.push(label_line);
        }
        label_lines
    }
}
