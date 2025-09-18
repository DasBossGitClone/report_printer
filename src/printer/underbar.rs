use ::token::saturating::SaturatingArithmetic;

use super::*;

impl Report {
    pub(super) fn generate_underbar(
        input_label_offset: usize,
        labels: impl IntoIterator<Item = TokenizedLabelFull>,
    ) -> ReportLabels {
        let offset = input_label_offset;

        // Calculate the down caret (┬) positions and underbar ranges
        let labels = labels.into_iter().map(|label| {
            let TokenizedLabelFull {
                range,
                message,
                child_labels,
                #[cfg(feature = "caret_color")]
                caret_color,
            } = label;

            // If a offset is set, the input is prepended by 3x. and a space
            let offset = offset.saturating_sub(4);
            let start = range.start().saturating_sub(offset);
            let end = range.end().saturating_sub(offset);

            let underbar_range: RangeInclusive = (start..=end).into();

            let underbar_range_len = underbar_range.len();

            let label_line: (
                // underbar_start
                usize,
                // underbar_range
                RangeInclusive,
                // caret_positionals
                Vec<(
                    // caret_position (relative to start)
                    usize,
                    // caret_length
                    usize,
                    // label_message
                    TokenizedLabel,
                    // child_labels
                    Vec<TokenizedChildLabel>,
                )>,
            ) = (
                start,
                underbar_range,
                // Generate the underbar line positionals
                // It is important that this is generated first, as multiple labels can overlap and we cannot change after printing
                vec![(
                    if underbar_range_len > 4 {
                        2
                    } else if underbar_range_len > 2 {
                        (underbar_range_len.sat_div(2)).sat_sub(1)
                    } else {
                        0
                    },
                    underbar_range_len,
                    TokenizedLabel::new_from(
                        message,
                        #[cfg(feature = "caret_color")]
                        caret_color,
                    ),
                    child_labels.clone(),
                )],
            );

            label_line
        });

        // Merge overlapping ranges and their caret positions
        let mut new_labels: Vec<(
            usize,
            RangeInclusive,
            Vec<(usize, usize, TokenizedLabel, Vec<TokenizedChildLabel>)>,
        )> = Vec::new();

        if let Some(rem) = labels.into_iter().fold(
            None,
            |mut current: Option<(
                usize,
                RangeInclusive,
                Vec<(usize, usize, TokenizedLabel, Vec<TokenizedChildLabel>)>,
            )>,
             other| {
                if let Some((current_start, current_range, mut current_labels)) = current.take() {
                    let (other_start, other_range, other_labels) = other;

                    #[cfg(not(feature = "merge_overlap"))]
                    {
                        // If both starting positions are the same, we split them into two separate carets
                        // as this improved readability tremendously
                        if other_start == current_start {
                            new_labels.push((current_start, current_range, current_labels));
                            return Some((other_start, other_range, other_labels));
                        }
                    }
                    // If "other" fits entirely within the current range, we remove "other" and merge it into the current range, adding its caret positions
                    if other_start >= current_start && (other_range.end() <= current_range.end()) {
                        // Merge caret positions
                        // Adjusting the positions to be relative to the current range
                        let offset = other_start.saturating_sub(current_start);
                        current_labels.extend(other_labels.into_iter().map(
                            |(pos, len, label_message, child_labels)| {
                                (pos.sat_add(offset), len, label_message, child_labels)
                            },
                        ));
                        Some((current_start, current_range, current_labels))
                    }
                    // If both overlap, we remove "other" and merge them into one, extending the current range to encompass both, and adding caret positions
                    else if (other_start < (current_start.sat_add(current_range.len())))
                        && (other_range.end() > current_start)
                    {
                        // Merge caret positions
                        let offset = other_start.saturating_sub(current_start);
                        current_labels.extend(other_labels.into_iter().map(
                            |(pos, len, label_message, child_labels)| {
                                (pos.sat_add(offset), len, label_message, child_labels)
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
        // the mapping is pretty much:
        /*
        Vec<ReportCaret {
            start: usize [1],
            end: usize [2],
            positions: Vec<
                ReportLabel {
                    position: usize [3],
                    message: TokenStream [4],
                    child_labels: Vec<TokenStream> [5],
                },
            >,
        }>

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

        new_labels
            .into_iter()
            .map(|(start, range, positions)| {
                let end = range.end();
                ReportCaret::new(
                    start,
                    end,
                    positions.into_iter().map(ReportLabel::from).collect(),
                )
            })
            .collect()
    }
}
