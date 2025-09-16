use ::std::fmt::Display;
use ::token::AnsiStyle;

use crate::ArgumentErrorReport;

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
        Some(self.start.cmp(&other.start).then(self.end.cmp(&other.end)))
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

/// A simple wrapper to ensure the string is ASCII only
pub struct AsciiString(pub String);
impl Display for AsciiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl AsciiString {
    pub fn new<S: Into<String>>(s: S) -> Result<Self, Vec<usize>> {
        let s: String = s.into();
        if s.chars().all(|c| c.is_ascii()) {
            Ok(Self(s))
        } else {
            // Return the indices of non-ascii characters
            Err(s
                .chars()
                .map(|c| c.is_ascii())
                .enumerate()
                .filter_map(|(i, is_ascii)| if !is_ascii { Some(i) } else { None })
                .collect::<Vec<usize>>())
        }
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
}

impl ArgumentErrorReporter {
    pub fn new<I: Into<String>>(input: I) -> Self {
        Self {
            trim_input: true,
            input: input.into(),
            labels: Vec::new(),
        }
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
    pub(super) message: String,
    /// Optional child labels for more detailed annotations
    /// or if a message would repeat too much
    pub(super) child_labels: Vec<ChildLabel>,
    /// If no colors is set, it will be generated at runtime
    pub(super) style: Option<AnsiStyle>,
}
impl Label {
    pub fn new<I: Display, R: IntoRange>(range: R, message: I) -> Self {
        Self {
            range: range.into_range(),
            message: message.to_string(),
            child_labels: Vec::new(),
            style: None,
        }
    }

    pub fn with_message<I: Display>(mut self, message: I) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn with_color<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.style = Some(style.into());
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
}

#[derive(Debug, Clone)]
pub struct ChildLabel {
    pub(super) message: String,
    /// If no colors is set, it will be generated at runtime
    pub(super) style: Option<AnsiStyle>,
}

impl ChildLabel {
    pub fn new<I: Display>(message: I) -> Self {
        Self {
            message: message.to_string(),
            style: None,
        }
    }

    pub fn with_color<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.style = Some(style.into());
        self
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

        Ok(ArgumentErrorReport::new(
            input,
            input_label_offset,
            self.labels.iter(),
        ))
    }
}
