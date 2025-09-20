use ::std::fmt::Display;
#[cfg(feature = "caret_color")]
use ::token::RgbColor;
use ::token::{AnsiStyle, LineTokenStream, saturating::SaturatingArithmetic};

use crate::{Report, TokenizedChildLabel, TokenizedLabelFull};

pub const CHILD_LABEL_PADDING: usize = 4;

pub trait IntoRange {
    fn into_range(self) -> RangeInclusive;
}
impl IntoRange for RangeInclusive {
    fn into_range(self) -> RangeInclusive {
        self
    }
}
impl IntoRange for std::ops::Range<usize> {
    fn into_range(self) -> RangeInclusive {
        (self.start..=self.end.saturating_sub(1)).into()
    }
}
impl IntoRange for std::ops::RangeInclusive<usize> {
    fn into_range(self) -> RangeInclusive {
        self.into()
    }
}
impl IntoRange for std::ops::RangeFrom<usize> {
    fn into_range(self) -> RangeInclusive {
        (self.start..=usize::MAX).into()
    }
}
impl IntoRange for std::ops::RangeTo<usize> {
    fn into_range(self) -> RangeInclusive {
        (0..=self.end.saturating_sub(1)).into()
    }
}
impl IntoRange for std::ops::RangeFull {
    fn into_range(self) -> RangeInclusive {
        (0..=usize::MAX).into()
    }
}
macro_rules! impl_into_range_for_primitive {
    ($($t:ty),*) => {
        $(
            impl IntoRange for $t {
                fn into_range(self) -> RangeInclusive {
                    RangeInclusive {
                        start: self as usize,
                        end: self as usize,
                    }
                }
            }
        )*
    };
}
impl_into_range_for_primitive!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RangeInclusive {
    pub start: usize,
    pub end: usize,
}
impl PartialOrd for RangeInclusive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.start
                .cmp(&(*other).start)
                .then(self.end.cmp(&other.end)),
        )
    }
}
impl Ord for RangeInclusive {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start).then(self.end.cmp(&other.end))
    }
}
impl From<std::ops::RangeInclusive<usize>> for RangeInclusive {
    fn from(r: std::ops::RangeInclusive<usize>) -> Self {
        Self {
            start: *r.start(),
            end: *r.end(),
        }
    }
}
impl RangeInclusive {
    pub fn contains(&self, index: usize) -> bool {
        index >= self.start && index <= self.end
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start).sat_add(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn max_end(&self, other: usize) -> Self {
        if other > self.end {
            Self {
                start: self.start,
                end: other,
            }
        } else {
            *self
        }
    }

    pub fn max_start(&self, other: usize) -> Self {
        if other > self.start {
            Self {
                start: other,
                end: self.end,
            }
        } else {
            *self
        }
    }

    pub fn min_end(&self, other: usize) -> Self {
        if other < self.end {
            Self {
                start: self.start,
                end: other,
            }
        } else {
            *self
        }
    }

    pub fn min_start(&self, other: usize) -> Self {
        if other < self.start {
            Self {
                start: other,
                end: self.end,
            }
        } else {
            *self
        }
    }

    #[cfg(feature = "truncate_out_of_bounds")]
    pub fn truncate_end(&mut self, max: usize) -> bool {
        if self.end > max {
            self.end = max;
            true
        } else {
            false
        }
    }
}

impl Display for RangeInclusive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{} - {}", self.start, self.end)
        } else {
            if self.start == self.end {
                write!(f, "{}", self.start)
            } else {
                write!(f, "{}..={}", self.start, self.end)
            }
        }
    }
}

impl FromIterator<usize> for RangeInclusive {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let start = iter.next().unwrap_or(0);
        let end = iter.next().unwrap_or(start);
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg(feature = "truncate_out_of_bounds")]
#[repr(u8)]
pub enum TruncateMode {
    None,
    Silent,
    Indicate,
}

#[derive(Debug, Clone)]
pub struct ReportBuilder {
    /// Will only display the relevant part of the input if true
    trim_input: bool,
    /// The full input string thats referenced by the labels
    input: String,
    /// The labels to annotate the input with
    labels: Vec<Label>,
    display_range: bool,
    max_label_length: usize,
    /// If not set, it will be set to "max_label_length - CHILD_LABEL_PADDING" to offset the padding on the child labels
    max_child_label_length: Option<usize>,
    colored_input: bool,
    #[cfg(feature = "caret_color")]
    caret_color: bool,
    #[cfg(feature = "truncate_out_of_bounds")]
    truncate_out_of_bounds: TruncateMode,
}

impl ReportBuilder {
    pub fn new<I: Into<String>>(input: I) -> Self {
        Self {
            trim_input: true,
            display_range: false,
            input: input.into(),
            labels: Vec::new(),
            // Default max label length is 30 characters
            max_label_length: 30,
            max_child_label_length: None,
            colored_input: false,
            #[cfg(feature = "caret_color")]
            caret_color: false,
            #[cfg(feature = "truncate_out_of_bounds")]
            truncate_out_of_bounds: TruncateMode::Silent,
        }
    }

    #[cfg(feature = "truncate_out_of_bounds")]
    pub fn truncate_out_of_bounds<I: Into<TruncateMode>>(mut self, truncate: I) -> Self {
        self.truncate_out_of_bounds = truncate.into();
        self
    }

    #[cfg(feature = "caret_color")]
    pub fn caret_color(mut self, colored: bool) -> Self {
        self.caret_color = colored;
        self
    }

    pub fn colored_input(mut self, colored: bool) -> Self {
        self.colored_input = colored;
        self
    }

    pub fn with_range(self) -> Self {
        Self {
            display_range: true,
            ..self
        }
    }

    pub fn max_label_length(&mut self, length: usize) -> &mut Self {
        self.max_label_length = length;
        if self.max_child_label_length.is_none() {
            self.max_child_label_length = Some(length + CHILD_LABEL_PADDING);
        }
        self
    }

    pub fn max_child_label_length(&mut self, length: usize) -> &mut Self {
        self.max_child_label_length = Some(length);
        self
    }

    pub fn trim_input(&mut self, trim: bool) -> &mut Self {
        self.trim_input = trim;
        self
    }

    pub fn with_label(&mut self, label: Label) -> &mut Self {
        self.labels.push(label);
        self
    }

    pub fn push<I: Into<Label>>(&mut self, label: I) -> &mut Self {
        self.labels.push(label.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    /// The range in the input string that this label annotates
    pub(super) range: RangeInclusive,
    /// If no colors is set, it will be generated at runtime
    pub(super) message: String,
    /// Optional child labels for more detailed annotations
    /// or if a message would repeat too much
    pub(super) child_labels: Vec<ChildLabel>,
    pub(super) color: Option<Vec<AnsiStyle>>,
    #[cfg(feature = "caret_color")]
    pub(super) caret_color: Option<RgbColor>,
}
impl Label {
    pub fn new<I: Display, R: IntoRange>(range: R, message: I) -> Self {
        Self {
            range: range.into_range(),
            message: message.to_string(),
            child_labels: Vec::new(),
            color: None,
            #[cfg(feature = "caret_color")]
            caret_color: None,
        }
    }

    /// Replaces the current message
    pub fn with_message<I: Display>(mut self, message: I) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn with_color<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.color.get_or_insert_with(Vec::new).push(style.into());
        self
    }

    pub fn with_child_label(mut self, child_label: ChildLabel) -> Self {
        self.child_labels.push(child_label);
        self
    }

    pub fn push<I: Into<ChildLabel>>(&mut self, child_label: I) -> &mut Self {
        self.child_labels.push(child_label.into());
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get_message(&self) -> String {
        let Self { message, color, .. } = self;
        let message = message.clone();
        if let Some(color) = color {
            // Wrap the message in color codes
            color.into_iter().fold(message, |msg, c| c.with_color(&msg))
        } else {
            message
        }
    }

    #[allow(dead_code)]
    pub(crate) fn into_message(self) -> String {
        let Self { message, color, .. } = self;
        if let Some(color) = color {
            // Wrap the message in color codes
            color.into_iter().fold(message, |msg, c| c.with_color(&msg))
        } else {
            message
        }
    }
    #[cfg(feature = "caret_color")]
    pub fn with_caret_color<I: Into<RgbColor>>(mut self, style: I) -> Self {
        self.caret_color = Some(style.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct ChildLabel {
    /// If no colors is set, it will be generated at runtime
    pub(super) message: String,
    pub(super) color: Option<Vec<AnsiStyle>>,
    #[cfg(feature = "caret_color")]
    pub(super) caret_color: Option<RgbColor>,
}

impl ChildLabel {
    pub fn new<I: Display>(message: I) -> Self {
        Self {
            message: message.to_string(),
            color: None,
            #[cfg(feature = "caret_color")]
            caret_color: None,
        }
    }

    pub fn with_color<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.color.get_or_insert_with(Vec::new).push(style.into());
        self
    }

    pub fn reset_color(mut self) -> Self {
        self.color = None;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get_message(&self) -> String {
        let Self { message, color, .. } = self;
        let message = message.clone();
        if let Some(color) = color {
            // Wrap the message in color codes
            color.into_iter().fold(message, |msg, c| c.with_color(&msg))
        } else {
            message
        }
    }

    #[allow(dead_code)]
    pub(crate) fn into_message(self) -> String {
        let Self { message, color, .. } = self;
        if let Some(color) = color {
            // Wrap the message in color codes
            color.into_iter().fold(message, |msg, c| c.with_color(&msg))
        } else {
            message
        }
    }
    #[cfg(feature = "caret_color")]
    pub fn with_caret_color<I: Into<RgbColor>>(mut self, style: I) -> Self {
        self.caret_color = Some(style.into());
        self
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("No labels were added to the report")]
    NoLabels,
    #[error("A label has an empty message")]
    LabelEmptyMessage,
    #[error("A child label has an empty message")]
    LabelChildEmptyMessage,
    #[error("The input string is empty")]
    EmptyInput,
    /// (given range, valid range)
    #[error(
        "A label has a range that is out of bounds: {valid:?} not contained within {attempted_range:?}"
    )]
    #[cfg(not(feature = "truncate_out_of_bounds"))]
    OutOfBounds {
        valid: RangeInclusive,
        attempted_range: RangeInclusive,
    },
}

impl ReportBuilder {
    /// Validates the current state of the builder and generates the Report.
    /// Returns a BuilderError if the state is invalid, but does not consume Self.
    /// This allows one to fix the issues and try again without reinstantiating
    /// the builder (so its a sorta "soft-fail").
    pub fn finish(&self) -> Result<Report, Error> {
        // This verification allows us to carelessly use the ranges later on
        if self.labels.is_empty() {
            return Err(Error::NoLabels);
        }
        if self.input.is_empty() {
            return Err(Error::EmptyInput);
        }

        let valid_range = 0..=self.input.len();
        #[cfg(feature = "truncate_out_of_bounds")]
        let mut needs_truncate = false;
        self.labels.iter().try_for_each(|label| {
            if label.range.start() < *valid_range.start() || label.range.end() > *valid_range.end()
            {
                #[cfg(not(feature = "truncate_out_of_bounds"))]
                {
                    return Err(Error::OutOfBounds {
                        attempted_range: label.range.clone(),
                        valid: valid_range.clone().into(),
                    });
                }
                #[cfg(feature = "truncate_out_of_bounds")]
                {
                    needs_truncate = true;
                }
            }
            if label.message.is_empty() {
                return Err(Error::LabelEmptyMessage);
            }
            label.child_labels.iter().try_for_each(|child_label| {
                if child_label.message.is_empty() {
                    return Err(Error::LabelChildEmptyMessage);
                }
                Ok(())
            })?;
            Ok(())
        })?;

        // Offset to apply to label ranges when trimming the input
        let mut input_label_offset = 0;

        let input = if self.trim_input {
            let (trimmed_input, offset) = Report::trim_input(
                &self.input,
                self.labels.iter(),
                #[cfg(feature = "truncate_out_of_bounds")]
                {
                    needs_truncate && self.truncate_out_of_bounds as u8 != 0
                },
            );
            input_label_offset = offset;
            trimmed_input
        } else {
            self.input.clone()
        };

        let labels = self
            .labels
            .iter()
            .map(|label| {
                #[cfg(feature = "caret_color")]
                let label_caret_color: Option<RgbColor> = if self.caret_color {
                    label.caret_color.or_else(|| {
                        // If the label has no caret color, use the first color of the label if it exists
                        label.color.as_ref().and_then(|colors| {
                            if colors.is_empty() {
                                None
                            } else {
                                if let Ok(rbg_color) = RgbColor::try_from(colors[0]) {
                                    Some(rbg_color)
                                } else {
                                    None
                                }
                            }
                        })
                    })
                } else {
                    None
                };

                let stream = {
                    let mut stream = LineTokenStream::from_str_with_length(
                        &label.message,
                        self.max_label_length,
                    );
                    if let Some(color) = &label.color {
                        color.into_iter().for_each(|c| {
                            stream.on_color_all(*c);
                        });
                    }
                    stream
                };
                #[cfg(feature = "truncate_out_of_bounds")]
                // Avoids needing mut if the feature is not enabled
                let mut stream = stream;

                #[cfg(not(feature = "truncate_out_of_bounds"))]
                // Not need for mut here
                let range = label.range;
                #[cfg(feature = "truncate_out_of_bounds")]
                let mut range = label.range;
                #[cfg(feature = "truncate_out_of_bounds")]
                {
                    // Make sure to call "truncate_end" first, as it changes the range
                    // if its out of bounds
                    if range.truncate_end(*valid_range.end())
                        && self.truncate_out_of_bounds == TruncateMode::Indicate
                    {
                        use ::token::{Token, TokenStream};

                        stream.insert_line(
                            0,
                            TokenStream::from_iter(
                                [Token::Styled(
                                    AnsiStyle::BRIGHT_YELLOW,
                                    Some(Box::new(Token::Literal(
                                        "[ Label Range Truncated ]".into(),
                                    ))),
                                )]
                                .into_iter(),
                            ),
                        );
                    }
                }

                TokenizedLabelFull::new_from(
                    range,
                    stream,
                    label.child_labels.clone().into_iter().map(|cl| {
                        #[cfg(feature = "caret_color")]
                        let child_caret_color = if self.caret_color {
                            cl.caret_color.or_else(|| {
                                // If the label has no caret color, use the first color of the label if it exists
                                cl.color.as_ref().and_then(|colors| {
                                    if colors.is_empty() {
                                        None
                                    } else {
                                        if let Ok(rbg_color) = RgbColor::try_from(colors[0]) {
                                            Some(rbg_color)
                                        } else {
                                            None
                                        }
                                    }
                                })
                            })
                        } else {
                            None
                        };

                        TokenizedChildLabel::new_from(
                            {
                                let mut stream = LineTokenStream::from_str_with_length(
                                    &cl.message,
                                    self.max_child_label_length
                                        .unwrap_or(self.max_label_length - CHILD_LABEL_PADDING),
                                );
                                if let Some(color) = cl.color {
                                    color.into_iter().for_each(|c| {
                                        stream.on_color_all(c);
                                    });
                                }
                                stream
                            },
                            #[cfg(feature = "caret_color")]
                            child_caret_color,
                        )
                    }),
                    #[cfg(feature = "caret_color")]
                    label_caret_color,
                )
            })
            .collect::<Vec<_>>();
        Ok(Report::new(
            input,
            input_label_offset,
            self.display_range,
            labels,
            self.colored_input,
        ))
    }
}

#[test]
#[allow(unused)]
fn tokensrcmultilinestreamrs223a2ae17e7e4d239ec9c66cf1cdf40a() {
    let message = "Third Label";
    let stream = LineTokenStream::from_str_with_length(message, 30);
    dbg!(stream);
}

#[test]
#[allow(unused)]
fn srcbuilderrs466a325e875e424bbbc4474ff8735c3a() {
    let mut stream = ::token::TokenStream::new();
    let token = ::token::Token::Styled(
        AnsiStyle::RED,
        Some(Box::new(::token::Token::Literal("Hello".into()))),
    );
    stream.push_iter(token);
    let token = ::token::Token::Styled(AnsiStyle::RESET, None);
    stream.push_iter(token);
    dbg!(&stream);
    dbg!(format!("{:#}", stream));
}
