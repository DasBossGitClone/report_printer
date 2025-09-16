use super::*;
use crate::printer::LabelLine;

#[derive(Debug, Clone, derive_more::IntoIterator)]
pub struct FormattedCaretSegment {
    /// The main underbar line
    underbar: TokenStream,
    /// Separator line between caret lines including the V_CARETs
    underbar_sep: TokenStream,
    // (Main line, Separator line)
    #[into_iterator]
    caret_lines: Vec<CaretLine>,
}
impl FormattedCaretSegment {
    pub(super) fn new(
        underbar: TokenStream,
        underbar_sep: TokenStream,
        caret_lines: Vec<CaretLine>,
    ) -> Self {
        Self {
            underbar,
            underbar_sep,
            caret_lines,
        }
    }
    pub fn underbar(&self) -> [&TokenStream; 2] {
        [&self.underbar, &self.underbar_sep]
    }
    pub fn underbar_lines(&self) -> LineTokenStream {
        let mut line = LineTokenStream::new();
        line.push_new(&self.underbar);
        line.push_new(&self.underbar_sep);
        line
    }
    pub fn caret_lines(&self) -> &[CaretLine] {
        &self.caret_lines
    }
}

impl ArgumentErrorReport {
    pub(super) fn generate_underbar(&self) -> std::io::Result<ReportLabels> {
        let offset = self.input_label_offset;

        // (underbar_start, underbar_range, caret_positionals (relative to start, label_message, child_labels))
        let mut labels: Vec<(
            usize,
            RangeInclusive,
            Vec<(usize, usize, TokenStream, Vec<TokenizedChildLabel>)>,
        )> = Vec::with_capacity(self.labels.len());

        // Calculate the down caret (┬) positions and underbar ranges
        for label in self.labels.iter() {
            let TokenizedLabel {
                range,
                message,
                child_labels,
            } = label;

            // If a offset is set, the input is prepended by 3x. and a space
            let offset = offset.saturating_sub(4);
            let start = range.start().saturating_sub(offset);
            let end = range.end().saturating_sub(offset);

            let underbar_range: RangeInclusive = (start..=end).into();
            /* let underbar_len = end.saturating_sub(start + 1).max(1);

            // The underbar tree split (┬) usually starts at the 3rd character to the right of the start
            // Check if that is possible, otherwise just start in the middle
            let down_start = if start + 3 < end {
                2
            } else {
                (underbar_len / 2).saturating_sub(1)
            }; */

            /* let forward_label_indent = down_start + start;

            let child_labels_len = child_labels.len(); */

            /* let child_lines: Vec<TokenizedChildLabel> =
            Vec::from_iter(child_labels.iter().cloned()); */

            let underbar_range_len = underbar_range.end().saturating_sub(underbar_range.start());

            let label_line: (
                usize,
                RangeInclusive,
                Vec<(usize, usize, TokenStream, Vec<TokenizedChildLabel>)>,
            ) = (
                start,
                underbar_range,
                // Generate the underbar line positionals
                // It is important that this is generated first, as multiple labels can overlap and we cannot change after printing
                if underbar_range_len > 5 {
                    vec![(2, underbar_range_len, message.clone(), child_labels.clone())]
                } else if underbar_range_len > 3 {
                    vec![(
                        (underbar_range_len / 2).saturating_sub(1),
                        underbar_range_len,
                        message.clone(),
                        child_labels.clone(),
                    )]
                } else {
                    vec![(0, underbar_range_len, message.clone(), child_labels.clone())]
                },
            );

            labels.push(label_line);
        }

        // Merge overlapping ranges and their caret positions
        /*
        Vec<(
            usize [absolute start],
            RangeInclusive [underbar range],
            Vec<(
                usize [relative caret / underbar position],
                usize [length of the underbar],
                TokenStream [label message],
                Vec<TokenizedChildLabel> [child labels]
            )>,
        )>
        */

        let mut new_labels: Vec<(
            usize,
            RangeInclusive,
            Vec<(usize, usize, TokenStream, Vec<TokenizedChildLabel>)>,
        )> = Vec::with_capacity(labels.len());

        // Check for overlapping ranges
        for self_i in 0..labels.len() {
            for other in labels.iter().skip(self_i + 1).cloned() {
                let (other_start, other_range, other_label) = other;
                let (self_start, self_range, mut self_label) = labels[self_i].clone();
                if self_start == other_start && self_range == other_range {
                    // Exact same range, just merge caret positions
                    self_label.extend(other_label);
                    new_labels.push((self_start, self_range, self_label));
                    continue;
                }

                let self_start = self_start;
                let other_start = other_start;

                let self_len = self_range.end().saturating_sub(self_range.start());
                let other_len = other_range.end().saturating_sub(other_range.start());

                // If "other" fits entirely within the current range, we remove "other" and merge it into the current range, adding its caret positions
                if other_start >= self_start && (other_start + other_len) <= (self_start + self_len)
                {
                    // Merge caret positions
                    // Adjusting the positions to be relative to the current range
                    let offset = other_start.saturating_sub(self_start);
                    self_label.extend(other_label.into_iter().map(
                        |(pos, len, label_message, child_labels)| {
                            (pos + offset, len, label_message, child_labels)
                        },
                    ));
                    new_labels.push((self_start, self_range, self_label));
                }
                // If both overlap, we remove "other" and merge them into one, extending the current range to encompass both, and adding caret positions
                else if (other_start < (self_start + self_len))
                    && ((other_start + other_len) > self_start)
                {
                    // Merge caret positions
                    let offset = other_start.saturating_sub(self_start);
                    self_label.extend(other_label.into_iter().map(
                        |(pos, len, label_message, child_labels)| {
                            (pos + offset, len, label_message, child_labels)
                        },
                    ));

                    // Extend the current range to encompass both
                    let new_end = std::cmp::max(self_range.end(), other_range.end());
                    let new_range = (self_range.start()..=new_end).into();
                    new_labels.push((self_start, new_range, self_label));
                } else {
                    // No overlap, just add the current range as-is
                    new_labels.push((self_start, self_range, self_label));
                }
            }
        }
        dbg!(&new_labels);
        dbg!(&new_labels);

        /*         'outer: loop {
            let mut len = labels.len();
            'inner: for self_i in 0..len {
                let self_i = (len - self_i).saturating_sub(1);

                for other_i in (0..self_i).rev() {
                    if self_i == other_i {
                        continue;
                    }
                    let (other_start, other_range, other_label) = labels[other_i].clone();
                    let (self_start, self_range, self_label) = &mut labels[self_i];

                    let self_range_start = *self_start;
                    let self_len = self_range.end().saturating_sub(self_range.start());

                    let other_range_start = other_start;
                    let other_len = other_range.end().saturating_sub(other_range.start());

                    // If "other" fits entirely within the current range, we remove "other" and merge it into the current range, adding its caret positions
                    if other_range_start >= self_range_start
                        && (other_range_start + other_len) <= (self_range_start + self_len)
                    {
                        // Merge caret positions
                        // Adjusting the positions to be relative to the current range
                        let offset = other_range_start.saturating_sub(self_range_start);
                        self_label.extend(other_label.into_iter().map(
                            |(pos, len, label_message, child_labels)| {
                                (pos + offset, len, label_message, child_labels)
                            },
                        ));
                    }
                    // If both overlap, we remove "other" and merge them into one, extending the current range to encompass both, and adding caret positions
                    else if (other_range_start < (self_range_start + self_len))
                        && ((other_range_start + other_len) > self_range_start)
                    {
                        // Merge caret positions
                        let offset = other_range_start.saturating_sub(self_range_start);
                        self_label.extend(other_label.into_iter().map(
                            |(pos, len, label_message, child_labels)| {
                                (pos + offset, len, label_message, child_labels)
                            },
                        ));

                        // Extend the current range to encompass both
                        let new_end = std::cmp::max(self_range.end(), other_range.end());
                        *self_range = (self_range.start()..=new_end).into();
                    } else {
                        continue 'inner;
                    }
                    // Remove "other" as we have merged it
                    // We can safely do this as we are skipping this due to the continue
                    let removed = labels.remove(other_i);
                    dbg!(removed);
                    //len -= 1;
                    // We need to continue here, as the vector has changed and it could lead to out-of-bounds panics
                    continue 'outer;
                }
                if self_i == 0 {
                    break 'outer;
                }
            }
        }
        dbg!(&labels); */

        // Sort by starting position, then by range length (earliest start first then shortest range first)
        let labels: Vec<(
            usize,
            RangeInclusive,
            Vec<(usize, usize, TokenStream, Vec<TokenizedChildLabel>)>,
        )> = new_labels
            .into_iter()
            .sorted_by(|a, b| {
                a.0.cmp(&b.0)
                    .then(a.1.start().cmp(&b.1.start()))
                    .then(a.1.end().cmp(&b.1.end()))
            })
            .collect::<Vec<_>>();

        // Now we wanna transform each label from "labels" into a ReportSegment
        // aka the mapping is pretty much
        /*
        ReportCaret {
            start: usize [1],
            end: usize [2],
            positions: Vec<
                ReportLabel {
                    position: usize [3],
                    message: TokenStream [4],
                    child_labels: Vec<TokenStream> [5],
                },
            >,
        }

        Vec<(
            [1] usize,
            [2] RangeInclusive,
            Vec<(
                [3] usize,
                [4] TokenStream,
                [5] Vec<TokenizedChildLabel>
            )>,
        )>
        */

        Ok(labels
            .into_iter()
            .map(|(start, range, positions)| {
                let end = range.end();
                ReportCaret::new(
                    start,
                    end,
                    positions.into_iter().map(ReportLabel::from).collect(),
                )
            })
            .collect())
    }
}
