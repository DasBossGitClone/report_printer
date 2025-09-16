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
        labels: impl Iterator<Item = &'a Label>,
    ) -> Self {
        let input = input.into();
        Self {
            input_label_offset: offset,
            input: TokenStream::from(&input),
            raw_input: input,
            labels: labels
                .cloned()
                .map(TokenizedLabel::from)
                // Sort labels by their start range
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

#[derive(Debug, Clone)]
pub struct TokenizedLabel {
    /// The range in the input string that this label annotates
    pub(super) range: RangeInclusive,
    pub(super) message: TokenStream,
    /// Optional child labels for more detailed annotations
    /// or if a message would repeat too much
    pub(super) child_labels: Vec<TokenizedChildLabel>,
}

impl From<Label> for TokenizedLabel {
    fn from(value: Label) -> Self {
        let mut stream = TokenStream::from(value.message);
        if let Some(color) = value.color {
            stream.on_color(color);
        }
        Self {
            range: value.range,
            message: stream,
            child_labels: value
                .child_labels
                .into_iter()
                .map(TokenizedChildLabel::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenizedChildLabel {
    pub(super) message: TokenStream,
}
impl From<ChildLabel> for TokenizedChildLabel {
    fn from(value: ChildLabel) -> Self {
        let mut stream = TokenStream::from(value.message);
        if let Some(color) = value.color {
            stream.on_color(color);
        }
        Self { message: stream }
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
