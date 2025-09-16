use ::std::str::FromStr;

use super::*;
/// The final report that can be printed to the user
/// Contained labels are printed each on their own
#[derive(Debug)]
pub struct ArgumentErrorReport {
    pub(super) input_label_offset: usize,
    pub(crate) raw_input: String,
    pub(super) input: TokenStream,
    pub(super) labels: Vec<TokenizedLabel>,
}
impl ArgumentErrorReport {
    pub fn new<'a, I: Into<String>>(
        input: I,
        offset: usize,
        labels: impl IntoIterator<Item = TokenizedLabel>,
    ) -> Self {
        let input = input.into();
        Self {
            input_label_offset: offset,
            input: TokenStream::from(&input),
            raw_input: input,
            labels: labels
                .into_iter() // Sort labels by their start range
                // with the earliest starting point first
                .sorted_by(|a, b| {
                    a.range
                        .start()
                        .cmp(&b.range.start())
                        .then(b.range.end().cmp(&a.range.end()))
                })
                .collect(),
        }
    }
}

impl ArgumentErrorReport {
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
pub struct TokenizedLabel {
    /// The range in the input string that this label annotates
    pub(super) range: RangeInclusive,
    /// If no colors is set, it will be generated at runtime
    pub(super) message: LineTokenStream,
    /// Optional child labels for more detailed annotations
    /// or if a message would repeat too much
    pub(super) child_labels: Vec<TokenizedChildLabel>,
}
impl TokenizedLabel {
    pub fn new<I: Display, R: IntoRange>(range: R, message: I) -> Self {
        Self {
            range: range.into_range(),
            message: LineTokenStream::from_str(&message.to_string())
                .expect("Failed to parse label message"),
            child_labels: Vec::new(),
        }
    }
    pub fn new_with<I: Display, R: IntoRange>(
        range: R,
        message: I,
        child_labels: impl IntoIterator<Item = TokenizedChildLabel>,
    ) -> Self {
        Self {
            range: range.into_range(),
            message: LineTokenStream::from_str(&message.to_string())
                .expect("Failed to parse label message"),
            child_labels: child_labels.into_iter().collect(),
        }
    }
    pub fn new_from<R: IntoRange, I: Into<LineTokenStream>>(
        range: R,
        message: I,
        child_labels: impl IntoIterator<Item = TokenizedChildLabel>,
    ) -> Self {
        Self {
            range: range.into_range(),
            message: message.into(),
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
    /* pub fn from_label(label: Label, max_label_width: usize, max_child_label_width: usize) -> Self {
        let message: LineTokenStream =
            LineTokenStream::from_str_with_length(&label.message, max_label_width);

        let child_labels: Vec<TokenizedChildLabel> = label
            .child_labels
            .into_iter()
            .map(|cl| {
                let message =
                    LineTokenStream::from_str_with_length(&cl.message, max_child_label_width);
                TokenizedChildLabel { message }
            })
            .collect();

        Self {
            range: label.range,
            message,
            child_labels,
        }
    } */
}

#[derive(Debug, Clone, derive_more::Into)]
pub struct TokenizedChildLabel {
    /// If no colors is set, it will be generated at runtime
    pub(super) message: LineTokenStream,
}

impl TokenizedChildLabel {
    pub fn new<I: Display>(message: I) -> Self {
        // We can safely unwrap here, as FromStr for LineTokenStream cannot fail
        let stream = LineTokenStream::from_str(&message.to_string())
            .expect("Failed to parse child label message");
        Self { message: stream }
    }
    pub fn new_from<I: Into<LineTokenStream>>(message: I) -> Self {
        Self {
            message: message.into(),
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
}
