use ::std::str::FromStr;

use super::*;
/// The final report that can be printed to the user
/// Contained labels are printed each on their own
#[derive(Debug)]
pub struct Report {
    /* pub(super) input_label_offset: usize,
    pub(crate) raw_input: String,
    pub(super) labels: Vec<TokenizedLabel>,
    */
    pub(crate) display_range: bool,
    pub(super) input: TokenStream,
    pub(super) report_labels: ReportLabels,
}
impl Report {
    pub fn new<'a, I: Into<String>>(
        input: I,
        offset: usize,
        display_range: bool,
        labels: impl IntoIterator<Item = TokenizedLabelFull>,
    ) -> Self {
        let input = input.into();
        Self {
            display_range,
            input: TokenStream::from(&input),
            report_labels: Self::generate_underbar(offset, labels),
        }
    }
}

impl Report {
    pub(crate) fn trim_input<'a, A: AsRef<str>>(
        input: A,
        labels: impl Iterator<Item = &'a Label>,
    ) -> (String, usize) {
        let input = input.as_ref();
        let input_len = input.len();
        // Raw start of the first range
        // Raw end of the last range
        let (min_start, max_end) = labels.fold((input_len, 0), |(min_start, max_end), label| {
            (
                min_start.min(label.range.start()),
                max_end.max(label.range.end()),
            )
        });

        // Add 1 word of context on each side if possible

        let min_start_padded = input[..min_start]
            .find_rev_iter(" ")
            .nth(1)
            .map(|pos| pos + 1)
            .unwrap_or(0);

        let max_end_padded = input[max_end..]
            .find_iter(" ")
            .nth(1)
            .map(|pos| max_end + pos)
            .unwrap_or(input_len);

        // Ensure we don't go out of bounds
        let trimmed_input = if max_end_padded < input_len {
            let pre = if min_start_padded > 0 {
                // As we "prepend" with 4 chars, we need to update the offset accordingly
                "... "
            } else {
                ""
            };
            let post = if max_end_padded < input_len {
                " ..."
            } else {
                ""
            };
            format!("{pre}{}{post}", &input[min_start_padded..max_end_padded])
        } else if max_end < input_len {
            let pre = if min_start_padded > 0 {
                // As we "prepend" with 4 chars, we need to update the offset accordingly
                "... "
            } else {
                ""
            };
            format!("{pre}{}", &input[min_start_padded..])
        } else {
            // Just in case. This case should not be possible, but better safe than sorry
            let pre = if min_start_padded > 0 {
                // As we "prepend" with 4 chars, we need to update the offset accordingly
                "... "
            } else {
                ""
            };
            format!("{pre}{}", &input[min_start_padded..])
        };
        (trimmed_input, min_start_padded)
    }
}

#[derive(Debug, Clone)]
pub struct TokenizedLabelFull {
    /// The range in the input string that this label annotates
    pub(super) range: RangeInclusive,
    /// If no colors is set, it will be generated at runtime
    pub(super) message: LineTokenStream,
    #[cfg(feature = "caret_color")]
    /// If the outer Option is populated, colored carets are enabled
    /// If the inner Option is None, the colors of the label message are used
    /// If the inner Option is Some(color), that color is used for the carets
    pub(super) caret_color: Option<RgbColor>,
    /// Optional child labels for more detailed annotations
    /// or if a message would repeat too much
    pub(super) child_labels: Vec<TokenizedChildLabel>,
}
impl TokenizedLabelFull {
    pub fn new<I: Display, R: IntoRange>(
        range: R,
        message: I,
        #[cfg(feature = "caret_color")] caret_color: Option<RgbColor>,
    ) -> Self {
        let stream =
            LineTokenStream::from_str(&message.to_string()).expect("Failed to parse label message");
        Self {
            #[cfg(feature = "caret_color")]
            caret_color,
            range: range.into_range(),
            message: stream,
            child_labels: Vec::new(),
        }
    }
    pub fn new_with<I: Display, R: IntoRange>(
        range: R,
        message: I,
        child_labels: impl IntoIterator<Item = TokenizedChildLabel>,
        #[cfg(feature = "caret_color")] caret_color: Option<RgbColor>,
    ) -> Self {
        let message =
            LineTokenStream::from_str(&message.to_string()).expect("Failed to parse label message");
        Self {
            #[cfg(feature = "caret_color")]
            caret_color,
            range: range.into_range(),
            message,
            child_labels: child_labels.into_iter().collect(),
        }
    }
    pub fn new_from<R: IntoRange, I: Into<LineTokenStream>>(
        range: R,
        message: I,
        child_labels: impl IntoIterator<Item = TokenizedChildLabel>,
        #[cfg(feature = "caret_color")] caret_color: Option<RgbColor>,
    ) -> Self {
        let message: LineTokenStream = message.into();
        Self {
            #[cfg(feature = "caret_color")]
            caret_color,
            range: range.into_range(),
            message,
            child_labels: child_labels.into_iter().collect(),
        }
    }

    /// Replaces the current message
    pub fn with_message<I: Display>(mut self, message: I) -> Self {
        self.message =
            LineTokenStream::from_str(&message.to_string()).expect("Failed to parse label message");
        self
    }

    pub fn with_color<I: Into<AnsiStyle>>(self, style: I) -> Self {
        self.with_color_all(style)
    }
    pub fn with_color_last<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.message.on_color_last(style);
        self
    }
    pub fn with_color_all<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.message.on_color_all(style);
        self
    }

    pub fn push<I: Into<TokenizedChildLabel>>(mut self, child_label: I) -> Self {
        self.child_labels.push(child_label.into());
        self
    }

    pub fn is_multi_line(&self) -> bool {
        self.message.is_multi_line()
    }
}

impl IntoIterator for TokenizedLabelFull {
    type Item = TokenStreamLine;
    type IntoIter = TokenStreamLineIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.message.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct TokenizedChildLabel {
    /// If no colors is set, it will be generated at runtime
    pub(super) message: LineTokenStream,
    #[cfg(feature = "caret_color")]
    /// If the outer Option is populated, colored carets are enabled
    /// If the inner Option is None, the colors of the label message are used
    /// If the inner Option is Some(color), that color is used for the carets
    pub(super) caret_color: Option<RgbColor>,
}

impl TokenizedChildLabel {
    pub fn new<I: Display>(
        message: I,
        #[cfg(feature = "caret_color")] caret_color: Option<RgbColor>,
    ) -> Self {
        // We can safely unwrap here, as FromStr for LineTokenStream cannot fail
        let stream = LineTokenStream::from_str(&message.to_string())
            .expect("Failed to parse child label message");
        Self {
            #[cfg(feature = "caret_color")]
            caret_color,
            message: stream,
        }
    }
    pub fn new_from<I: Into<LineTokenStream>>(
        message: I,
        #[cfg(feature = "caret_color")] caret_color: Option<RgbColor>,
    ) -> Self {
        let message: LineTokenStream = message.into();
        Self {
            #[cfg(feature = "caret_color")]
            caret_color,
            message,
        }
    }

    pub fn with_color<I: Into<AnsiStyle>>(self, style: I) -> Self {
        self.with_color_all(style)
    }

    pub fn with_color_all<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.message.on_color_all(style);
        self
    }
    pub fn with_color_last<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.message.on_color_last(style);
        self
    }
    pub fn to_token_stream<'a>(&'a self) -> &'a LineTokenStream {
        &self.message
    }
    pub fn into_token_stream(self) -> LineTokenStream {
        self.message
    }

    #[cfg(feature = "caret_color")]
    pub(crate) fn get_color(&self) -> Option<RgbColor> {
        self.caret_color.clone()
    }
}

impl Into<LineTokenStream> for TokenizedChildLabel {
    fn into(self) -> LineTokenStream {
        self.message
    }
}

impl IntoIterator for TokenizedChildLabel {
    type Item = TokenStreamLine;
    type IntoIter = TokenStreamLineIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.message.into_iter()
    }
}

#[derive(Debug, Clone)]
/// The minimal representation of a label thats used in the Report
///
/// It does not contain the range nor child labels, as they are handled separately by the Report
pub struct TokenizedLabel {
    pub(super) message: LineTokenStream,
    #[cfg(feature = "caret_color")]
    /// The optional color used for the carets
    #[warn(dead_code)]
    pub(super) caret_color: Option<RgbColor>,
}
impl TokenizedLabel {
    pub fn new<I: Display, #[cfg(feature = "caret_color")] C: Into<Option<RgbColor>>>(
        message: I,
        #[cfg(feature = "caret_color")] caret_color: C,
    ) -> Self {
        Self {
            #[cfg(feature = "caret_color")]
            caret_color: caret_color.into(),
            message: LineTokenStream::from_str(&message.to_string())
                .expect("Failed to parse label message"),
        }
    }
    pub fn new_from<
        I: Into<LineTokenStream>,
        #[cfg(feature = "caret_color")] C: Into<Option<RgbColor>>,
    >(
        message: I,
        #[cfg(feature = "caret_color")] caret_color: C,
    ) -> Self {
        Self {
            #[cfg(feature = "caret_color")]
            caret_color: caret_color.into(),
            message: message.into(),
        }
    }

    pub fn with_color<I: Into<AnsiStyle>>(self, style: I) -> Self {
        self.with_color_all(style)
    }
    pub fn with_color_last<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.message.on_color_last(style);
        self
    }
    pub fn with_color_all<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.message.on_color_all(style);
        self
    }

    pub fn is_multi_line(&self) -> bool {
        self.message.is_multi_line()
    }
    #[cfg(feature = "caret_color")]
    #[warn(dead_code)]
    pub(crate) fn get_color(&self) -> Option<RgbColor> {
        self.caret_color.clone()
    }
    #[cfg(feature = "caret_color")]
    #[warn(dead_code)]
    pub(crate) fn ref_color(&self) -> Option<&RgbColor> {
        self.caret_color.as_ref()
    }
}
impl Into<LineTokenStream> for TokenizedLabel {
    fn into(self) -> LineTokenStream {
        self.message
    }
}
impl IntoIterator for TokenizedLabel {
    type Item = TokenStreamLine;
    type IntoIter = TokenStreamLineIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.message.into_iter()
    }
}

pub trait TryWithStyling {
    #[allow(dead_code)]
    fn try_with_coloring<A: AsRef<RgbColor>>(self, style: Option<A>) -> Self;
    /// Proxy method that only applies coloring if the "caret_color" feature is enabled
    fn try_with_coloring_feature<A: AsRef<RgbColor>>(self, style: Option<A>) -> Self;
}

impl TryWithStyling for Token {
    fn try_with_coloring<A: AsRef<RgbColor>>(self, style: Option<A>) -> Self {
        if let Some(style) = style {
            Token::Styled(AnsiStyle::RgbColor(*style.as_ref()), Some(Box::new(self)))
        } else {
            self
        }
    }
    #[cfg(feature = "caret_color")]
    fn try_with_coloring_feature<A: AsRef<RgbColor>>(self, style: Option<A>) -> Self {
        Self::try_with_coloring(self, style)
    }
    #[cfg(not(feature = "caret_color"))]
    fn try_with_coloring_feature<A: AsRef<RgbColor>>(self, _: Option<A>) -> Self {
        self
    }
}
