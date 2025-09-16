use ::std::fmt::Display;
use ::token::{AnsiStyle, LineTokenStream};

use crate::{ArgumentErrorReport, TokenizedChildLabel, TokenizedLabel};

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
        self.end.saturating_sub(self.start).saturating_add(1)
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

#[derive(Debug, Clone)]
pub struct ArgumentErrorReporter {
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
}

impl ArgumentErrorReporter {
    pub fn new<I: Into<String>>(input: I) -> Self {
        Self {
            trim_input: true,
            display_range: false,
            input: input.into(),
            labels: Vec::new(),
            // Default max label length is 30 characters
            max_label_length: 30,
            max_child_label_length: None,
        }
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
    pub(super) color: Option<AnsiStyle>,
}
impl Label {
    pub fn new<I: Display, R: IntoRange>(range: R, message: I) -> Self {
        Self {
            range: range.into_range(),
            message: message.to_string(),
            child_labels: Vec::new(),
            color: None,
        }
    }

    /// Replaces the current message
    pub fn with_message<I: Display>(mut self, message: I) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn with_color<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.color = Some(style.into());
        self
    }

    pub fn with_child_label(mut self, child_label: ChildLabel) -> Self {
        self.child_labels.push(child_label);
        self
    }

    pub fn push<I: Into<ChildLabel>>(mut self, child_label: I) -> Self {
        self.child_labels.push(child_label.into());
        self
    }

    pub(crate) fn get_message(&self) -> String {
        if let Some(color) = self.color {
            // Wrap the message in color codes
            color.with_color(&self.message)
        } else {
            self.message.clone()
        }
    }
    pub(crate) fn into_message(self) -> String {
        if let Some(color) = self.color {
            // Wrap the message in color codes
            color.with_color(&self.message)
        } else {
            self.message
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChildLabel {
    /// If no colors is set, it will be generated at runtime
    pub(super) message: String,
    pub(super) color: Option<AnsiStyle>,
}

impl ChildLabel {
    pub fn new<I: Display>(message: I) -> Self {
        Self {
            message: message.to_string(),
            color: None,
        }
    }

    pub fn with_color<I: Into<AnsiStyle>>(self, style: I) -> Self {
        Self {
            color: Some(style.into()),
            ..self
        }
    }

    pub(crate) fn get_message(&self) -> String {
        if let Some(color) = self.color {
            // Wrap the message in color codes
            color.with_color(&self.message)
        } else {
            self.message.clone()
        }
    }
    pub(crate) fn into_message(self) -> String {
        if let Some(color) = self.color {
            // Wrap the message in color codes
            color.with_color(&self.message)
        } else {
            self.message
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("No labels were added to the report")]
    NoLabels,
    #[error("A label has an empty message")]
    LabelEmptyMessage,
    #[error("A child label has an empty message")]
    LabelChildEmptyMessage,
    #[error("The input string is empty")]
    EmptyInput,
    /// (given range, valid range)
    #[error("A label has a range that is out of bounds: {0:?} not contained within {1:?}")]
    OutOfBounds(RangeInclusive, RangeInclusive),
}

impl ArgumentErrorReporter {
    /// Validates the current state of the builder and generates an ArgumentErrorReport
    /// Returns a BuilderError if the state is invalid, but does not consume Self.
    /// This allows one to fix the issues and try again without reinstantiating
    /// the builder (so its a sorta "soft-fail").
    pub fn finish(&self) -> Result<ArgumentErrorReport, BuilderError> {
        // This verification allows us to carelessly use the ranges later on
        if self.labels.is_empty() {
            return Err(BuilderError::NoLabels);
        }
        if self.input.is_empty() {
            return Err(BuilderError::EmptyInput);
        }
        let valid_range = 0..=self.input.len();
        self.labels.iter().try_for_each(|label| {
            if label.range.start() < *valid_range.start() || label.range.end() > *valid_range.end()
            {
                return Err(BuilderError::OutOfBounds(
                    label.range.clone(),
                    valid_range.clone().into(),
                ));
            }
            if label.message.is_empty() {
                return Err(BuilderError::LabelEmptyMessage);
            }
            label.child_labels.iter().try_for_each(|child_label| {
                if child_label.message.is_empty() {
                    return Err(BuilderError::LabelChildEmptyMessage);
                }
                Ok(())
            })?;
            Ok(())
        })?;

        // Offset to apply to label ranges when trimming the input
        let mut input_label_offset = 0;

        let input = if self.trim_input {
            let (trimmed_input, offset) =
                ArgumentErrorReport::trim_input(&self.input, self.labels.iter());
            input_label_offset = offset;
            trimmed_input
        } else {
            self.input.clone()
        };

        let labels = self
            .labels
            .iter()
            .map(|label| {
                let t_label = TokenizedLabel::new_from(
                    label.range,
                    {
                        let mut stream = LineTokenStream::from_str_with_length(
                            &label.message,
                            self.max_label_length,
                        );
                        if let Some(color) = label.color {
                            stream.on_color_all(color);
                        }
                        stream
                    },
                    label.child_labels.clone().into_iter().map(|cl| {
                        TokenizedChildLabel::new_from({
                            let mut stream = LineTokenStream::from_str_with_length(
                                &cl.message,
                                self.max_child_label_length
                                    .unwrap_or(self.max_label_length - CHILD_LABEL_PADDING),
                            );
                            if let Some(color) = cl.color {
                                stream.on_color_all(color);
                            }
                            stream
                        })
                    }),
                );
                if let Some(color) = label.color {
                    t_label.with_color_all(color)
                } else {
                    t_label
                }
            })
            .collect::<Vec<_>>();

        Ok(ArgumentErrorReport::new(
            input,
            input_label_offset,
            self.display_range,
            labels,
        ))
    }
}
