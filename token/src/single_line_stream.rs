use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Into, derive_more::IntoIterator)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
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

    pub fn on_color(&mut self, color: Color) {
        if let Some(first) = self.tokens.first_mut() {
            *first = Token::Styled(color, Some(Box::new(first.clone())));
            self.push(Token::Reset);
        }
    }

    pub fn push<I: Into<Token>>(&mut self, item: I) {
        if let Some(last) = self.tokens.last_mut() {
            // Try to merge with the last token
            if let Some(merged) = last.merge(item.into()) {
                // Push the non-mergeable token
                self.tokens.push(merged);
            }
            // Merged. As "merge" takes a mutable reference, we dont need to do anything
        } else {
            // No last token, just push
            self.tokens.push(item.into());
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
                        self.push(last_tkn);
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
            self.push(tkn);
        }
        true
    }

    pub fn extend<I: IntoIterator<Item = Token>>(&mut self, iter: I) {
        // Try to merge the first item with the last token
        let mut iter = iter.into_iter();
        if let Some(mut outer) = iter.next() {
            for inner in iter {
                if let Some(merged) = outer.merge(inner) {
                    // Push the non-mergeable token
                    self.push(outer);
                    outer = merged;
                }
                // Merged. As "merge" takes a mutable reference, we dont need to do anything
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
        for token in &self.tokens {
            write!(f, "{}", token)?;
        }
        Ok(())
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
        if stream.push_str(s) {
            Ok(stream)
        } else {
            Err(())
        }
    }
}

impl<A: AsRef<str>> From<A> for TokenStream {
    fn from(value: A) -> Self {
        let mut stream = TokenStream::new();
        stream.push_str(value.as_ref());
        stream
    }
}
