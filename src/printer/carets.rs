use super::*;

#[macro_export]
macro_rules! impl_field {
    ($($struct_name:ident, $field_name:ident, $field_type:ty $(, $return_type:ty as $expr:expr)?);*$(;)?) => {
        $(
            $crate::impl_field!(@coerce $struct_name, $field_name, $field_type $(, $return_type as $expr)?);
        )*
    };
    ($struct_name:ident, $field_name:ident, $field_type:ty $(, $return_type:ty as $expr:expr)?) => {
        $crate::impl_field!(@coerce $struct_name, $field_name, $field_type $(, $return_type as $expr)?);
    };
    (@coerce $struct_name:ident, $field_name:ident, $field_type:ty, $return_type:ty as $expr:expr) => {
        $crate::impl_field!(@expand $struct_name, $field_name, $field_type, $return_type, $expr);
    };
    (@coerce $struct_name:ident, $field_name:ident, $field_type:ty) => {
        $crate::impl_field!(@expand $struct_name, $field_name, $field_type, $field_type);
    };
    (@expand $struct_name:ident, $field_name:ident, $field_type:ty, $return_type:ty $(, $expr:expr)?) => {
        impl $struct_name {
            paste::paste! {
                pub fn [< $field_name >](&self) -> $return_type
                where
                    $field_type: Copy
                {
                    // If expr is Some, apply it
                    $crate::impl_field!(@expand_expr self.$field_name $(=> $expr)?)
                }
                pub fn [< $field_name _cloned>](&self) -> $return_type
                where
                    $field_type: Clone
                {
                    // If expr is Some, apply it
                    $crate::impl_field!(@expand_expr self.$field_name.clone() $(=> $expr)?)
                }
                pub fn [< $field_name _ref>]<'a>(&'a self) -> &'a $return_type {
                    // If expr is Some, apply it
                    $crate::impl_field!(@expand_expr &self.$field_name $(=> $expr)?)
                }
                pub fn [< $field_name _mut>]<'a>(&'a mut self) -> &'a mut $return_type {
                    // If expr is Some, apply it
                    $crate::impl_field!(@expand_expr &mut self.$field_name $(=> $expr)?)
                }
                $(
                    pub fn [< $field_name _as >]<T>(&self) -> T
                    where
                        T: From<$field_type>,
                    {
                        // If expr is Some, apply it
                        $crate::impl_field!(@expand_expr self.$field_name.clone().into() => $expr)
                    }
                )?
            }
        }
    };
    (@expand_expr $field:expr => $expr:expr) => {
        $field.$expr
    };
    (@expand_expr $field:expr ) => {{
        $field
    }}

}

crate::impl_field!(
    ReportCaret,start,usize;
    ReportCaret,end,usize;
    ReportCaret,r_positions,Vec<ReportLabel>;
    UnderbarLine,underbar,TokenStream;
    UnderbarLine,underbar_sep,TokenStream;
    CaretLine,main,TokenStream;
    ReportLabel,position,usize;
    ReportLabel,message,TokenStream;
    ReportLabel,child_labels,Vec<TokenStream>;
);

#[derive(Debug, Clone)]
pub struct UnderbarLine {
    underbar: TokenStream,
    underbar_sep: TokenStream,
}

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

#[derive(Debug, Clone)]
pub(self) enum Line {
    Sep(TokenStream),
    Underbar(TokenStream),
    LabelWithCaret(TokenStream),
}
impl Line {
    pub fn into_inner(self) -> TokenStream {
        match self {
            Line::Sep(sep) => sep,
            Line::Underbar(underbar) => underbar,
            Line::LabelWithCaret(main) => main,
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
            Line::LabelWithCaret(main) => {
                write!(f, "{}{}", main, f.alternate().then(|| "\n").unwrap_or(""))
            }
        }
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
    pub(self) fn format(mut self) -> Option<Vec<Line>> {
        let mut lines: Vec<Line> = vec![];

        if self.is_empty() {
            return None;
        }

        let mut underbar = TokenStream::new();
        let mut underbar_sep = TokenStream::new();

        underbar.push(Token::Space(self.start));

        let mut last_index = 0;

        for label in self.iter().rev() {
            let pos = label.position();
            let sep = pos.saturating_sub(last_index);
            underbar.push(Token::HCaret(sep));
            underbar.push(Token::HDown);
            last_index = pos + 1;
        }
        if last_index < self.end.saturating_sub(self.start) {
            underbar.push(Token::HCaret(
                self.end
                    .saturating_sub(self.start)
                    .saturating_sub(last_index),
            ));
        }

        lines.push(Line::Underbar(underbar));

        underbar_sep.push(Token::Space(self.start));

        let mut last_index = 0;

        for label in self.iter().rev() {
            let pos = label.position();
            let sep = pos.saturating_sub(last_index);
            underbar_sep.push(Token::Space(sep));
            underbar_sep.push(Token::VCaret);
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

            for (i, label) in self.iter().rev().enumerate() {
                let pos = label.position();
                // Insert spaces until we reach the next position, if its the first position, else we draw H_CARET
                if i == 0 {
                    label_line.push(Token::Space(pos.saturating_sub(current_pos)));
                } else {
                    label_line.push(Token::HCaret(pos.saturating_sub(current_pos)));
                }

                current_pos = pos + 1;
                if i == 0 {
                    // Transition from H_CARET to UP_RIGHT
                    label_line.push([Token::UpRight, Token::HCaret(1)]);
                } else {
                    label_line.push(Token::HCaret(1));
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
                label_line.push([Token::LArrow, Token::Space(1)]);
                label_line.extend(message);
                lines.push(Line::LabelWithCaret(label_line));
            } else {
                label_line.push([
                    Token::HCaret(1),
                    Token::HDown,
                    Token::HCaret(2),
                    Token::LArrow,
                    Token::Space(1),
                ]);
                label_line.extend(message);
                lines.push(Line::LabelWithCaret(label_line));
                // Now we wanna print the child labels
                // The returned separator line reaches until the last caret of the last label
                // so we need to further extend it to the current parent label + 2 (for the arrow-transition)
                let mut child_sep = self
                    .get_separator_line()
                    .unwrap_or(Line::Sep(TokenStream::new()))
                    .into_inner();

                // Add spaces until we reach the caret of the parent label + 2 (for the arrow-transition)
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
                child_sep_with_caret.push(Token::VCaret);
                lines.push(Line::Sep(child_sep_with_caret.clone()));
                let child_labels_len = child_labels.len();
                for (i, child) in child_labels.into_iter().enumerate() {
                    // We can "unsafely" sub here, as the for loop ensures that child_labels_len > 0
                    let is_last_child_label = i == child_labels_len - 1;
                    // Each child label is prepended by the same "child_sep" as that resembles the carets of the other labels
                    let mut child_line = child_sep.clone();
                    if is_last_child_label {
                        child_line.push([
                            Token::UpRight,
                            Token::HCaret(3),
                            Token::LArrow,
                            Token::Space(1),
                        ]);
                    } else {
                        child_line.push([
                            Token::VRight,
                            Token::HCaret(3),
                            Token::LArrow,
                            Token::Space(1),
                        ]);
                    }
                    child_line.extend(child);
                    // Add the child line
                    lines.push(Line::LabelWithCaret(child_line));
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
            sep.push(Token::VCaret);
        }
        Some(Line::Sep(sep))
    }
}
impl Display for ReportCaret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(formatted) = self.clone().format() {
            formatted
                .into_iter()
                .try_for_each(|cl| write!(f, "{:#}", cl))
        } else {
            std::fmt::Result::Err(std::fmt::Error)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReportLabel {
    pub position: usize,
    pub length: usize,
    pub message: TokenStream,
    pub child_labels: Vec<TokenStream>,
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
    pub fn new<I: Into<TokenStream>, C: Into<TokenStream>, V: IntoIterator<Item = C>>(
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
    pub fn from_iter<I: Into<TokenStream>, C: Into<TokenStream>, V: IntoIterator<Item = C>>(
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
}

impl<I: Into<TokenStream>, C: Into<TokenStream>, V: IntoIterator<Item = C>>
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
    ) -> std::io::Result<()> {
        let ref_input: TokenBuffer = ref_input.as_ref().into();

        for caret in self.labels.iter() {
            writeln!(writer, "{:#}", ref_input)?;
            writeln!(writer, "{:#}", caret)?;
        }
        Ok(())
    }
}
