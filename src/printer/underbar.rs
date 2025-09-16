use super::*;

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
        let mut new_labels: Vec<(
            usize,
            RangeInclusive,
            Vec<(usize, usize, TokenStream, Vec<TokenizedChildLabel>)>,
        )> = Vec::with_capacity(labels.len());

        if let Some(rem) = labels.into_iter().fold(
            None,
            |mut current: Option<(
                usize,
                RangeInclusive,
                Vec<(usize, usize, TokenStream, Vec<TokenizedChildLabel>)>,
            )>,
             other| {
                if let Some((current_start, current_range, mut current_labels)) = current.take() {
                    let (other_start, other_range, other_labels) = other;
                    // If "other" fits entirely within the current range, we remove "other" and merge it into the current range, adding its caret positions
                    if other_start >= current_start && (other_range.end() <= current_range.end()) {
                        // Merge caret positions
                        // Adjusting the positions to be relative to the current range
                        let offset = other_start.saturating_sub(current_start);
                        current_labels.extend(other_labels.into_iter().map(
                            |(pos, len, label_message, child_labels)| {
                                (pos + offset, len, label_message, child_labels)
                            },
                        ));
                        Some((current_start, current_range, current_labels))
                    }
                    // If both overlap, we remove "other" and merge them into one, extending the current range to encompass both, and adding caret positions
                    else if (other_start < (current_start + current_range.len()))
                        && (other_range.end() > current_start)
                    {
                        // Merge caret positions
                        let offset = other_start.saturating_sub(current_start);
                        current_labels.extend(other_labels.into_iter().map(
                            |(pos, len, label_message, child_labels)| {
                                (pos + offset, len, label_message, child_labels)
                            },
                        ));

                        // Extend the current range to encompass both
                        let new_end = std::cmp::max(current_range.end(), other_range.end());
                        let new_range = (current_range.start()..=new_end).into();
                        Some((current_start, new_range, current_labels))
                    } else {
                        // No overlap, just add the current range as-is and move to the next
                        new_labels.push((current_start, current_range, current_labels));
                        Some((other_start, other_range, other_labels))
                    }
                } else {
                    Some((other.0, other.1, other.2))
                }
            },
        ) {
            new_labels.push(rem);
        }

        // Sort by starting position, then by range length (earliest start first then shortest range first)
        new_labels.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.start().cmp(&b.1.start()))
                .then(a.1.end().cmp(&b.1.end()))
        });

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

        Ok(new_labels
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
