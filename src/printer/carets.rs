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
    CaretBuilder,start,usize;
    CaretBuilder,end,usize;
    CaretBuilder,positions,Vec<usize>;
    Caret,start,usize;
    Caret,end,usize;
    Caret,r_positions,Vec<usize>;
);

#[derive(Debug, Clone)]
/// The vertical line character before the labels to visually connect them to the underbar
pub struct CaretBuilder {
    /// Start of underbar
    start: usize,
    /// End of underbar
    end: usize,
    /// The positions of the down facing connectors (┬) in the underbar
    /// Relative to start
    positions: Vec<usize>,
}
impl CaretBuilder {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            positions: Vec::new(),
        }
    }
    pub(super) fn push(&mut self, pos: usize) -> bool {
        if pos > self.end.saturating_sub(self.start) {
            false
        } else {
            self.positions.push(pos);
            true
        }
    }
    pub fn pop(&mut self) -> Option<usize> {
        self.positions.pop()
    }
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    pub fn finish(mut self) -> Option<Caret> {
        if self.positions.is_empty() {
            return None;
        }
        self.positions.sort_by(|a, b| a.cmp(b));
        self.positions.dedup();
        // Reverse it so that popping gets the leftmost position first
        self.positions.reverse();
        Some(Caret {
            start: self.start,
            end: self.end,
            r_positions: self.positions,
        })
    }
}

#[derive(Debug, Clone, derive_more::IntoIterator)]
pub struct Caret {
    /// Start of underbar
    start: usize,
    /// End of underbar
    end: usize,
    /// Relative to start
    /// Reversed, so that popping gets the leftmost position first
    #[into_iterator]
    r_positions: Vec<usize>,
}
impl Caret {
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &usize> {
        self.r_positions.iter()
    }
    pub fn push(&mut self, pos: usize) -> bool {
        if pos > self.end.saturating_sub(self.start) {
            false
        } else {
            self.r_positions.push(pos);
            self.r_positions.sort_by(|a, b| a.cmp(b));
            self.r_positions.dedup();
            self.r_positions.reverse();
            true
        }
    }
    pub fn pop(&mut self) -> Option<usize> {
        self.r_positions.pop()
    }
    pub fn is_empty(&self) -> bool {
        self.r_positions.is_empty()
    }
    pub fn len(&self) -> usize {
        self.r_positions.len()
    }
    pub fn format(mut self) -> Option<FormattedCaretSegment> {
        let mut lines: Vec<CaretLine> = vec![];

        if self.is_empty() {
            return None;
        }

        let mut underbar = TokenStream::new();
        let mut underbar_sep = TokenStream::new();

        underbar.push(Token::Space(self.start()));

        let mut last_index = 0;

        for pos in self.iter().rev() {
            let sep = pos.saturating_sub(last_index);
            underbar.push_str(REP(sep, H_CARET));
            underbar.push_str(H_DOWN);
            last_index = pos + 1;
        }
        if last_index < self.end().saturating_sub(self.start()) {
            underbar.push_str(
                H_CARET.repeat(
                    self.end()
                        .saturating_sub(self.start())
                        .saturating_sub(last_index),
                ),
            );
        }

        underbar_sep.push(Token::Space(self.start()));

        let mut last_index = 0;

        for pos in self.iter().rev() {
            let sep = pos.saturating_sub(last_index);
            underbar_sep.push(Token::Space(sep));
            underbar_sep.push_str(V_CARET);
            last_index = pos + 1;
        }
        if last_index < self.end().saturating_sub(self.start()) {
            underbar_sep.push(Token::Space(
                self.end()
                    .saturating_sub(self.start())
                    .saturating_sub(last_index),
            ));
        }

        // Generate each caret line
        // each line "arrows" the first position to the end
        // followed by a "sepeartor" line containing all V_CARETs at the given positions
        while !self.is_empty() {
            // First generate the main line
            let mut line = TokenStream::new();
            line.push(Token::Space(self.start()));

            let mut current_pos = 0;

            for (i, pos) in self.iter().rev().enumerate() {
                // Insert spaces until we reach the next position, if its the first position, else we draw H_CARET
                if i == 0 {
                    line.push(Token::Space(pos.saturating_sub(current_pos)));
                } else {
                    line.push_str(REP(pos.saturating_sub(current_pos), H_CARET));
                }

                current_pos = *pos + 1;
                if i == 0 {
                    // Transition from H_CARET to UP_RIGHT
                    line.push_str(UP_RIGHT);
                    line.push_str(H_CARET);
                } else {
                    line.push_str(H_CARET);
                }
            }
            // pop the position, so we dont print it again in the separator line
            let _ = self.r_positions_mut().pop().unwrap();
            if !self.is_empty() {
                // Generate the separator line
                let mut sep: TokenStream = TokenStream::new();
                sep.push(Token::Space(self.start()));
                current_pos = 0;
                for pos in self.iter().rev() {
                    // Insert spaces until we reach the next position
                    sep.push_str((SPACE(pos.saturating_sub(current_pos))).as_str());
                    current_pos = *pos + 1;
                    sep.push_str(V_CARET);
                }
                lines.push((line, Some(sep)).into());
            } else {
                lines.push((line, None).into());
            }
        }

        Some(FormattedCaretSegment::new(underbar, underbar_sep, lines))
    }
}

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
    pub fn main(&self) -> &TokenStream {
        &self.main
    }
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
pub struct CaretsBuilder {
    segments: Vec<Caret>,
}
impl CaretsBuilder {
    pub fn new() -> Self {
        Self { segments: vec![] }
    }
    pub fn push(&mut self, seg: CaretBuilder) {
        if let Some(seg) = seg.finish() {
            self.segments.push(seg);
        }
    }
    pub fn finish(mut self) -> Vec<Caret> {
        self.segments.sort_by(|a, b| a.start.cmp(&b.start));
        self.segments
    }
}
