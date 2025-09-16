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
    Label(String),
    /// Represents a colored label with its associated line token stream, if None, the following tokens are colored
    Styled(Color, Option<Box<Token>>),
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

impl Token {
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
            Token::Label(label) => label.len(),
            Token::Styled(_, inner) => inner.as_ref().map_or(1, |b| b.len() + 1),
            _ => 1,
        }
    }

    pub fn is_mergeable(&self) -> bool {
        matches!(
            self,
            Token::HCaret(_) | Token::Space(_) | Token::Label(_) | Token::Reset // Reset is mergeable because 2 or more resets are the same as 1 reset
        )
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
            (Token::Label(a), Token::Label(b)) => {
                a.push_str(&b);
                None
            }
            (Token::Reset, Token::Reset) => None, // No need to do anything, as 2 or more resets are the same as 1 reset
            _ => Some(other),
        }
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
                // ANSI escape sequence for color
                let mut ansi_sequence = String::new();
                ansi_sequence.push(first_char);
                while let Some(c) = chars.next() {
                    ansi_sequence.push(c);
                    if c == 'm' {
                        break;
                    }
                }
                if ansi_sequence == "\u{1b}[0m" && ansi_sequence.len() == 4 {
                    // Reset code
                    return Some((Token::Reset, Some(chars.collect())));
                }

                // We dont wanna use regex here because its a heavy dependency and runtime for something this small
                if ansi_sequence.chars().filter(|c| *c == ';').count() == 4 {
                    // RGB ANSI color code
                    ansi_sequence = ansi_sequence.split_off(7); // Remove the "\x1b[38;2;
                } else {
                    // Simple ANSI color code
                    ansi_sequence = ansi_sequence.split_off(2); // Remove the "\x1b[
                }
                let _ = ansi_sequence.split_off(ansi_sequence.len() - 1); // Remove the trailing "m"
                let parts = ansi_sequence.split(';'); // Split by ';'

                // Now we only have the numbers left
                let mut parts = parts.array_chunks();
                let color = if let Some::<[&str; 3]>(parts) = parts.next() {
                    let r = parts[0].parse::<u8>().unwrap_or(255);
                    let g = parts[1].parse::<u8>().unwrap_or(255);
                    let b = parts[2].parse::<u8>().unwrap_or(255);
                    Color::new(r, g, b)
                } else {
                    // We have a non-rgb ANSI color sequence
                    // so its just a single color code
                    if let Some(mut parts) = parts.into_remainder() {
                        if let Some(code) = parts.next() {
                            if parts.next().is_some() {
                                return None; // Invalid color code
                            }
                            if let Ok(code) = code.parse::<u8>() {
                                if let Some(color) = Color::from_ansi_code(code) {
                                    color
                                } else {
                                    return None; // Invalid color code
                                }
                            } else {
                                return None; // Invalid color code
                            }
                        } else {
                            return None; // Invalid color code
                        }
                    } else {
                        return None; // Invalid color code
                    }
                };

                // Parse the next token after the ANSI sequence
                if let Some((next_token, rem)) = Token::parse_from_str(chars.as_str()) {
                    return Some((Token::Styled(color, Some(Box::new(next_token))), rem));
                }
                return Some((Token::Styled(color, None), None));
            }
            '│' => Token::VCaret,
            '─' => Token::HCaret(chars.take_while_ref(|&c| c == '─').count() + 1),
            '┬' => Token::HDown,
            '╰' => Token::UpRight,
            '├' => Token::VRight,
            '┤' => Token::VLeft,
            '▶' => Token::LArrow,
            ' ' => Token::Space(chars.take_while_ref(|&c| c == ' ').count() + 1),
            _ => {
                let label = s.to_string();
                return Some((Token::Label(label), None));
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
            Token::Label(label) => write!(f, "{}", label),
            Token::Styled(color, token) => {
                write!(f, "{}", color.to_ansi_fg())?;
                if let Some(token) = token {
                    write!(f, "{}", token)?;
                    write!(f, "{RESET}") // Reset color
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
