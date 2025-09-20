use ::token::saturating::SaturatingArithmetic;

use super::*;

// We wanna be able to allow one to configure these at runtime
// thus we use static mut variables here
/// The amount of white-space padding to add directly after the arrow and before the label message
static mut ARROR_LABEL_PADDING: usize = 1;

/// The offset from the parent label to the child labels
static mut CHILD_LABEL_OFFSET: usize = 3;

pub fn set_arrow_label_padding(padding: usize) {
    unsafe {
        ARROR_LABEL_PADDING = padding;
    }
}
pub fn set_child_label_offset(offset: usize) {
    unsafe {
        CHILD_LABEL_OFFSET = offset;
    }
}

crate::impl_field!(
    ReportCaret,start,usize;
    ReportCaret,end,usize;
    ReportCaret,rev_positions,Vec<ReportLabel>;
    ReportLabel,position,usize;
    ReportLabel,message,TokenizedLabel;
    ReportLabel,child_labels,Vec<TokenizedChildLabel>;
);

#[derive(Debug, Clone)]
pub(self) struct Lines {
    lines: Vec<Line>,
}
impl Lines {
    #[allow(dead_code)]
    pub fn to_inner<'a>(&'a self) -> impl Iterator<Item = TokenStream> {
        self.lines.iter().map(|l| l.clone().into_inner())
    }
    #[allow(dead_code)]
    pub fn into_inner(self) -> impl Iterator<Item = TokenStream> {
        self.lines.into_iter().map(|l| l.into_inner())
    }
    pub fn len(&self) -> usize {
        self.lines.len()
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Line> {
        self.lines.iter()
    }
    pub fn push<I: Into<Line>>(&mut self, line: I) -> &mut Self {
        self.lines.push(line.into());
        self
    }
    #[allow(dead_code)]
    pub fn extend<I: Into<Line>>(&mut self, lines: impl IntoIterator<Item = I>) -> &mut Self {
        self.lines.extend(lines.into_iter().map(Into::into));
        self
    }
    #[allow(dead_code)]
    pub fn extend_clone<I: AsRef<Line>>(
        &mut self,
        lines: impl IntoIterator<Item = I>,
    ) -> &mut Self {
        self.lines
            .extend(lines.into_iter().map(|l| l.as_ref().clone()));
        self
    }
    pub fn new() -> Self {
        Self { lines: vec![] }
    }
    #[allow(dead_code)]
    pub fn last(&self) -> Option<&Line> {
        self.lines.last()
    }
    pub fn last_mut(&mut self) -> Option<&mut Line> {
        self.lines.last_mut()
    }
}
impl IntoIterator for Lines {
    type Item = Line;
    type IntoIter = std::vec::IntoIter<Line>;
    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}
impl FromIterator<Line> for Lines {
    fn from_iter<T: IntoIterator<Item = Line>>(iter: T) -> Self {
        let lines: Vec<Line> = iter.into_iter().collect();
        Self { lines }
    }
}

impl Display for Lines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = self.len();
        if total == 0 {
            return Ok(());
        }
        let mut iter = self.iter().rev();
        let last = iter.next().expect("No lines");
        if total == 1 {
            write!(f, "{last}")
        } else {
            let mut skipped = iter.rev().take(total.sat_sub(1));
            skipped.try_for_each(|line| write!(f, "{line:#}"))?;
            write!(f, "{last}")
        }
    }
}

#[derive(Debug, Clone)]
/// A single line in the caret report
///
/// This is just a wrapper around a TokenStream
/// to differentiate between the different types of lines
/// but there is not logical difference between them (just easier to read)
pub(self) enum Line {
    /// Separator line with carets only
    Sep(TokenStream),
    /// The underbar that annotates the positions of the labels on the reference input
    Underbar(TokenStream),
    /// Label with the carets
    Label(TokenStream),
    /// Mutliline Label with the carets
    ///
    /// The first line is Self::Label and the rest are
    /// Self::LabelSeq (Label Sequence)
    LabelSeq(TokenStream),
}
impl Line {
    pub fn into_inner(self) -> TokenStream {
        match self {
            Line::Sep(line) | Line::Underbar(line) | Line::LabelSeq(line) | Line::Label(line) => {
                line
            }
        }
    }
    pub fn push<I: Into<Token>>(&mut self, token: I) -> &mut Self {
        match self {
            Line::Sep(line) | Line::Underbar(line) | Line::LabelSeq(line) | Line::Label(line) => {
                line.push_iter(token.into());
            }
        }
        self
    }
    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<Token> {
        match self {
            Line::Sep(line) | Line::Underbar(line) | Line::LabelSeq(line) | Line::Label(line) => {
                line.pop()
            }
        }
    }
}
impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // All lines share the same implementation of Display
        match self {
            Line::Sep(line) | Line::Underbar(line) | Line::LabelSeq(line) | Line::Label(line) => {
                write!(f, "{}{}", line, f.alternate().then(|| "\n").unwrap_or(""))
            }
        }
    }
}

#[derive(Debug, Clone, derive_more::IntoIterator)]
pub struct ReportCaret {
    /// Start of underbar
    start: usize,
    /// End of underbar
    end: usize,
    /// Relative to start
    /// Reversed, so that popping gets the leftmost position first
    #[into_iterator]
    rev_positions: Vec<ReportLabel>,
}
impl PartialEq for ReportCaret {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl Eq for ReportCaret {}
impl PartialOrd for ReportCaret {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.start.cmp(&other.start).then(self.end.cmp(&other.end)))
    }
}
impl Ord for ReportCaret {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start
            .cmp(&other.start)
            .then(self.end.cmp(&other.end))
            .then(self.rev_positions.len().cmp(&other.rev_positions.len()))
            .then_with(|| {
                self.rev_positions
                    .iter()
                    .rev()
                    .zip(other.rev_positions.iter().rev())
                    .find_map(|(a, b)| {
                        let ord = a.cmp(b);
                        if ord == std::cmp::Ordering::Equal {
                            None
                        } else {
                            Some(ord)
                        }
                    })
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

impl ReportCaret {
    pub(super) fn new(start: usize, end: usize, mut rev_positions: Vec<ReportLabel>) -> Self {
        rev_positions.sort_by(|a, b| a.cmp(b));
        rev_positions.dedup();
        rev_positions.reverse();
        Self {
            start,
            end,
            rev_positions,
        }
    }
    pub fn range(&self) -> RangeInclusive {
        (self.start..=self.end).into()
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &ReportLabel> {
        self.rev_positions.iter()
    }
    /// Adds a label to the caret report at the correct position
    ///
    /// If the label is out of bounds, the range is extended to include it
    ///
    /// This method does sort and dedup the labels after adding
    /// to ensure that there are no duplicate labels and that they are in the correct order
    pub fn push_sorted<I: Into<ReportLabel>>(&mut self, label: I) {
        let label: ReportLabel = label.into();
        // Validate that the label is in bounds
        if label.position() > self.end.saturating_sub(self.start) {
            // Its not in bounds, so we need to extend the range
            self.start = self.start.min(label.position());
            self.end = self.end.max(
                label
                    .position()
                    .saturating_add(label.length.saturating_sub(1)),
            );
            self.rev_positions.push(label.into());
        } else {
            self.rev_positions.push(label.into());
        }
        self.rev_positions.sort_by(|a, b| a.cmp(b));
        self.rev_positions.dedup();
        self.rev_positions.reverse();
    }
    /// Adds a label to the caret report at the correct position
    ///
    /// If the label is out of bounds, the range is extended to include it
    ///
    /// This method does NOT sort and dedup the labels after adding
    /// thus it is the callers responsibility to ensure that there are no duplicate labels
    /// and that they are in the correct order
    ///
    /// Its not encouraged to use this method
    pub fn push<I: Into<ReportLabel>>(&mut self, label: I) {
        let label: ReportLabel = label.into();
        // Validate that the label is in bounds
        if label.position() > self.end.saturating_sub(self.start) {
            // Its not in bounds, so we need to extend the range
            self.start = self.start.min(label.position());
            self.end = self.end.max(
                label
                    .position()
                    .saturating_add(label.length.saturating_sub(1)),
            );
            self.rev_positions.push(label.into());
        } else {
            self.rev_positions.push(label.into());
        }
    }
    pub fn pop(&mut self) -> Option<ReportLabel> {
        self.rev_positions.pop()
    }
    pub fn is_empty(&self) -> bool {
        self.rev_positions.is_empty()
    }
    pub fn len(&self) -> usize {
        self.rev_positions.len()
    }
    pub(super) fn get_underbar_ranges(&self) -> Vec<(usize, usize, Option<RgbColor>)> {
        if self.is_empty() {
            return vec![];
        }

        let mut ranges = Vec::with_capacity(self.len());

        let mut last_color: Option<RgbColor> = None;

        #[cfg(feature = "merge_overlap")]
        let mut last_pos = 0;

        let mut index = self.end;

        for label in self.iter() {
            #[cfg(feature = "caret_color")]
            {
                last_color = label.message.ref_color().cloned();
            }
            #[cfg(not(feature = "caret_color"))]
            {
                last_color = label
                    .message
                    .message
                    .iter()
                    .find_map(|line| {
                        line.iter().find_map(|tkn| {
                            if let Token::Styled(color, _) = tkn {
                                match color {
                                    AnsiStyle::RgbColor(rgb) => Some(*rgb),
                                    AnsiStyle::Color(color) => Some(RgbColor::from(*color)),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        })
                    })
                    .or(last_color);
            };
            // We dont wanna use "position" from the ReportLabel here, as that inducates the caret position
            // but we wanna cover the whole label
            let len = label.length;
            if self.end.saturating_sub(len) < index {
                let sep = index.saturating_sub(self.end.saturating_sub(len));
                if sep > 0 && last_color.is_some() {
                    ranges.push((self.end.saturating_sub(len), sep, last_color.clone()));
                }
                index = self.end.saturating_sub(len);
            } else {
                // We push until we hit the start of the underbar
                break;
            }
        }
        if index > self.start {
            let sep = index.saturating_sub(self.start);
            if sep > 0 && last_color.is_some() {
                ranges.push((self.start, sep, last_color.clone()));
            }
        }

        ranges
    }

    pub(self) fn format(mut self) -> Option<Lines> {
        // We wanna deref them once, so we dont need to do unsafe blocks everywhere else
        #[allow(non_snake_case)]
        let ARROR_LABEL_PADDING_REF = unsafe { ARROR_LABEL_PADDING };
        #[allow(non_snake_case)]
        let CHILD_LABEL_OFFSET_DEREF = unsafe { CHILD_LABEL_OFFSET };

        if self.is_empty() {
            return None;
        }
        let mut lines = Lines::new();

        let mut underbar = TokenStream::new();
        let mut underbar_sep = TokenStream::new();

        underbar.push_iter(Token::Space(self.start));

        let mut last_index = 0;

        let mut last_color: Option<&RgbColor> = None;

        #[cfg(feature = "merge_overlap")]
        let mut last_pos = 0;

        for (i, label) in self.iter().rev().enumerate() {
            let pos = label.position();

            #[cfg(feature = "merge_overlap")]
            {
                if pos == last_pos {
                    continue;
                }
                last_pos = pos;
            }

            let sep = pos.saturating_sub(last_index);

            #[cfg(feature = "caret_color")]
            {
                last_color = label.message.ref_color()
            };
            #[cfg(not(feature = "caret_color"))]
            {
                last_color = None
            };
            underbar.push_iter(Token::HCaret(sep).try_with_coloring_feature(last_color));
            underbar.push_iter(Token::HDown.try_with_coloring_feature(last_color));
            last_index = pos.sat_add(1);

            if let Some(next) = self.iter().rev().nth(i.sat_add(1)) {
                let next_pos = next.position();

                // Cover the underbar until the "next" label

                // We add 2, as we need to cover the HDown and the position of the next label
                let next_sep = next_pos.saturating_sub(last_index.sat_add(2));
                underbar.push_iter(Token::HCaret(next_sep).try_with_coloring_feature(last_color));
                last_index += next_sep;
            }
        }
        if last_index < self.end.saturating_sub(self.start) {
            underbar.push_iter(
                Token::HCaret(
                    self.end
                        .saturating_sub(self.start)
                        .saturating_sub(last_index),
                )
                .try_with_coloring_feature(last_color),
            );
        }

        lines.push(Line::Underbar(underbar));

        underbar_sep.push_iter(Token::Space(self.start));

        let mut last_index = 0;

        #[cfg(feature = "merge_overlap")]
        let mut last_pos = 0;

        for label in self.iter().rev() {
            let pos = label.position();

            #[cfg(feature = "merge_overlap")]
            {
                if pos == last_pos {
                    continue;
                }
                last_pos = pos;
            }

            let sep = pos.saturating_sub(last_index);

            #[cfg(feature = "caret_color")]
            let label_caret_color = label.message.ref_color();
            #[cfg(not(feature = "caret_color"))]
            let label_caret_color: Option<&RgbColor> = None;

            underbar_sep.push_iter(Token::Space(sep));
            underbar_sep.push_iter(Token::VCaret.try_with_coloring_feature(label_caret_color));
            last_index = pos.sat_add(1);
        }
        if last_index < self.end.saturating_sub(self.start) {
            underbar_sep.push_iter(Token::Space(
                self.end
                    .saturating_sub(self.start)
                    .saturating_sub(last_index),
            ));
        }

        lines.push(Line::Sep(underbar_sep));

        // Generate each caret line
        // each line "arrows" the first position to the end
        // followed by a "sepeartor" line containing all V_CARETs at the given positions
        while !self.is_empty() {
            // First generate the main line
            let mut label_line = TokenStream::new();
            label_line.push_iter(Token::Space(self.start));

            let mut current_pos = 0;

            // We wanna use the color of the first label here, as this is also used for the Arrows
            #[cfg(feature = "caret_color")]
            let label_color: Option<&RgbColor> = self
                .iter()
                .rev()
                .next()
                .expect("No labels")
                .ref_label_color();
            #[cfg(not(feature = "caret_color"))]
            let label_color: Option<&RgbColor> = None;

            for (i, label) in self.iter().rev().enumerate() {
                let pos = label.position();

                // Insert spaces until we reach the next position, if its the first position, else we draw H_CARET
                if i == 0 {
                    label_line.push_iter(Token::Space(pos.saturating_sub(current_pos)));
                } else {
                    label_line.push_iter(
                        Token::HCaret(pos.saturating_sub(current_pos))
                            .try_with_coloring_feature(label_color),
                    );
                }

                current_pos = pos.sat_add(1);

                if i == 0 {
                    // Transition from H_CARET to UP_RIGHT
                    #[cfg(feature = "merge_overlap")]
                    {
                        // No we need to check that at this position, there are no other labels
                        let transition = if self.iter().any(|l| l.position() == pos && l != label) {
                            Token::VRight.try_with_coloring_feature(label_color)
                        } else {
                            Token::UpRight.try_with_coloring_feature(label_color)
                        };
                        label_line.push([
                            transition,
                            Token::HCaret(1).try_with_coloring_feature(label_color),
                        ]);
                    }

                    #[cfg(not(feature = "merge_overlap"))]
                    {
                        label_line.push(Token::UpRight.try_with_coloring_feature(label_color));
                        label_line.push(Token::HCaret(1).try_with_coloring_feature(label_color));
                    }
                } else {
                    #[cfg(feature = "merge_overlap")]
                    {
                        // No we need to check that at this position, there are no other labels
                        if !self.iter().any(|l| l.position() == pos && l != label) {
                            label_line
                                .push(Token::HCaret(1).try_with_coloring_feature(label_color));
                        };
                    }

                    #[cfg(not(feature = "merge_overlap"))]
                    {
                        label_line.push(Token::HCaret(1).try_with_coloring_feature(label_color));
                    }
                }
            }
            // pop the position, so we dont print it again in the separator line
            let last = self.rev_positions.pop().unwrap();

            // Write the label
            let ReportLabel {
                message,
                child_labels,
                position: parent_label_position,
                ..
            } = last;
            if child_labels.is_empty() {
                // We can print a short arrow
                if message.is_multi_line() {
                    let mut message = message.into_iter();

                    let first = message.next().unwrap();
                    let is_only = first.is_only();
                    let mut line = label_line.clone();
                    line.push_iter([
                        if is_only {
                            Token::LArrow
                        } else {
                            Token::VCaret
                        },
                        Token::Space(ARROR_LABEL_PADDING_REF),
                    ]);
                    line.extend(first);
                    lines.push(Line::Label(line));

                    if let Some(Token::Styled(_, label_line_transition)) = label_line.r_get_mut(2) {
                        *label_line_transition = Some(Box::new(Token::Space(1)));
                    } else {
                        let _ = label_line.pop();
                        let _ = label_line.pop();
                        label_line.push(Token::Space(2));
                    }
                    // We want a space for multiline labels, to indicate that there are more lines
                    label_line.push_iter([
                        Token::VCaret,
                        Token::Space(ARROR_LABEL_PADDING_REF.sat_add(1)),
                    ]);

                    for line in message {
                        let mut label_line = (&label_line).clone();
                        label_line.extend(line);
                        lines.push(Line::Label(label_line));
                    }
                } else {
                    for line in message.into_iter() {
                        let mut label_line = (&label_line).clone();
                        label_line.push_iter([
                            if line.is_only() {
                                Token::LArrow
                            } else {
                                Token::VCaret
                            },
                            Token::Space(ARROR_LABEL_PADDING_REF),
                        ]);
                        label_line.extend(line);
                        lines.push(Line::Label(label_line));
                    }
                }
            } else {
                #[cfg(feature = "caret_color")]
                let label_caret_color: Option<RgbColor> = message.get_color();
                // pcc = parent_colored_caret
                // Shorthand to apply coloring if enabled
                // Its just there to reduce boilerplate
                let pcc = |token: Token| -> Token {
                    #[cfg(feature = "caret_color")]
                    {
                        token.try_with_coloring_feature(label_caret_color.as_ref())
                    }
                    #[cfg(not(feature = "caret_color"))]
                    {
                        token
                    }
                };

                if message.is_multi_line() {
                    // We need a separator line, for the following lines, as it contains the carets
                    let mut sep = self
                        .get_separator_line()
                        .unwrap_or(Line::Sep(TokenStream::new()))
                        .into_inner();

                    // Add spaces until we reach the caret of the parent label +  2 (for the arrow-transition)
                    let current_pos = (sep).lit_len();
                    if current_pos == 0 {
                        let target_pos = parent_label_position
                            // 2 for the arrow-transition
                            .saturating_add(3)
                            .saturating_add(self.start);
                        if target_pos > current_pos {
                            sep.push_iter(Token::Space(target_pos.saturating_sub(current_pos)));
                        }
                    } else {
                        let target_pos = current_pos
                            // 2 for the arrow-transition
                            .saturating_add(2);
                        if target_pos > current_pos {
                            sep.push_iter(Token::Space(target_pos.saturating_sub(current_pos)));
                        }
                    }

                    let mut message = message.into_iter();
                    if let Some(line) = message.next() {
                        label_line.push_iter([
                            pcc(Token::HCaret(1)),
                            pcc(Token::HDown),
                            pcc(Token::HCaret(2)),
                            pcc(Token::VLeft),
                            Token::Space(ARROR_LABEL_PADDING_REF),
                        ]);
                        label_line.extend(line);
                        lines.push(Line::Label(label_line));

                        for line in message {
                            let mut label_line = sep.clone();
                            label_line.push_iter([
                                pcc(Token::VCaret),
                                Token::Space(2),
                                pcc(Token::VCaret),
                                // offset by 1 to indicate that the line was split
                                Token::Space(ARROR_LABEL_PADDING_REF.sat_add(1)),
                            ]);
                            label_line.extend(line);
                            lines.push(Line::LabelSeq(label_line));
                        }
                    } else {
                        panic!("Message is multi-line, but has no lines");
                    }
                } else {
                    label_line.push_iter([
                        pcc(Token::HCaret(1)),
                        pcc(Token::HDown),
                        pcc(Token::HCaret(2)),
                        pcc(Token::LArrow),
                        Token::Space(ARROR_LABEL_PADDING_REF),
                    ]);

                    label_line.extend(message.into_iter().next().unwrap());

                    lines.push(Line::Label(label_line));
                }

                // Now we wanna print the child labels
                // The returned separator line reaches until the last caret of the last label
                // so we need to further extend it to the current parent label + CHILD_LABEL_PADDING (4) (for the arrow-transition)
                let mut child_sep = self
                    .get_separator_line()
                    .unwrap_or(Line::Sep(TokenStream::new()))
                    .into_inner();

                // Add spaces until we reach the caret of the parent label + CHILD_LABEL_PADDING (4) (for the arrow-transition)
                let current_pos = (child_sep).lit_len();
                if current_pos == 0 {
                    let target_pos = parent_label_position
                        // 2 for the arrow-transition
                        .saturating_add(3)
                        .saturating_add(self.start);
                    if target_pos > current_pos {
                        child_sep.push_iter(Token::Space(target_pos.saturating_sub(current_pos)));
                    }
                } else {
                    let target_pos = current_pos
                        // 2 for the arrow-transition
                        .saturating_add(2);
                    if target_pos > current_pos {
                        child_sep.push_iter(Token::Space(target_pos.saturating_sub(current_pos)));
                    }
                }
                // We wanna clone it here, as this is the separator for all child labels
                let mut child_sep_with_caret = child_sep.clone();
                child_sep_with_caret.push_iter(pcc(Token::VCaret));
                lines.push(Line::Sep(child_sep_with_caret.clone()));
                let child_labels_len = child_labels.len();
                for (i, child) in child_labels.into_iter().enumerate() {
                    // We can "unsafely" sub here, as the for loop ensures that child_labels_len > 0
                    let is_last_child_label = i == child_labels_len.sat_sub(1);

                    #[cfg(feature = "caret_color")]
                    let child_label_caret_color: Option<RgbColor> =
                        child.get_color().or(label_caret_color.clone());
                    // ccc = child_colored_caret
                    // Shorthand to apply coloring if enabled
                    // Its just there to reduce boilerplate
                    let ccc = |token: Token| -> Token {
                        #[cfg(feature = "caret_color")]
                        {
                            token.try_with_coloring_feature(child_label_caret_color.as_ref())
                        }
                        #[cfg(not(feature = "caret_color"))]
                        {
                            token
                        }
                    };

                    // Each child label is prepended by the same "child_sep" as that resembles the carets of the other labels
                    for child_label_line in child.into_iter() {
                        let mut child_line = child_sep.clone();
                        // CHILD_LABEL_OFFSET +  2 as we need to offset the VCaret by 2, otherwise they would directly be next to the caret for the follwing labels
                        match (
                            child_label_line.is_first(),
                            is_last_child_label,
                            child_label_line.is_only(),
                        ) {
                            // Only line in the last child label
                            (true, true, true) => {
                                child_line.push_iter([
                                    ccc(Token::UpRight),
                                    ccc(Token::HCaret(CHILD_LABEL_OFFSET_DEREF.sat_add(2))),
                                    ccc(Token::LArrow),
                                    Token::Space(ARROR_LABEL_PADDING_REF),
                                ]);
                            }
                            // Only line in label-child, but not the last child label
                            (true, false, true) => {
                                child_line.push_iter([
                                    pcc(Token::VRight),
                                    ccc(Token::HCaret(CHILD_LABEL_OFFSET_DEREF.sat_add(2))),
                                    ccc(Token::LArrow),
                                    Token::Space(ARROR_LABEL_PADDING_REF),
                                ]);
                            }
                            // First line but not last in label-child, not last child label
                            (true, true, false) => {
                                child_line.push_iter([
                                    pcc(Token::UpRight),
                                    ccc(Token::HCaret(CHILD_LABEL_OFFSET_DEREF.sat_add(2))),
                                    ccc(Token::VLeft),
                                    Token::Space(ARROR_LABEL_PADDING_REF),
                                ]);
                            }
                            // First line but not last in label-child, last child label
                            (true, false, false) => {
                                child_line.push_iter([
                                    pcc(Token::VRight),
                                    ccc(Token::HCaret(CHILD_LABEL_OFFSET_DEREF.sat_add(2))),
                                    ccc(Token::VLeft),
                                    Token::Space(ARROR_LABEL_PADDING_REF),
                                ]);
                            }
                            // not last Child label, but not the only line
                            (_, false, false) => {
                                child_line.push_iter([
                                    // We do not wanna style the first caret, as it is the continuation of the parent label
                                    Token::VCaret,
                                    Token::Space(CHILD_LABEL_OFFSET_DEREF.sat_add(2)),
                                    ccc(Token::VCaret),
                                    // offset by 1 to indicate that the line was split
                                    Token::Space(ARROR_LABEL_PADDING_REF.sat_add(1)),
                                ]);
                            }
                            // Last Child label, but not the only line
                            (_, true, false) => {
                                child_line.push_iter([
                                    // We need to add an extra space here, as there are not more child labels, thus no carets which would offset the line
                                    Token::Space(CHILD_LABEL_OFFSET_DEREF.sat_add(3)),
                                    ccc(Token::VCaret),
                                    // offset by 1 to indicate that the line was split
                                    Token::Space(ARROR_LABEL_PADDING_REF.sat_add(1)),
                                ]);
                            }
                            (_, _, true) => {
                                child_line.push_iter([Token::VCaret, Token::Space(5)]);
                                eprintln!(
                                    "{RED} This should never happen, how the fuck could this even be possible?! {RESET}"
                                );
                            }
                        }

                        if is_last_child_label {
                        } else {
                        }
                        child_line.extend(child_label_line);
                        // Add the child line
                        lines.push(Line::Label(child_line));
                    }

                    if !is_last_child_label {
                        // We wanna clone it here, as this is the separator for all child labels
                        lines.push(Line::Sep(child_sep_with_caret.clone()));
                    }
                }
            }
            // If there are more labels, we wanna add a separator line
            if let Some(sep) = self.get_separator_line() {
                lines.push(sep);
            }
        }
        if let Some(last) = lines.last_mut() {
            last.push(Token::Reset);
        }
        Some(lines)
    }
    fn get_separator_line(&self) -> Option<Line> {
        if self.is_empty() {
            return None;
        }
        let mut sep: TokenStream = TokenStream::new();
        sep.push_iter(Token::Space(self.start));
        let mut current_pos = 0;

        #[cfg(feature = "merge_overlap")]
        let mut last_pos = 0;

        for label in self.iter().rev() {
            let pos = label.position();

            #[cfg(feature = "merge_overlap")]
            {
                if pos == last_pos {
                    continue;
                }
                last_pos = pos;
            }

            // Insert spaces until we reach the next position
            sep.push_iter(Token::Space(pos.saturating_sub(current_pos)));
            current_pos = pos.sat_add(1);
            #[cfg(feature = "caret_color")]
            sep.push(Token::VCaret.try_with_coloring_feature(label.ref_label_color()));

            #[cfg(not(feature = "caret_color"))]
            sep.push_iter(Token::VCaret);
        }
        Some(Line::Sep(sep))
    }
}
impl Display for ReportCaret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(formatted) = self.clone().format() {
            write!(f, "{formatted:#}")
        } else {
            std::fmt::Result::Err(std::fmt::Error)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReportLabel {
    pub position: usize,
    pub length: usize,
    pub message: TokenizedLabel,
    pub child_labels: Vec<TokenizedChildLabel>,
}

impl PartialEq for ReportLabel {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.length == other.length
            && self.message == other.message
            && self.child_labels == other.child_labels
    }
}
impl Eq for ReportLabel {}
impl PartialOrd for ReportLabel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.position.cmp(&other.position))
    }
}
impl Ord for ReportLabel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.cmp(&other.position)
    }
}
impl ReportLabel {
    pub fn new<I: Into<TokenizedLabel>, C: Into<TokenizedChildLabel>, V: IntoIterator<Item = C>>(
        position: usize,
        length: usize,
        message: I,
        child_labels: V,
    ) -> Self {
        Self {
            position,
            length,
            message: message.into(),
            child_labels: child_labels.into_iter().map(Into::into).collect(),
        }
    }
    pub fn from_iter<
        I: Into<TokenizedLabel>,
        C: Into<TokenizedChildLabel>,
        V: IntoIterator<Item = C>,
    >(
        position: usize,
        length: usize,
        message: I,
        child_labels: V,
    ) -> Self {
        Self {
            position,
            length,
            message: message.into(),
            child_labels: child_labels.into_iter().map(Into::into).collect(),
        }
    }
    #[cfg(feature = "caret_color")]
    pub(crate) fn ref_label_color(&self) -> Option<&RgbColor> {
        self.message.ref_color()
    }
}

impl<I: Into<TokenizedLabel>, C: Into<TokenizedChildLabel>, V: IntoIterator<Item = C>>
    From<(usize, usize, I, V)> for ReportLabel
{
    fn from(value: (usize, usize, I, V)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

#[derive(Debug, Clone, derive_more::IntoIterator, derive_more::From, derive_more::Into)]
pub struct ReportLabels {
    labels: Vec<ReportCaret>,
}
impl FromIterator<ReportCaret> for ReportLabels {
    fn from_iter<T: IntoIterator<Item = ReportCaret>>(iter: T) -> Self {
        let mut labels: Vec<ReportCaret> = iter.into_iter().collect();
        labels.sort();
        Self { labels }
    }
}

impl ReportLabels {
    pub fn new() -> Self {
        Self { labels: vec![] }
    }
    pub fn push<I: Into<ReportCaret>>(&mut self, label: I) {
        let label: ReportCaret = label.into();
        if !self.labels.contains(&label) {
            self.labels.push(label);
            self.labels.sort();
        }
    }
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
    pub fn len(&self) -> usize {
        self.labels.len()
    }
    pub fn write<T: std::io::Write, A: AsRef<[Token]>>(
        &self,
        mut writer: T,
        ref_input: A,
        colored_input: bool,
        display_range: bool,
    ) -> std::io::Result<()> {
        if self.is_empty() {
            return Ok(());
        }

        let ref_input: TokenBuffer = ref_input.as_ref().into();

        self.labels.iter().try_for_each(|label| {
            let range = label.range();
            if colored_input {
                let color_ranges = label.get_underbar_ranges();
                let mut index = 0;

                if color_ranges.is_empty() {
                    write!(writer, "{:#}", ref_input)?;
                } else {
                    let ref_input_str = ref_input.to_string();

                    for (start, len, color) in color_ranges {
                        if start > index {
                            // Write plain until we reach the start
                            if let Some(ref_input_slice) = ref_input_str.get(index..start) {
                                write!(writer, "{:#}", ref_input_slice)?;
                            }
                            index = start;
                        }
                        if let Some(color) = color {
                            if let Some(ref_input_slice) =
                                ref_input_str.get(index..start.sat_add(len))
                            {
                                write!(
                                    writer,
                                    "{color}{:#}{}",
                                    ref_input_slice,
                                    AnsiStyle::RESET_COLOR
                                )?;
                            }
                        } else {
                            if let Some(ref_input_slice) =
                                ref_input_str.get(index..start.sat_add(len))
                            {
                                write!(writer, "{:#}", ref_input_slice)?;
                            }
                        }
                        index = start.sat_add(len);
                    }

                    if index < ref_input_str.len() {
                        if let Some(ref_input_slice) = ref_input_str.get(index..ref_input_str.len())
                        {
                            write!(writer, "{:#}", ref_input_slice)?;
                        }
                    }
                }
                if display_range {
                    writeln!(writer, " [{range:#}]")?;
                } else {
                    writeln!(writer)?;
                }
            } else {
                if display_range {
                    writeln!(writer, "{:#} [{range:#}]", ref_input)?;
                } else {
                    writeln!(writer, "{:#}", ref_input)?;
                }
            }

            writeln!(writer, "{:#}", label)?;
            // Just add a separator line between
            writeln!(writer)
        })
    }

    pub fn into_writer<'a, W: Write, A: AsRef<[Token]>>(
        &'a self,
        writer: &'a mut W,
        reference_input: &'a A,
        display_range: bool,
    ) -> ReportWriter<'a, W> {
        ReportWriter::new(
            writer,
            reference_input.as_ref(),
            &self.labels,
            display_range,
        )
    }
    pub fn into_writer_with<
        'a,
        W: Write,
        D: Display,
        F: FnMut(std::io::Result<()>, ReportWriterMeta) -> std::io::Result<Option<D>>,
        A: AsRef<[Token]>,
    >(
        &'a self,
        writer: &'a mut W,
        reference_input: &'a A,
        display_range: bool,
        callback: F,
    ) -> ReportWriterWith<'a, W, D, F> {
        ReportWriterWith::new(
            writer,
            reference_input.as_ref(),
            &self.labels,
            display_range,
            callback,
        )
    }
}

#[derive(Debug)]
pub struct ReportWriter<'a, W: Write> {
    writer: &'a mut W,
    reference_input: &'a [Token],
    index: usize,
    report_labels: &'a [ReportCaret],
    display_range: bool,
}
impl<'a, W: Write> ReportWriter<'a, W> {
    pub(crate) fn new(
        writer: &'a mut W,
        reference_input: &'a [Token],
        report_labels: &'a [ReportCaret],
        display_range: bool,
    ) -> Self {
        Self {
            writer,
            reference_input,
            index: 0,
            report_labels,
            display_range,
        }
    }
    pub fn write(mut self) -> std::io::Result<()> {
        self.try_for_each(|res| res)
    }
}

impl<W: Write> Iterator for ReportWriter<'_, W> {
    type Item = std::io::Result<()>;
    fn next(&mut self) -> Option<Self::Item> {
        fn write<W: Write>(
            writer: &mut W,
            label: &ReportCaret,
            reference_input: TokenBuffer,
            display_range: bool,
            is_last: bool,
        ) -> std::io::Result<()> {
            if display_range {
                let range = label.range();
                writeln!(writer, "{:#} [{range:#}]", reference_input)?;
            } else {
                writeln!(writer, "{:#}", reference_input)?;
            }
            writeln!(writer, "{:#}", label)?;
            if is_last {
                // Just add a separator line between
                writeln!(writer)
            } else {
                Ok(())
            }
        }

        let len = self.report_labels.len();
        if self.index >= len {
            return None;
        }
        let label: &ReportCaret = &self.report_labels[self.index];
        self.index += 1;
        let is_last = len == self.index.sat_add(1);
        Some(write(
            self.writer,
            label,
            token::TokenBuffer {
                buffer: self.reference_input,
            },
            self.display_range,
            is_last,
        ))
    }
}

#[derive(Debug)]
pub struct ReportWriterMeta {
    pub total: usize,
    pub index: usize,
    pub is_last: bool,
    pub is_first: bool,
    pub is_only: bool,
}
// As creation is not pub, we just need methods to ref the values
impl ReportWriterMeta {
    pub fn total(&self) -> usize {
        self.total
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn is_last(&self) -> bool {
        self.is_last
    }
    pub fn is_first(&self) -> bool {
        self.is_first
    }
    pub fn is_only(&self) -> bool {
        self.is_only
    }
}

#[derive(Debug)]
pub struct ReportWriterWith<
    'a,
    W: Write,
    D: Display,
    F: FnMut(std::io::Result<()>, ReportWriterMeta) -> std::io::Result<Option<D>>,
> {
    writer: &'a mut W,
    reference_input: &'a [Token],
    index: usize,
    report_labels: &'a [ReportCaret],
    display_range: bool,

    callback: F,

    _marker: std::marker::PhantomData<D>,
}
impl<
    'a,
    W: Write,
    D: Display,
    F: FnMut(std::io::Result<()>, ReportWriterMeta) -> std::io::Result<Option<D>>,
> ReportWriterWith<'a, W, D, F>
{
    pub(crate) fn new(
        writer: &'a mut W,
        reference_input: &'a [Token],
        report_labels: &'a [ReportCaret],
        display_range: bool,
        callback: F,
    ) -> Self {
        Self {
            writer,
            reference_input,
            index: 0,
            report_labels,
            display_range,
            callback,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn write(mut self) -> std::io::Result<()> {
        self.try_for_each(|res| res)
    }
}

impl<
    'a,
    W: Write,
    D: Display,
    F: FnMut(std::io::Result<()>, ReportWriterMeta) -> std::io::Result<Option<D>>,
> Iterator for ReportWriterWith<'a, W, D, F>
{
    type Item = std::io::Result<()>;

    fn next(&mut self) -> Option<Self::Item> {
        fn write<W: Write>(
            writer: &mut W,
            label: &ReportCaret,
            reference_input: TokenBuffer,
            display_range: bool,
            is_last: bool,
        ) -> std::io::Result<()> {
            if display_range {
                let range = label.range();
                writeln!(writer, "{:#} [{range:#}]", reference_input)?;
            } else {
                writeln!(writer, "{:#}", reference_input)?;
            }
            writeln!(writer, "{:#}", label)?;
            if is_last {
                // Just add a separator line between
                writeln!(writer)
            } else {
                Ok(())
            }
        }

        let len = self.report_labels.len();
        if self.index >= len {
            return None;
        }
        let meta = ReportWriterMeta {
            total: len,
            index: self.index,
            is_last: len == self.index.sat_add(1),
            is_first: self.index == 0,
            is_only: len == 1,
        };

        let label: &ReportCaret = &self.report_labels[self.index];
        self.index += 1;
        let is_last = len == self.index.sat_add(1);
        let res = write(
            self.writer,
            label,
            token::TokenBuffer {
                buffer: self.reference_input,
            },
            self.display_range,
            is_last,
        );

        (self.callback)(res, meta)
            .map(|display| {
                if let Some(display) = display {
                    self.writer.write_fmt(format_args!("{display}"))
                } else {
                    Ok(())
                }
            })
            .ok()
    }
}
