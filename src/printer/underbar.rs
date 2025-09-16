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
    pub(super) fn generate_underbar(&self) -> std::io::Result<ReportSegments> {
        let offset = self.input_label_offset;

        let mut colors = colors();

        // (underbar_start, underbar_range, caret_positionals (relative to start, label_message, child_labels))
        let mut labels: Vec<(
            usize,
            RangeInclusive,
            Vec<(usize, TokenStream, Vec<TokenizedChildLabel>)>,
        )> = Vec::with_capacity(self.labels.len());

        // Calculate the down caret (┬) positions and underbar ranges
        for label in self.labels.iter() {
            let TokenizedLabel {
                range,
                message,
                child_labels,
            } = label;

            let start = range.start().saturating_sub(offset);
            let end = range.end().saturating_sub(offset);

            let underbar_range: RangeInclusive = (start..=end).into();
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

            let mut child_lines: Vec<TokenizedChildLabel> =
                Vec::from_iter(child_labels.iter().cloned());

            let underbar_range_len = underbar_range.end().saturating_sub(underbar_range.start());

            let label_line: (
                usize,
                RangeInclusive,
                Vec<(usize, TokenStream, Vec<TokenizedChildLabel>)>,
            ) = (
                if offset != 0 { start + 4 } else { start },
                underbar_range,
                // Generate the underbar line positionals
                // It is important that this is generated first, as multiple labels can overlap and we cannot change after printing
                if underbar_range_len > 5 {
                    vec![(2, message.clone(), child_lines)]
                } else if underbar_range_len > 3 {
                    vec![(
                        (underbar_range_len / 2).saturating_sub(1),
                        message.clone(),
                        child_lines,
                    )]
                } else {
                    vec![(0, message.clone(), child_lines)]
                },
            );

            labels.push(label_line);
        }

        for label in labels {}
        /*
               let mut labels: Vec<(
                   usize,
                   RangeInclusive,
                   Vec<usize>,
                   TokenStream,
                   Vec<TokenizedChildLabel>,
               )> = labels
                   .into_iter()
                   .map(|label| {
                       let (underbar_start, underbar_range, label_message, child_labels) = label;

                       let range_len = underbar_range.end().saturating_sub(underbar_range.start());

                       if range_len > 5 {
                           // We start the down caret (┬) 3 characters to the right of the start
                           (
                               underbar_start,
                               underbar_range,
                               vec![2],
                               label_message,
                               child_labels,
                           )
                       } else if range_len > 3 {
                           // We start the down caret (┬) in the middle of the underbar
                           (
                               underbar_start,
                               underbar_range,
                               vec![(range_len / 2).saturating_sub(1)],
                               label_message,
                               child_labels,
                           )
                       } else {
                           // Just put it at the start
                           (
                               underbar_start,
                               underbar_range,
                               vec![0],
                               label_message,
                               child_labels,
                           )
                       }
                   })
                   .collect::<Vec<_>>();
        */
        // Check for overlapping ranges
        let mut len = labels.len();
        'inner: for i in 0..len {
            for j in (i + 1)..len {
                let (other_start, other_range, other_caret) = labels[j].clone();
                let (start, range, caret) = &mut labels[i];

                let range_start = *start;
                let self_len = range.end().saturating_sub(range.start());

                let other_range_start = other_start;
                let other_len = other_range.end().saturating_sub(other_range.start());

                // If "other" fits entirely within the current range, we remove "other" and merge it into the current range, adding its caret positions
                if other_range_start >= range_start
                    && (other_range_start + other_len) <= (range_start + self_len)
                {
                    // Merge caret positions
                    // Adjusting the positions to be relative to the current range
                    let offset = other_range_start.saturating_sub(range_start);
                    caret.extend(other_caret.into_iter().map(
                        |(pos, label_message, child_labels)| {
                            (pos + offset, label_message, child_labels)
                        },
                    ));
                }
                // If both overlap, we remove "other" and merge them into one, extending the current range to encompass both, and adding caret positions
                else if (other_range_start < (range_start + self_len))
                    && ((other_range_start + other_len) > range_start)
                {
                    // Merge caret positions
                    let offset = other_range_start.saturating_sub(range_start);
                    caret.extend(other_caret.into_iter().map(
                        |(pos, label_message, child_labels)| {
                            (pos + offset, label_message, child_labels)
                        },
                    ));

                    // Extend the current range to encompass both
                    let new_end = std::cmp::max(range.end(), other_range.end());
                    *range = (range.start()..=new_end).into();
                } else {
                    continue 'inner;
                }
                // Remove "other"
                let removed = labels.remove(j);
                dbg!(removed);
                len -= 1;
            }
        }

        let underbar_ranges: Vec<(usize, RangeInclusive, Vec<usize>)> = underbar_ranges
            .into_iter()
            .sorted_by(|a, b| a.0.cmp(&b.0))
            .collect::<Vec<_>>();

        let mut caret_segments = CaretsBuilder::new();

        // This cannot be possible, as we initially check for empty labels
        // but just to be sure
        if !underbar_ranges.is_empty() {
            let last = underbar_ranges.first().unwrap();

            let mut underbar = String::with_capacity(last.0 + last.1.end() - last.1.start() + 1);

            let initial_start = last.0;

            underbar.push_str(&SPACE(initial_start));

            let last_end = last.1.end();

            for (start, range, mut caret_pos) in underbar_ranges {
                let underbar_len = range.end().saturating_sub(range.start());

                let pad = if last_end < range.start() {
                    SPACE(start.saturating_sub(last_end))
                } else {
                    String::new()
                };

                underbar.push_str(pad.as_str());

                caret_pos.sort_by(|a, b| a.cmp(b));

                let mut caret_segment = CaretBuilder::new(start, start + underbar_len);
                for pos in caret_pos.iter() {
                    caret_segment.push(*pos);
                }
                caret_segments.push(caret_segment);
            }
        }
        Ok(caret_segments.finish())
    }
}
