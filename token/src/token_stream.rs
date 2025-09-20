use ::std::{borrow::Borrow, slice::SliceIndex};

use super::*;

#[derive(Clone, PartialEq, Eq, Hash, derive_more::Into, derive_more::IntoIterator)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
}

impl Debug for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "alt_debug")]
        {
            if f.alternate() {
                // Pretty print
                write!(f, "TokenStream [\n")?;
                let mut stream: String = String::new();
                if self.tokens.is_empty() {
                    write!(f, "<empty>")?;
                    return Ok(());
                }
                let mut last = self.tokens.first().unwrap();
                stream.push_str(&format!("{last:#}"));
                for token in self.tokens.iter().skip(1) {
                    stream.push_str(&token.format_context(last, true));
                    last = token;
                }
                write!(f, "{stream:#?}")?;
                write!(
                    f,
                    "\n] {{literal len: {}, len: {}}}",
                    self.lit_len(),
                    self.len()
                )
            } else {
                // Default debug implementation
                f.debug_struct("TokenStream")
                    .field("tokens", &self.tokens)
                    .finish()
            }
        }
        #[cfg(not(feature = "alt_debug"))]
        {
            // Default debug implementation
            f.debug_struct("TokenStream")
                .field("tokens", &self.tokens)
                .finish()
        }
    }
}

impl TokenStream {
    delegate::delegate! {
        to self.tokens {
            pub fn pop(&mut self) -> Option<Token>;
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
            pub fn iter(&self) -> std::slice::Iter<'_, Token>;
            pub fn last(&self) -> Option<&Token>;
            pub fn last_mut(&mut self) -> Option<&mut Token>;
        }
    }

    pub fn new() -> Self {
        Self { tokens: vec![] }
    }

    pub fn lit_len(&self) -> usize {
        self.tokens.iter().map(|tkn| tkn.len()).sum()
    }

    pub fn on_color<I: Into<AnsiStyle>>(&mut self, style: I) {
        if let Some(first) = self.tokens.first_mut() {
            *first = Token::Styled(style.into(), Some(Box::new(first.clone())));
            self.push_iter(Token::Reset);
        }
    }
    pub fn with_color<I: Into<AnsiStyle>>(mut self, style: I) -> Self {
        self.on_color(style);
        self
    }

    /// Get a mutable reference to the token at the given index, counting from the end.
    pub fn r_get_mut(&mut self, index: usize) -> Option<&mut Token> {
        let len = self.tokens.len();
        if index >= len {
            None
        } else {
            self.tokens.get_mut(len - 1 - index)
        }
    }

    pub fn push<T: Into<Token>>(&mut self, item: T) {
        // Try to merge the item with the last token
        let item: Token = item.into();
        if let Some(last) = self.tokens.last_mut() {
            if let Some(unmerged) = last.merge(item) {
                // Not merged
                self.tokens.push(unmerged);
            }
            // Merged
        } else {
            self.tokens.push(item);
        }
    }

    pub fn push_iter<T: Into<Token>, I: IntoIterator<Item = T>>(&mut self, item: I) {
        // Try to merge the iterator via fold
        let result = item
            .into_iter()
            // Use the last token as the initial accumulator
            .fold(self.tokens.pop(), |acc: Option<Token>, item| {
                let item: Token = item.into();
                if let Some(mut acc) = acc {
                    if let Some(unmerged) = acc.merge(item) {
                        // Not able to merge
                        self.tokens.push(acc);
                        Some(unmerged)
                    } else {
                        // Merged. As "merge" takes a mutable reference, return the merged accumulator
                        Some(acc)
                    }
                } else {
                    // No accumulator yet, so just return the item as the new accumulator
                    Some(item)
                }
            });
        if let Some(tkn) = result {
            self.tokens.push(tkn);
        }
    }

    pub fn push_str<A: AsRef<str>>(&mut self, s: A) -> bool {
        let line = s.as_ref().replace("\r", "");
        if line.is_empty() {
            return false;
        }
        if line.contains('\n') {
            return false;
        }

        let mut line = line.to_string();
        let mut last_token: Option<Token> = None;
        loop {
            if let Some((token, rem)) = Token::parse_from_str(&line) {
                if let Some(mut last_tkn) = last_token.take() {
                    // Try to merge with the last token
                    if let Some(tkn) = last_tkn.merge(token) {
                        // Push the non-mergeable token
                        self.push_iter(last_tkn);
                        // Set the last_token to the new token
                        last_token = Some(tkn);
                    }
                    // Merged. As "merge" takes a mutable reference, we dont need to do anything
                } else {
                    last_token = Some(token);
                }
                if let Some(rem) = rem {
                    line = rem;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if let Some(tkn) = last_token {
            self.push_iter(tkn);
        }
        true
    }

    pub fn extend<I: IntoIterator<Item = Token>>(&mut self, iter: I) {
        // Try to merge the first item with the last token
        let mut iter = iter.into_iter();
        if let Some(mut outer) = iter.next() {
            let mut outer_needs_push = true;
            for inner in iter {
                if let Some(merged) = outer.merge(inner) {
                    // Push the non-mergeable token
                    self.push_iter(outer);
                    outer = merged;
                    outer_needs_push = true;
                } else {
                    outer_needs_push = false;
                }
                // Merged. As "merge" takes a mutable reference, we dont need to do anything
            }
            if outer_needs_push {
                self.push_iter(outer);
            }
        }
    }

    pub fn insert<T: Into<Token>>(&mut self, index: usize, item: T) {
        let mut item = item.into();
        if index >= self.tokens.len() {
            self.push_iter(item);
        } else {
            if let Some(last) = self.tokens.get_mut(index.saturating_sub(1)) {
                if let Some(unmerged) = last.merge(item) {
                    // Not merged
                    self.tokens.insert(index, unmerged);
                }
                // Merged
            } else if let Some(next) = self.tokens.get_mut(index) {
                if let Some(unmerged) = item.merge(next.clone()) {
                    // Not merged
                    self.tokens.insert(index, unmerged);
                }
                // Merged
            } else {
                self.tokens.insert(index, item);
            }
        }
    }
}
impl AsRef<[Token]> for TokenStream {
    fn as_ref(&self) -> &[Token] {
        &self.tokens
    }
}
impl AsMut<[Token]> for TokenStream {
    fn as_mut(&mut self) -> &mut [Token] {
        &mut self.tokens
    }
}
impl Display for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tokenbuffer = TokenBuffer::new(&self.tokens);
        Display::fmt(&tokenbuffer, f)
    }
}

impl From<&TokenStream> for TokenStream {
    fn from(value: &TokenStream) -> Self {
        value.clone()
    }
}

impl FromStr for TokenStream {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stream = TokenStream::new();
        stream.push_str(s).then_res(stream, ())
    }
}

impl<A: AsRef<str>> From<A> for TokenStream {
    fn from(value: A) -> Self {
        let mut stream = TokenStream::new();
        stream.push_str(value.as_ref());
        stream
    }
}

impl FromIterator<Token> for TokenStream {
    fn from_iter<T: IntoIterator<Item = Token>>(iter: T) -> Self {
        let mut stream = TokenStream::new();
        stream.extend(iter);
        stream
    }
}

/// The borrowed variant of TokenStream
#[derive(Debug, Clone)]
pub struct TokenBuffer<'a> {
    pub buffer: &'a [Token],
}
impl<'a> TokenBuffer<'a> {
    pub fn new<A: AsRef<[Token]> + ?Sized>(buffer: &'a A) -> Self {
        Self {
            buffer: buffer.as_ref(),
        }
    }

    pub fn get<I: SliceIndex<[token::Token], Output = [Token]>>(
        &'a self,
        index: I,
    ) -> Option<TokenBuffer<'a>> {
        self.buffer
            .get(index)
            .map(|slice| TokenBuffer { buffer: slice })
    }
    pub fn to_owned(&self) -> TokenStream {
        TokenStream {
            tokens: self.buffer.to_vec(),
        }
    }
}
impl<'a> Display for TokenBuffer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.buffer.is_empty() {
            return Ok(());
        }
        #[cfg(feature = "merging_tokens")]
        {
            // We know its not empty, so unwrap is safe
            let mut prev_token = self.buffer.first().unwrap();
            if f.alternate() {
                write!(f, "{prev_token:#}")?;
            } else {
                write!(f, "{prev_token}")?;
            }
            for token in self.buffer.iter().skip(1) {
                token.fmt_context(prev_token, f)?;
                prev_token = token;
            }
        }
        #[cfg(not(feature = "merging_tokens"))]
        {
            for token in self.buffer {
                if f.alternate() {
                    write!(f, "{token:#}")?;
                } else {
                    write!(f, "{token}")?;
                }
            }
        }
        Ok(())
    }
}
impl AsRef<[Token]> for TokenBuffer<'_> {
    fn as_ref(&self) -> &[Token] {
        self.buffer
    }
}
impl<'a, I: AsRef<[Token]> + ?Sized + 'a> From<&'a I> for TokenBuffer<'a> {
    fn from(value: &'a I) -> Self {
        Self::new(value)
    }
}

impl Borrow<[Token]> for TokenBuffer<'_> {
    fn borrow(&self) -> &[Token] {
        self.buffer
    }
}
