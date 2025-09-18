use crate::saturating::SaturatingArithmetic;

use super::*;

/// TokenTree-esk line segments
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    /// Vertical caret (│)
    VCaret,
    /// Horizontal caret (─) (amount)
    HCaret(usize),
    /// Down caret (┬)
    HDown,
    /// Up-right caret (╰)
    UpRight,
    /// Vertical-right caret (├)
    VRight,
    /// Vertical-left caret (┤)
    VLeft,
    /// Left Arrow (▶)
    LArrow,
    /// Right Arrow (◀)
    RArrow,
    /// Space (amount)
    Space(usize),
    /// Plain text label
    Literal(String),
    /// Represents a colored label with its associated line token stream, if None, the following tokens are colored
    Styled(AnsiStyle, Option<Box<Token>>),
    /// Reset color
    Reset,
}

macro_rules! impl_consts {
    ($($name:ident = $value:expr);* $(;)?) => {
        $(
            impl Token {
                pub const $name: &str = $value;
            }
            pub const $name: &str = $value;
        )*
    };
}
impl_consts! {
    V_CARET = "│";
    DOWN_RIGHT = "╭";
    UP_RIGHT = "╰";
    DOWN_LEFT = "╮";
    UP_LEFT = "╯";
    V_RIGHT = "├";
    V_LEFT = "┤";
    H_DOWN = "┬";
    H_UP = "┴";
    L_ARROW = "▶";
    R_ARROW = "◀";
}
pub const H_CARET: &str = "─";

macro_rules! boxed_option {
    (!) => {
        None::<Box<Token>>
    };
    ($e:expr) => {
        Some(Box::new($e))
    };
}
use boxed_option as bo;

impl Token {
    #[allow(non_snake_case, dead_code)]
    pub fn new_styled(style: AnsiStyle, inner: Option<Box<Token>>) -> Option<Box<Self>> {
        if let Some(inner_token) = inner {
            match *inner_token {
                Self::Literal(label) => {
                    if label.is_empty() {
                        bo!(Token::Styled(style, None))
                    } else {
                        bo!(Token::Styled(style, Some(Box::new(Token::Literal(label)))))
                    }
                }
                Self::Styled(ansi_style, inner) => {
                    // Dedup if possible, as multiple consecutive styles are often redundant
                    match (style, ansi_style) {
                        (AnsiStyle::RgbColor(_), AnsiStyle::RgbColor(_))
                        | (AnsiStyle::Color(_), AnsiStyle::Color(_)) => {
                            //  No matter the color, the inner one would always overwrite the outer one
                            bo!(Token::Styled(ansi_style, inner))
                        }
                        (AnsiStyle::Style(a), AnsiStyle::Style(b)) if a == b => {
                            // Same style, so the inner would overwrite the outer one
                            bo!(Token::Styled(ansi_style, inner))
                        }
                        (AnsiStyle::Style(a), AnsiStyle::Style(b)) if a != b => {
                            // Styles dont overwrite each other, so we must nest them
                            // But maybe the inner is nested further, so must recursive call Styled
                            bo!(Token::Styled(style, Token::new_styled(ansi_style, inner)))
                        }
                        (AnsiStyle::Reset(a), AnsiStyle::Reset(b))
                            if a == b || a == Resets::All || b == Resets::All =>
                        {
                            // Same reset or one is All, so they would overwrite each other
                            bo!(Token::Styled(ansi_style, inner))
                        }
                        (AnsiStyle::Color(_) | AnsiStyle::RgbColor(_), AnsiStyle::Reset(reset))
                            if matches!(reset, Resets::All | Resets::Color | Resets::FgColor) =>
                        {
                            // Same reset or one is All, so they would overwrite each other
                            Token::new_styled(ansi_style, inner)
                        }
                        (AnsiStyle::Style(inner_style), AnsiStyle::Reset(reset)) => {
                            match (inner_style, reset) {
                                (_, Resets::All | Resets::Style) => {
                                    // Reset All always overwrites any style
                                    Token::new_styled(ansi_style, inner)
                                }
                                (Style::Bold, Resets::Bold)
                                | (Style::Dim, Resets::Dim)
                                | (Style::Italic, Resets::Italic)
                                | (Style::Underline, Resets::Underline)
                                | (Style::Blink, Resets::Blink)
                                | (Style::Inverse, Resets::Inverse)
                                | (Style::Hidden, Resets::Hidden)
                                | (Style::Strikethrough, Resets::Strikethrough) => {
                                    // The style would immediately be overwritten by the reset
                                    Token::new_styled(ansi_style, inner)
                                }
                                _ => {
                                    // Different styles, so we must nest them
                                    // But maybe the inner is nested further, so must recursive call Styled
                                    bo!(Token::Styled(style, Token::new_styled(ansi_style, inner)))
                                }
                            }
                        }
                        _ => {
                            // Different styles, so we must nest them
                            // But maybe the inner is nested further, so must recursive call Styled
                            bo!(Token::Styled(style, Token::new_styled(ansi_style, inner)))
                        }
                    }
                }
                Self::Reset => {
                    // Reset always overwrites any style
                    bo!(!)
                }
                // As anything else cannot be nested, we just create a new styled token
                _ => bo!(Token::Styled(style, Some(inner_token))),
            }
        } else {
            // We still wanna keep the style, even if there is no inner token
            // as this will get displayed (see Display impl)
            bo!(Token::Styled(style, None))
        }
    }

    #[allow(non_snake_case)]
    pub fn SPACE(amount: usize) -> String {
        const SPACE: &str = " ";
        SPACE.repeat(amount)
    }
    #[allow(non_snake_case)]
    pub fn H_CARET(amount: usize) -> String {
        const H_CARET: &str = "─";
        H_CARET.repeat(amount)
    }

    pub fn len(&self) -> usize {
        match self {
            Token::HCaret(amount) => *amount,
            Token::Space(amount) => *amount,
            Token::Literal(label) => label.len(),
            Token::Styled(_, inner) => inner.as_ref().map_or(1, |b| b.len().sat_add(1)),
            _ => 1,
        }
    }

    pub fn is_mergeable(&self) -> bool {
        #[cfg(feature = "merging_tokens")]
        {
            matches!(
                self,
                Token::HCaret(_)
                | Token::Space(_)
                | Token::Literal(_)
                // Reset is mergeable because 2 or more resets are the same as 1 reset
                | Token::Reset
                | Token::Styled(_, _)
            )
        }
        #[cfg(not(feature = "merging_tokens"))]
        {
            matches!(
                self,
                Token::HCaret(_)
                | Token::Space(_)
                | Token::Literal(_)
                // Reset is mergeable because 2 or more resets are the same as 1 reset
                | Token::Reset
            )
        }
    }

    pub fn merge(&mut self, other: Token) -> Option<Token> {
        if !self.is_mergeable() || !other.is_mergeable() {
            return Some(other);
        }
        match (self, &other) {
            (Token::HCaret(a), Token::HCaret(b)) => {
                *a += *b;
                None
            }
            (Token::Space(a), Token::Space(b)) => {
                *a += *b;
                None
            }
            (Token::Literal(a), Token::Literal(b)) => {
                a.push_str(&b);
                None
            }
            (Token::Reset, Token::Reset) => {
                // No need to do anything, as 2 or more resets are the same as 1 reset
                None
            }
            (
                Token::Styled(AnsiStyle::Reset(Resets::All), _),
                // Its important that the other is also a reset and empty
                Token::Styled(AnsiStyle::Reset(_), None),
            ) => {
                // No need to do anything, as 2 or more style resets are the same as 1 style reset
                None
            }
            (
                Token::Styled(AnsiStyle::Reset(Resets::All), _),
                // Its important that the other is also a reset and its label is empty (although that should never happen)
                Token::Styled(AnsiStyle::Reset(_), Some(box Token::Literal(l))),
            ) if l.is_empty() => {
                // No need to do anything, as 2 or more style resets are the same as 1 style reset
                None
            }
            (
                Token::Styled(AnsiStyle::Reset(Resets::All), _),
                // Its important that the other is also a reset and its label is empty (although that should never happen)
                Token::Styled(
                    AnsiStyle::Reset(_),
                    Some(box (Token::Reset | Token::Styled(AnsiStyle::Reset(_), None))),
                ),
            ) => {
                // No need to do anything, as 2 or more style resets are the same as 1 style reset
                None
            }
            #[cfg(feature = "merging_tokens")]
            (
                Token::Styled(AnsiStyle::Color(color_a), Some(box inner_a)),
                // Its important that the other is also a reset and empty
                Token::Styled(AnsiStyle::Color(color_b), Some(box inner_b)),
            ) => {
                if color_a == color_b {
                    // Same color, so the inner would overwrite the outer one
                    if let Some(_) = inner_a.merge(inner_b.clone()) {
                        // Not able to merge
                        Some(other)
                    } else {
                        None
                    }
                } else {
                    // Different colors, so we must not merge
                    Some(other)
                }
                // No need to do anything, as 2 or more style resets are the same as 1 style reset
            }
            #[cfg(feature = "merging_tokens")]
            (
                Token::Styled(AnsiStyle::RgbColor(color_a), Some(box inner_a)),
                // Its important that the other is also a reset and empty
                Token::Styled(AnsiStyle::RgbColor(color_b), Some(box inner_b)),
            ) => {
                if color_a == color_b {
                    // Same color, so the inner would overwrite the outer one
                    if let Some(_) = inner_a.merge(inner_b.clone()) {
                        // Not able to merge
                        Some(other)
                    } else {
                        None
                    }
                } else {
                    // Different colors, so we must not merge
                    Some(other)
                }
                // No need to do anything, as 2 or more style resets are the same as 1 style reset
            }
            _ => Some(other),
        }
    }

    /// Used to parse a label that may or may not be stylized
    /// thus it either returns Self::Label or Self::Styled
    pub fn parse_label_all(s: String) -> Vec<Token> {
        let mut result = Vec::new();

        let mut remaining = Some(s);

        // Use the "parse_label" function in a loop to parse all labels
        while let Some(rem) = remaining {
            if let Some((token, rem)) = Self::parse_label(rem) {
                result.push(token);
                remaining = rem;
            } else {
                break;
            }
        }

        result
    }
    /// Used to parse a label that may or may not be stylized
    /// thus it either returns Self::Label or Self::Styled
    pub fn parse_label(s: String) -> Option<(Self, Option<String>)> {
        let haystack = s;
        if haystack.is_empty() {
            return None;
        }

        const NEELDE: &str = "\u{1b}[";
        // Find all occurrences of the ANSI escape character

        if let Some(escape_pos) = stringzilla::sz::find(&haystack, NEELDE) {
            if escape_pos > 0 {
                // There is a normal label before the escape sequence
                // As we are only returning one token, we must return here, and return the remaining string
                return Some((
                    Token::Literal(haystack[..escape_pos].to_string()),
                    Some(haystack[escape_pos..].to_string()),
                ));
            }
            // Try to parse the escape sequence and the following label
            // As the escape sequence is at the start, we can just parse the whole string
            if let Ok((style, rem)) = AnsiStyle::try_from_str(&haystack[escape_pos..]) {
                // We have a valid ANSI style
                if let Some(rem) = rem {
                    // Now try to parse the remaining string as a token
                    if let Some((parsed_tkn, parsed_rem)) = Token::parse_from_str(rem) {
                        // Check if there is a remaining string after parsing the token (even though this is redundant)
                        if matches!(parsed_rem.as_ref(), Some(inner) if !inner.is_empty()) {
                            // If there is a remaining string, we must return it
                            return Some((
                                Token::Styled(style, Some(Box::new(parsed_tkn))),
                                parsed_rem,
                            ));
                        }
                        #[cfg(feature = "merging_tokens")]
                        {
                            if let Some(box valid_style) =
                                Token::new_styled(style, Some(Box::new(parsed_tkn)))
                            {
                                return Some((valid_style, None));
                            }
                            return None;
                        }
                        #[cfg(not(feature = "merging_tokens"))]
                        return Some((Token::Styled(style, Some(Box::new(parsed_tkn))), None));
                    }
                    // The remaining string is empty or invalid, so just return the styled token
                    return Some((Token::Styled(style, None), None));
                }
                // No remaining string, so just return the styled token
                return Some((Token::Styled(style, None), None));
            }
            // Invalid ANSI sequence
            // We'll just treat it as a normal label
            return Some((Token::Literal(haystack), None));
        }
        Some((Token::Literal(haystack), None))
    }

    pub(crate) fn parse_from_str<A: AsRef<str>>(s: A) -> Option<(Self, Option<String>)> {
        let s = s.as_ref();
        if s.is_empty() {
            return None;
        }
        let mut chars = s.chars();
        let first_char = chars.next().unwrap();
        let token = match first_char {
            '\u{1b}' => {
                if let Ok((style, rem)) = AnsiStyle::try_from_str(s) {
                    // We have a valid ANSI style
                    if let Some(rem) = rem
                        && !rem.is_empty()
                    {
                        // Now try to parse the remaining string as a token
                        if let Some((parsed_tkn, parsed_rem)) = Token::parse_from_str(rem) {
                            // Check if there is a remaining string after parsing the token (even though this is redundant)
                            if matches!(parsed_rem.as_ref(), Some(inner) if !inner.is_empty()) {
                                // If there is a remaining string, we must return it
                                return Some((
                                    Token::Styled(style, Some(Box::new(parsed_tkn))),
                                    parsed_rem,
                                ));
                            }
                            #[cfg(feature = "merging_tokens")]
                            {
                                if let Some(box valid_style) =
                                    Token::new_styled(style, Some(Box::new(parsed_tkn)))
                                {
                                    return Some((valid_style, None));
                                }
                                return None;
                            }
                            #[cfg(not(feature = "merging_tokens"))]
                            return Some((Token::Styled(style, Some(Box::new(parsed_tkn))), None));
                        }
                        // The remaining string is empty or invalid, so just return the styled token
                        return Some((Token::Styled(style, None), None));
                    }
                    // No remaining string, so just return the styled token
                    return Some((Token::Styled(style, None), None));
                }
                // Invalid ANSI sequence
                // We'll just treat it as a normal label
                return Some((Token::Literal(s.to_string()), None));
            }
            '│' => Token::VCaret,
            '─' => Token::HCaret(chars.take_while_ref(|&c| c == '─').count().sat_add(1)),
            '┬' => Token::HDown,
            '╰' => Token::UpRight,
            '├' => Token::VRight,
            '┤' => Token::VLeft,
            '▶' => Token::LArrow,
            ' ' => Token::Space(chars.take_while_ref(|&c| c == ' ').count().sat_add(1)),
            _ => {
                let label = s.to_string();
                return Self::parse_label(label);
            }
        };
        let remaining = chars.collect::<String>();
        let remaining = if remaining.is_empty() {
            None
        } else {
            Some(remaining)
        };
        Some((token, remaining))
    }

    #[cfg(feature = "merging_tokens")]
    pub fn fmt_context(&self, prev: &Self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (prev, self) {
            (Self::Reset, Self::Reset) => {
                // No need to do anything, as 2 or more resets are the same as 1 reset and one was already printed as the "prev" token must've been preceded by something else or its just resets, in which case we dont print anything
                Ok(())
            }
            (Self::Styled(ansi_style_a, token_a), Self::Styled(ansi_style_b, token_b)) => {
                if f.alternate() {
                    // No matter if the styles would've matched, the previous one has already been reset
                    if matches!(token_b.as_ref(), Some(&box Token::Reset)) {
                        // If there is no inner token, we dont want to print anything, as the style would be reset immediately after "nothing"
                        return Ok(());
                    }
                    Display::fmt(&self, f)
                } else {
                    if ansi_style_a == ansi_style_b {
                        // Same style, so the inner would overwrite the outer one
                        if let Some(box inner_token) = token_b {
                            inner_token.fmt_context(token_a.as_deref().unwrap_or(&Token::Reset), f)
                        } else {
                            // No inner token, so just print the style
                            Display::fmt(&self, f)
                        }
                    } else {
                        // Different styles, so we must print the current one as-is
                        Display::fmt(&self, f)
                    }
                }
            }
            (_, Self::Reset) => {
                if f.alternate() {
                    // If alternate is set, we dont print resets after anything, as these would've been handled already
                    Ok(())
                } else {
                    // We must print the reset, as it might be needed for the following tokens
                    Display::fmt(&self, f)
                }
            }
            _ => {
                // We must print the token as-is, as it might be needed for the following tokens
                Display::fmt(&self, f)
            }
        }
    }
    #[cfg(feature = "merging_tokens")]
    pub fn format_context(&self, prev: &Self, alt: bool) -> String {
        use ::std::fmt::FormattingOptions;

        let mut output = String::new();
        {
            let mut opts = FormattingOptions::default();
            opts.alternate(alt);
            let mut formatter = std::fmt::Formatter::new(&mut output, opts);
            let _ = self.fmt_context(prev, &mut formatter);
        }
        output
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::VCaret => write!(f, "{}", Self::V_CARET),
            Token::HCaret(amount) => write!(f, "{}", Self::H_CARET(*amount)),
            Token::HDown => write!(f, "{}", Self::H_DOWN),
            Token::UpRight => write!(f, "{}", Self::UP_RIGHT),
            Token::VRight => write!(f, "{}", Self::V_RIGHT),
            Token::VLeft => write!(f, "{}", Self::V_LEFT),
            Token::LArrow => write!(f, "{}", Self::L_ARROW),
            Token::RArrow => write!(f, "{}", Self::R_ARROW),
            Token::Space(amount) => write!(f, "{}", Self::SPACE(*amount)),
            Token::Literal(label) => write!(f, "{}", label),
            Token::Styled(style, token) => {
                write!(f, "{}", style)?;
                if let Some(token) = token {
                    write!(f, "{}", token)?;
                    #[cfg(feature = "merging_tokens")]
                    if f.alternate() {
                        write!(f, "{}", Self::Reset) // Reset color
                    } else {
                        Ok(())
                    }
                    #[cfg(not(feature = "merging_tokens"))]
                    write!(f, "{}", Self::Reset) // Reset color
                } else {
                    // Dont reset color, as we want the following tokens to be colored
                    Ok(())
                }
            }
            Token::Reset => write!(f, "{RESET}"), // Reset color
        }
    }
}

impl IntoIterator for Token {
    type Item = Token;
    type IntoIter = std::iter::Once<Token>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}
