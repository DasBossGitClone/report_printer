use super::*;

/// The amount of white-space padding to add directly after the arrow and before the label message
const ARROR_LABEL_PADDING: usize = 1;

/// The offset from the parent label to the child labels
const CHILD_LABEL_OFFSET: usize = 3;

crate::impl_field!(
    ReportCaret,start,usize;
    ReportCaret,end,usize;
    ReportCaret,r_positions,Vec<ReportLabel>;
    CaretLine,main,TokenStream;
    ReportLabel,position,usize;
    ReportLabel,message,TokenizedLabel;
    ReportLabel,child_labels,Vec<TokenizedChildLabel>;
);

#[derive(Debug, Clone, derive_more::Into, derive_more::From)]
pub struct CaretLine {
    main: TokenStream,
    sep: Option<TokenStream>,
}
impl From<(Option<TokenStream>, TokenStream)> for CaretLine {
    fn from(value: (Option<TokenStream>, TokenStream)) -> Self {
        Self {
            main: value.1,
            sep: value.0,
        }
    }
}
impl CaretLine {
    pub fn main_with<I: Into<TokenStream>>(&self, msg: I) -> LineTokenStream {
        let mut line = LineTokenStream::new();
        line.push_new(&self.main);
        line.push_new(msg);
        line
    }
    pub fn separator(&self) -> Option<&TokenStream> {
        self.sep.as_ref()
    }
}

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
            let mut until_last = iter.rev().take(total - 1);
            until_last.try_for_each(|line| write!(f, "{line:#}"))?;
            write!(f, "{last}")
        }
    }
}

#[derive(Debug, Clone)]
pub(self) enum Line {
    Sep(TokenStream),
    Underbar(TokenStream),
    // Label with the carets
    Label(TokenStream),
    // Mutliline Label with the carets
    // The first line is Self::Label and the rest are
    // Self::LabelSeq (Label Sequence)
    LabelSeq(TokenStream),
}
impl Line {
    pub fn into_inner(self) -> TokenStream {
        match self {
            Line::Sep(sep) => sep,
            Line::Underbar(underbar) => underbar,
            Line::LabelSeq(main) | Line::Label(main) => main,
        }
    }
}
impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Line::Sep(sep) => write!(f, "{}{}", sep, f.alternate().then(|| "\n").unwrap_or("")),
            Line::Underbar(underbar) => {
                write!(
                    f,
                    "{}{}",
                    underbar,
                    f.alternate().then(|| "\n").unwrap_or("")
                )
            }
            Line::LabelSeq(main) | Line::Label(main) => {
                write!(f, "{}{}", main, f.alternate().then(|| "\n").unwrap_or(""))
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
    r_positions: Vec<ReportLabel>,
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
            .then(self.r_positions.len().cmp(&other.r_positions.len()))
            .then_with(|| {
                self.r_positions
                    .iter()
                    .rev()
                    .zip(other.r_positions.iter().rev())
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
    pub(super) fn new(start: usize, end: usize, mut r_positions: Vec<ReportLabel>) -> Self {
        r_positions.sort_by(|a, b| a.cmp(b));
        r_positions.dedup();
        r_positions.reverse();
        Self {
            start,
            end,
            r_positions,
        }
    }
    pub fn range(&self) -> RangeInclusive {
        (self.start..=self.end).into()
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &ReportLabel> {
        self.r_positions.iter()
    }
    pub fn push<I: Into<ReportLabel>>(&mut self, label: I) -> bool {
        let label: ReportLabel = label.into();
        if label.position() > self.end.saturating_sub(self.start) {
            false
        } else {
            self.r_positions.push(label.into());
            self.r_positions.sort_by(|a, b| a.cmp(b));
            self.r_positions.dedup();
            self.r_positions.reverse();
            true
        }
    }
    pub fn pop(&mut self) -> Option<ReportLabel> {
        self.r_positions.pop()
    }
    pub fn is_empty(&self) -> bool {
        self.r_positions.is_empty()
    }
    pub fn len(&self) -> usize {
        self.r_positions.len()
    }
    pub(self) fn format(mut self) -> Option<Lines> {
        let mut lines = Lines::new();

        if self.is_empty() {
            return None;
        }

        let mut underbar = TokenStream::new();
        let mut underbar_sep = TokenStream::new();

        underbar.push(Token::Space(self.start));

        let mut last_index = 0;

        let mut last_color: Option<&RgbColor> = None;

        for (i, label) in self.iter().rev().enumerate() {
            let pos = label.position();

            let sep = pos.saturating_sub(last_index);

            #[cfg(feature = "caret_color")]
            {
                last_color = label.message.ref_color()
            };
            #[cfg(not(feature = "caret_color"))]
            {
                last_color = None
            };
            underbar.push(Token::HCaret(sep).try_with_coloring_feature(last_color));
            underbar.push(Token::HDown.try_with_coloring_feature(last_color));
            last_index = pos + 1;

            if let Some(next) = self.iter().rev().nth(i + 1) {
                let next_pos = next.position();

                // Cover the underbar until the "next" label

                // We add 2, as we need to cover the HDown and the position of the next label
                let next_sep = next_pos.saturating_sub(last_index + 2);
                underbar.push(Token::HCaret(next_sep).try_with_coloring_feature(last_color));
                last_index += next_sep;
            }
        }
        if last_index < self.end.saturating_sub(self.start) {
            underbar.push(
                Token::HCaret(
                    self.end
                        .saturating_sub(self.start)
                        .saturating_sub(last_index),
                )
                .try_with_coloring_feature(last_color),
            );
        }

        lines.push(Line::Underbar(underbar));

        underbar_sep.push(Token::Space(self.start));

        let mut last_index = 0;

        for label in self.iter().rev() {
            let pos = label.position();
            let sep = pos.saturating_sub(last_index);

            #[cfg(feature = "caret_color")]
            let label_caret_color = label.message.ref_color();
            #[cfg(not(feature = "caret_color"))]
            let label_caret_color: Option<&RgbColor> = None;

            underbar_sep.push(Token::Space(sep));
            underbar_sep.push(Token::VCaret.try_with_coloring_feature(label_caret_color));
            last_index = pos + 1;
        }
        if last_index < self.end.saturating_sub(self.start) {
            underbar_sep.push(Token::Space(
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
            label_line.push(Token::Space(self.start));

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
                    label_line.push(Token::Space(pos.saturating_sub(current_pos)));
                } else {
                    label_line.push(
                        Token::HCaret(pos.saturating_sub(current_pos))
                            .try_with_coloring_feature(label_color),
                    );
                }

                current_pos = pos + 1;

                if i == 0 {
                    // Transition from H_CARET to UP_RIGHT
                    label_line.push([
                        Token::UpRight.try_with_coloring_feature(label_color),
                        Token::HCaret(1).try_with_coloring_feature(label_color),
                    ]);
                } else {
                    label_line.push(Token::HCaret(1).try_with_coloring_feature(label_color));
                }
            }
            // pop the position, so we dont print it again in the separator line
            let last = self.r_positions.pop().unwrap();

            // Write the label
            let ReportLabel {
                message,
                child_labels,
                position: parent_label_position,
                ..
            } = last;
            if child_labels.is_empty() {
                // We can print a short arrow
                for line in message.into_iter() {
                    let mut label_line = label_line.clone();
                    label_line.push([
                        if line.is_only() {
                            Token::LArrow
                        } else {
                            Token::VCaret
                        },
                        Token::Space(ARROR_LABEL_PADDING),
                    ]);
                    label_line.extend(line);
                    lines.push(Line::Label(label_line));
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

                    // Add spaces until we reach the caret of the parent label + 2 (for the arrow-transition)
                    let current_pos = (sep).lit_len();
                    if current_pos == 0 {
                        let target_pos = parent_label_position
                            // 2 for the arrow-transition
                            .saturating_add(3)
                            .saturating_add(self.start);
                        if target_pos > current_pos {
                            sep.push(Token::Space(target_pos.saturating_sub(current_pos)));
                        }
                    } else {
                        let target_pos = current_pos
                            // 2 for the arrow-transition
                            .saturating_add(2);
                        if target_pos > current_pos {
                            sep.push(Token::Space(target_pos.saturating_sub(current_pos)));
                        }
                    }

                    let mut message = message.into_iter();
                    if let Some(line) = message.next() {
                        label_line.push([
                            pcc(Token::HCaret(1)),
                            pcc(Token::HDown),
                            pcc(Token::HCaret(2)),
                            pcc(Token::VLeft),
                            Token::Space(ARROR_LABEL_PADDING),
                        ]);
                        label_line.extend(line);
                        lines.push(Line::Label(label_line));

                        for line in message {
                            let mut label_line = sep.clone();
                            label_line.push([
                                pcc(Token::VCaret),
                                Token::Space(2),
                                pcc(Token::VCaret),
                                // offset by 1 to indicate that the line was split
                                Token::Space(ARROR_LABEL_PADDING + 1),
                            ]);
                            label_line.extend(line);
                            lines.push(Line::LabelSeq(label_line));
                        }
                    } else {
                        panic!("Message is multi-line, but has no lines");
                    }
                } else {
                    label_line.push([
                        pcc(Token::HCaret(1)),
                        pcc(Token::HDown),
                        pcc(Token::HCaret(2)),
                        pcc(Token::LArrow),
                        Token::Space(ARROR_LABEL_PADDING),
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
                        child_sep.push(Token::Space(target_pos.saturating_sub(current_pos)));
                    }
                } else {
                    let target_pos = current_pos
                        // 2 for the arrow-transition
                        .saturating_add(2);
                    if target_pos > current_pos {
                        child_sep.push(Token::Space(target_pos.saturating_sub(current_pos)));
                    }
                }
                // We wanna clone it here, as this is the separator for all child labels
                let mut child_sep_with_caret = child_sep.clone();
                child_sep_with_caret.push(pcc(Token::VCaret));
                lines.push(Line::Sep(child_sep_with_caret.clone()));
                let child_labels_len = child_labels.len();
                for (i, child) in child_labels.into_iter().enumerate() {
                    // We can "unsafely" sub here, as the for loop ensures that child_labels_len > 0
                    let is_last_child_label = i == child_labels_len - 1;

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
                        // CHILD_LABEL_OFFSET + 2 as we need to offset the VCaret by 2, otherwise they would directly be next to the caret for the follwing labels
                        match (
                            child_label_line.is_first(),
                            is_last_child_label,
                            child_label_line.is_only(),
                        ) {
                            // Only line in the last child label
                            (true, true, true) => {
                                child_line.push([
                                    ccc(Token::UpRight),
                                    ccc(Token::HCaret(CHILD_LABEL_OFFSET + 2)),
                                    ccc(Token::LArrow),
                                    Token::Space(ARROR_LABEL_PADDING),
                                ]);
                            }
                            // Only line in label-child, but not the last child label
                            (true, false, true) => {
                                child_line.push([
                                    pcc(Token::VRight),
                                    ccc(Token::HCaret(CHILD_LABEL_OFFSET + 2)),
                                    ccc(Token::LArrow),
                                    Token::Space(ARROR_LABEL_PADDING),
                                ]);
                            }
                            // First line but not last in label-child, not last child label
                            (true, true, false) => {
                                child_line.push([
                                    pcc(Token::UpRight),
                                    ccc(Token::HCaret(CHILD_LABEL_OFFSET + 2)),
                                    ccc(Token::VLeft),
                                    Token::Space(ARROR_LABEL_PADDING),
                                ]);
                            }
                            // First line but not last in label-child, last child label
                            (true, false, false) => {
                                child_line.push([
                                    pcc(Token::VRight),
                                    ccc(Token::HCaret(CHILD_LABEL_OFFSET + 2)),
                                    ccc(Token::VLeft),
                                    Token::Space(ARROR_LABEL_PADDING),
                                ]);
                            }
                            // not last Child label, but not the only line
                            (_, false, false) => {
                                child_line.push([
                                    // We do not wanna style the first caret, as it is the continuation of the parent label
                                    Token::VCaret,
                                    Token::Space(CHILD_LABEL_OFFSET + 2),
                                    ccc(Token::VCaret),
                                    // offset by 1 to indicate that the line was split
                                    Token::Space(ARROR_LABEL_PADDING + 1),
                                ]);
                            }
                            // Last Child label, but not the only line
                            (_, true, false) => {
                                child_line.push([
                                    // We need to add an extra space here, as there are not more child labels, thus no carets which would offset the line
                                    Token::Space(CHILD_LABEL_OFFSET + 3),
                                    ccc(Token::VCaret),
                                    // offset by 1 to indicate that the line was split
                                    Token::Space(ARROR_LABEL_PADDING + 1),
                                ]);
                            }
                            (_, _, true) => {
                                child_line.push([Token::VCaret, Token::Space(5)]);
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

        Some(lines)
    }
    fn get_separator_line(&self) -> Option<Line> {
        if self.is_empty() {
            return None;
        }
        let mut sep: TokenStream = TokenStream::new();
        sep.push(Token::Space(self.start));
        let mut current_pos = 0;

        for label in self.iter().rev() {
            let pos = label.position();
            // Insert spaces until we reach the next position
            sep.push(Token::Space(pos.saturating_sub(current_pos)));
            current_pos = pos + 1;
            #[cfg(feature = "caret_color")]
            sep.push(Token::VCaret.try_with_coloring_feature(label.ref_label_color()));

            #[cfg(not(feature = "caret_color"))]
            sep.push(Token::VCaret);
        }
        Some(Line::Sep(sep))
    }
}
impl Display for ReportCaret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(formatted) = self.clone().format() {
            write!(f, "{formatted}")
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
        display_range: bool,
    ) -> std::io::Result<()> {
        if self.is_empty() {
            return Ok(());
        }

        let ref_input: TokenBuffer = ref_input.as_ref().into();

        let len = self.len();

        let mut iter: std::iter::Rev<std::slice::Iter<'_, ReportCaret>> = self.labels.iter().rev();

        let last: &ReportCaret = iter.next().expect("No labels");

        if len != 1 {
            iter.rev().take(len - 1).try_for_each(|label| {
                if display_range {
                    let range = label.range();
                    writeln!(writer, "{:#} [{range:#}]", ref_input)?;
                } else {
                    writeln!(writer, "{:#}", ref_input)?;
                }
                writeln!(writer, "{:#}", label)?;
                // Just add a separator line between
                writeln!(writer)
            })?;
        }
        if display_range {
            let range = last.range();
            writeln!(writer, "{:#} [{range:#}]", ref_input)?;
        } else {
            writeln!(writer, "{:#}", ref_input)?;
        }
        writeln!(writer, "{:#}", last)
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
        let is_last = len == self.index + 1;
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
