use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineTokenStream {
    /// Outer vec: lines
    /// Inner vec: tokens in line
    tokens: Vec<TokenStream>,
}
impl Display for LineTokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tokens.iter().try_for_each(|line| {
            line.as_ref()
                .iter()
                .try_for_each(|token| write!(f, "{}", token))?;
            writeln!(f)
        })?;
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
    pub fn iter(&self) -> std::slice::Iter<'_, TokenStream> {
        self.tokens.iter()
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
    pub fn new() -> Self {
        Self { tokens: vec![] }
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stream = LineTokenStream::new();
        stream.push_str(s);
        Ok(stream)
    }
}
