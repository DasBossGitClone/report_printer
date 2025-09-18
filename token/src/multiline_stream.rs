use crate::saturating::SaturatingArithmetic;

use super::*;

#[derive(Clone, PartialEq, Eq, Hash, derive_more::Into)]
pub struct LineTokenStream {
    /// A vector of TokenStream, where each element represents a line
    /// It must be noted, that each TokenStream CANNOT contain newlines nor be empty,
    /// so it is not possible to have more or less lines than the length of this vector
    pub tokens: Vec<TokenStream>,
}
impl Debug for LineTokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "alt_debug")]
        {
            if f.alternate() {
                // Pretty print
                f.debug_struct("LineTokenStream")
                    .field("total_lines", &self.tokens.len())
                    .field(
                        "lines",
                        &self
                            .tokens
                            .iter()
                            .map(|line| format!("{:#}\n", line))
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .finish()
            } else {
                // Default debug implementation
                f.debug_struct("LineTokenStream")
                    .field("tokens", &self.tokens)
                    .finish()
            }
        }
        #[cfg(not(feature = "alt_debug"))]
        {
            // Default debug implementation
            f.debug_struct("LineTokenStream")
                .field("tokens", &self.tokens)
                .finish()
        }
    }
}

impl Display for LineTokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tokens
            .iter()
            .try_for_each(|line| Display::fmt(line, f))?;
        Ok(())
    }
}
impl LineTokenStream {
    #[inline]
    pub fn push_new<I: Into<TokenStream>>(&mut self, item: I) {
        self.tokens.push(item.into());
    }
    #[inline]
    pub fn push<I: Into<TokenStream>>(&mut self, item: I) {
        let item: TokenStream = item.into();
        if let Some(last) = self.tokens.last_mut() {
            last.extend(item);
        } else {
            self.tokens.push(item);
        }
    }
    pub fn lines<'a>(&'a self) -> impl Iterator<Item = &'a TokenStream> + 'a {
        self.tokens.iter()
    }
    pub fn lines_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut TokenStream> + 'a {
        self.tokens.iter_mut()
    }
    pub fn lines_count(&self) -> usize {
        self.tokens.len()
    }
    pub fn is_multi_line(&self) -> bool {
        self.tokens.len() > 1
    }
    pub fn try_fold_lines<B, F>(&self, init: B, f: F) -> B
    where
        F: FnMut(B, &TokenStream) -> B,
    {
        self.tokens.iter().fold(init, f)
    }
    pub fn try_rfold_lines<B, F>(&self, init: B, f: F) -> B
    where
        F: FnMut(B, &TokenStream) -> B,
    {
        self.tokens.iter().rfold(init, f)
    }
    pub fn try_into_token_stream(self) -> Result<TokenStream, Self> {
        if self.tokens.len() == 1 {
            Ok(self.tokens.into_iter().next().unwrap())
        } else {
            Err(self)
        }
    }
    #[inline]
    pub fn pop(&mut self) -> Option<TokenStream> {
        self.tokens.pop()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = TokenStreamLine> {
        TokenStreamLineIntoIter::new(self.clone()).into_iter()
    }
    #[inline]
    pub fn last(&self) -> Option<&TokenStream> {
        self.tokens.last()
    }
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut TokenStream> {
        self.tokens.last_mut()
    }
    pub fn break_line(&mut self) {
        self.tokens.push(TokenStream::new());
    }
    pub fn push_str<A: AsRef<str>>(&mut self, s: A) {
        let s = s.as_ref().replace("\r", "");
        for line in s.split_inclusive('\n') {
            if line == "\n" {
                self.tokens.push(TokenStream::new());
                continue;
            }
            let mut line = line.trim_end_matches('\n').to_string();
            if let Ok(stream) = TokenStream::from_str(&line) {
                self.tokens.push(stream);
            } else {
                // Fallback: parse manually
                // Should never happen, but just in case
                loop {
                    if let Some((token, rem)) = Token::parse_from_str(&line) {
                        if self.tokens.is_empty() {
                            self.tokens.push(TokenStream::new());
                        }
                        self.tokens.last_mut().unwrap().push(token);
                        if let Some(rem) = rem {
                            line = rem;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
    pub fn on_color_last<I: Into<AnsiStyle>>(&mut self, style: I) {
        if let Some(last) = self.tokens.last_mut() {
            last.on_color(style);
        }
    }
    pub fn on_color_all<I: Into<AnsiStyle>>(&mut self, style: I) {
        let style = style.into();
        for line in &mut self.tokens {
            // implements copy
            line.on_color(style);
        }
    }
    pub fn new() -> Self {
        Self { tokens: vec![] }
    }
    pub fn from_str_with_length<A: AsRef<str>>(s: A, max_line_length: usize) -> Self {
        let s = s.as_ref().replace("\r", "");
        let mut stream = Self::new();
        let mut current_line = TokenStream::new();
        for line in s.split_inclusive('\n') {
            if line == "\n" {
                // New line
                stream.break_line();
                continue;
            }
            let mut line = line.trim_end_matches('\n').to_string();
            if line.len() > max_line_length {
                // Need to break the line
                // We wanna try to break it at whitespace but at max "max_line_length" if possible, otherwise break it with a offset of 1 and add a hyphen
                while line.len() > max_line_length {
                    let break_at = line[..max_line_length]
                        .rfind(char::is_whitespace)
                        .unwrap_or(max_line_length.sat_sub(1));
                    // Check if we can break within range
                    if break_at > max_line_length {
                        // We need to break with a hyphen
                        let (part, rem) = line.split_at(max_line_length.sat_sub(1));
                        let rem = rem.trim_start().to_string();
                        // Add a hyphen to the part
                        let mut part = part.to_string();
                        part.push('-');
                        if let Ok(line_stream) = TokenStream::from_str(&part) {
                            current_line.extend(line_stream);
                        } else {
                            // Fallback: parse manually
                            loop {
                                if let Some((token, rem)) = Token::parse_from_str(&part) {
                                    current_line.push(token);
                                    if let Some(rem) = rem {
                                        part = rem.to_string();
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        stream.push_new(current_line);
                        current_line = TokenStream::new();
                        line = rem.to_string();
                    } else {
                        // We can break at whitespace
                        let (part, rem) = line.split_at(break_at);
                        let rem = rem.trim_start().to_string();
                        let mut part = part.trim_end().to_string();
                        if let Ok(line_stream) = TokenStream::from_str(&part) {
                            current_line.extend(line_stream);
                        } else {
                            // Fallback: parse manually
                            loop {
                                if let Some((token, rem)) = Token::parse_from_str(&part) {
                                    current_line.push(token);
                                    if let Some(rem) = rem {
                                        part = rem.to_string();
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        stream.push_new(current_line);
                        current_line = TokenStream::new();
                        line = rem.trim_start().to_string();
                    }
                }
            } else {
                // Just add the line as is
                if let Ok(line_stream) = TokenStream::from_str(&line) {
                    current_line.extend(line_stream);
                } else {
                    // Fallback: parse manually
                    loop {
                        if let Some((token, rem)) = Token::parse_from_str(&line) {
                            current_line.push(token);
                            if let Some(rem) = rem {
                                line = rem.to_string();
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
            stream.push_new(current_line);
            current_line = TokenStream::new();
        }
        if !current_line.is_empty() || stream.tokens.is_empty() {
            stream.push_new(current_line);
        }
        stream
    }
}

impl<I: Into<TokenStream>> From<I> for LineTokenStream {
    fn from(value: I) -> Self {
        Self {
            tokens: vec![value.into()],
        }
    }
}

impl FromStr for LineTokenStream {
    type Err = ();

    /// This implementation cannot fail
    ///
    /// The return type is required by the trait
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stream = LineTokenStream::new();
        stream.push_str(s);
        Ok(stream)
    }
}

impl IntoIterator for LineTokenStream {
    type Item = TokenStreamLine;
    type IntoIter = TokenStreamLineIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        TokenStreamLineIntoIter::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TokenStreamLine {
    pub index: usize,
    pub lines: usize,
    pub is_first: bool,
    pub is_last: bool,
    pub is_only: bool,
    pub line: TokenStream,
}
impl TokenStreamLine {
    pub fn is_first(&self) -> bool {
        self.is_first
    }
    pub fn is_last(&self) -> bool {
        self.is_last
    }
    pub fn is_only(&self) -> bool {
        self.is_only
    }
    pub fn lit_len(&self) -> usize {
        self.line.lit_len()
    }
    pub fn len(&self) -> usize {
        self.line.len()
    }
    pub fn is_empty(&self) -> bool {
        self.line.is_empty()
    }
}
impl IntoIterator for TokenStreamLine {
    type Item = Token;
    type IntoIter = std::vec::IntoIter<Token>;

    fn into_iter(self) -> Self::IntoIter {
        self.line.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Into)]
/// An abstraction over multiple lines of TokenStream
/// Adds additional metadata for iteration
/// such as if its the only, first or last line etc.
pub struct TokenStreamLineIntoIter {
    stream: LineTokenStream,
    index: usize,
}

impl TokenStreamLineIntoIter {
    pub fn new(stream: LineTokenStream) -> Self {
        Self { stream, index: 0 }
    }
    pub fn len(&self) -> usize {
        self.stream.len()
    }
    pub fn is_empty(&self) -> bool {
        self.stream.is_empty()
    }
}

impl Iterator for TokenStreamLineIntoIter {
    type Item = TokenStreamLine;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.stream.len() {
            return None;
        }
        let mut item = self.stream.tokens[self.index].clone();
        // Ensure we skip empty lines,
        // although they should not be present
        while item.is_empty() {
            self.index += 1;
            if self.index >= self.stream.len() {
                return None;
            }
            item = self.stream.tokens[self.index].clone();
        }
        let is_first = self.index == 0;
        let is_last = self.index == self.stream.len().sat_sub(1);
        let is_only = is_first && is_last;
        let lines = self.stream.len();
        let index = self.index;

        self.index += 1;
        Some(TokenStreamLine {
            index,
            is_first,
            is_last,
            is_only,
            lines,
            line: item,
        })
    }
}

impl DoubleEndedIterator for TokenStreamLineIntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        let mut item = self.stream.tokens[self.index].clone();
        // Ensure we skip empty lines,
        // although they should not be present
        while item.is_empty() {
            if self.index == 0 {
                return None;
            }
            self.index -= 1;
            item = self.stream.tokens[self.index].clone();
        }
        let is_first = self.index == 0;
        let is_last = self.index == self.stream.len().sat_sub(1);
        let is_only = is_first && is_last;
        let lines = self.stream.len();
        let index = self.index;

        Some(TokenStreamLine {
            index,
            is_first,
            is_last,
            is_only,
            lines,
            line: item,
        })
    }
}

impl AsRef<LineTokenStream> for LineTokenStream {
    fn as_ref(&self) -> &LineTokenStream {
        self
    }
}

impl AsMut<LineTokenStream> for LineTokenStream {
    fn as_mut(&mut self) -> &mut LineTokenStream {
        self
    }
}

impl AsRef<LineTokenStream> for TokenStreamLineIntoIter {
    fn as_ref(&self) -> &LineTokenStream {
        &self.stream
    }
}

impl AsMut<LineTokenStream> for TokenStreamLineIntoIter {
    fn as_mut(&mut self) -> &mut LineTokenStream {
        &mut self.stream
    }
}
