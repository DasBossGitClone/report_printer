#![deny(dead_code, unused)]

use ::std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};

pub use misc_extensions::consts::colors::{
    BG_BLACK, BG_BLUE, BG_CYAN, BG_DARKBLACK, BG_DARKBLUE, BG_DARKCYAN, BG_DARKGREEN,
    BG_DARKMAGENTA, BG_DARKRED, BG_DARKWHITE, BG_DARKYELLOW, BG_DEFAULT, BG_GREEN, BG_MAGENTA,
    BG_RED, BG_RESET, BG_WHITE, BG_YELLOW, BLACK, BLINK, BLINK_RESET, BLUE, BOLD, BOLD_RESET,
    COLOR_RESET, CYAN, DARKBLACK, DARKBLUE, DARKCYAN, DARKGREEN, DARKMAGENTA, DARKRED, DARKWHITE,
    DARKYELLOW, DEFAULT, DIM, DIM_RESET, FG_RESET, GREEN, HIDDEN, HIDDEN_RESET, INVERSE,
    INVERSE_RESET, ITALIC, ITALIC_RESET, MAGENTA, RED, RESET, STRIKETHROUGH, STRIKETHROUGH_RESET,
    STYLE_RESET, UNDERLINE, UNDERLINE_RESET, WHITE, YELLOW,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AnsiStyle {
    Color(Color),
    Style(Style),
    RgbColor(RgbColor),
}
impl AnsiStyle {
    pub const BLACK: Self = Self::Color(Color::BLACK);
    pub const RED: Self = Self::Color(Color::RED);
    pub const GREEN: Self = Self::Color(Color::GREEN);
    pub const YELLOW: Self = Self::Color(Color::YELLOW);
    pub const BLUE: Self = Self::Color(Color::BLUE);
    pub const MAGENTA: Self = Self::Color(Color::MAGENTA);
    pub const CYAN: Self = Self::Color(Color::CYAN);
    pub const WHITE: Self = Self::Color(Color::WHITE);
    pub const BRIGHT_BLACK: Self = Self::Color(Color::BRIGHT_BLACK);
    pub const BRIGHT_RED: Self = Self::Color(Color::BRIGHT_RED);
    pub const BRIGHT_GREEN: Self = Self::Color(Color::BRIGHT_GREEN);
    pub const BRIGHT_YELLOW: Self = Self::Color(Color::BRIGHT_YELLOW);
    pub const BRIGHT_BLUE: Self = Self::Color(Color::BRIGHT_BLUE);
    pub const BRIGHT_MAGENTA: Self = Self::Color(Color::BRIGHT_MAGENTA);
    pub const BRIGHT_CYAN: Self = Self::Color(Color::BRIGHT_CYAN);
    pub const BRIGHT_WHITE: Self = Self::Color(Color::BRIGHT_WHITE);
    pub const BOLD: Self = Self::Style(Style::BOLD);
    pub const DIM: Self = Self::Style(Style::DIM);
    pub const ITALIC: Self = Self::Style(Style::ITALIC);
    pub const UNDERLINE: Self = Self::Style(Style::UNDERLINE);
    pub const BLINK: Self = Self::Style(Style::BLINK);
    pub const INVERSE: Self = Self::Style(Style::INVERSE);
    pub const HIDDEN: Self = Self::Style(Style::HIDDEN);
    pub const STRIKETHROUGH: Self = Self::Style(Style::STRIKETHROUGH);

    pub fn to_ansi_escape_sequence(&self) -> String {
        match self {
            AnsiStyle::RgbColor(rgb) => rgb.to_string(),
            AnsiStyle::Color(color) => color.to_string(),
            AnsiStyle::Style(style) => style.to_string(),
        }
    }

    pub fn from_ansi_code(code: u8) -> Option<Self> {
        if let Some(color) = Color::from_ansi_code(code) {
            Some(AnsiStyle::Color(color))
        } else if let Some(rgb) = RgbColor::from_ansi_code(code) {
            Some(AnsiStyle::RgbColor(rgb))
        } else {
            match code {
                0 => Some(AnsiStyle::Style(Style::RESET)),
                1 => Some(AnsiStyle::Style(Style::BOLD)),
                2 => Some(AnsiStyle::Style(Style::DIM)),
                3 => Some(AnsiStyle::Style(Style::ITALIC)),
                4 => Some(AnsiStyle::Style(Style::UNDERLINE)),
                5 => Some(AnsiStyle::Style(Style::BLINK)),
                7 => Some(AnsiStyle::Style(Style::INVERSE)),
                8 => Some(AnsiStyle::Style(Style::HIDDEN)),
                9 => Some(AnsiStyle::Style(Style::STRIKETHROUGH)),
                _ => None,
            }
        }
    }

    pub fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        AnsiStyle::RgbColor(RgbColor::new(r, g, b))
    }
    pub fn new_color<I: Into<Color>>(color: I) -> Self {
        AnsiStyle::Color(color.into())
    }
    pub fn new_style<I: Into<Style>>(style: I) -> Self {
        AnsiStyle::Style(style.into())
    }

    /// Same as the "FromStr" implementation, but returns all trailing unparsed text
    pub fn try_from_str<A: AsRef<str>>(s: A) -> Result<(Self, Option<String>), ()> {
        let input = s.as_ref();
        if input.is_empty() {
            return Err(());
        }
        let input = input.strip_prefix("\u{1b}[").ok_or(())?;
        if let Some(pos) = input.find('m') {
            let (code_str, rest) = input.split_at(pos);
            if let Ok(code) = code_str.parse::<u8>() {
                if let Some(style) = AnsiStyle::from_ansi_code(code) {
                    return Ok((style, Some(rest[1..].to_string()))); // +1 to skip the 'm'
                }
            } else if let Some(pos) = code_str.find("38;2;") {
                let rgb_part = &code_str[pos + 5..]; // Skip "38;2;"
                let mut parts = rgb_part.split(';');
                if let (Some(r), Some(g), Some(b)) = (parts.next(), parts.next(), parts.next()) {
                    if let (Ok(r), Ok(g), Ok(b)) =
                        (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>())
                    {
                        return Ok((
                            AnsiStyle::RgbColor(RgbColor::new(r, g, b)),
                            Some(rest[code_str.len() + 1..].to_string()),
                        )); // +1 to skip the 'm'
                    }
                }
            }
            Err(())
        } else {
            Err(())
        }
    }

    pub fn with_color<D: Display>(&self, s: D) -> String {
        format!("{}{}{}", self.to_ansi_escape_sequence(), s, RESET)
    }
}

impl AsRef<AnsiStyle> for AnsiStyle {
    fn as_ref(&self) -> &AnsiStyle {
        self
    }
}

impl FromStr for AnsiStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // First identify the escape sequence
        let s = s.strip_prefix("\u{1b}[").ok_or(())?;
        let s = s.strip_suffix('m').ok_or(())?;
        if let Ok(code) = s.parse::<u8>() {
            if let Some(style) = AnsiStyle::from_ansi_code(code) {
                return Ok(style);
            }
        } else {
            // Try to parse as RGB, should be in the format "38;2;<r>;<g>;<b>"
            let s = s.strip_prefix("38;2;").ok_or(())?;
            let mut parts = s.split(';');
            if let (Some(r), Some(g), Some(b)) = (parts.next(), parts.next(), parts.next()) {
                if let (Ok(r), Ok(g), Ok(b)) = (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>()) {
                    return Ok(AnsiStyle::RgbColor(RgbColor::new(r, g, b)));
                }
            }
        }

        Err(())
    }
}

impl<I: Into<Color>> From<I> for AnsiStyle {
    fn from(value: I) -> Self {
        AnsiStyle::Color(value.into())
    }
}

impl Display for AnsiStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ansi_escape_sequence())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Style {
    Reset = 0,
    Bold = 1,
    Dim = 2,
    Italic = 3,
    Underline = 4,
    Blink = 5,
    Inverse = 7,
    Hidden = 8,
    Strikethrough = 9,
}
impl Style {
    pub const RESET: Self = Self::Reset;
    pub const BOLD: Self = Self::Bold;
    pub const DIM: Self = Self::Dim;
    pub const ITALIC: Self = Self::Italic;
    pub const UNDERLINE: Self = Self::Underline;
    pub const BLINK: Self = Self::Blink;
    pub const INVERSE: Self = Self::Inverse;
    pub const HIDDEN: Self = Self::Hidden;
    pub const STRIKETHROUGH: Self = Self::Strikethrough;
}
impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Style::Reset => write!(f, "{}", STYLE_RESET),
            Style::Bold => write!(f, "{}", BOLD),
            Style::Dim => write!(f, "{}", DIM),
            Style::Italic => write!(f, "{}", ITALIC),
            Style::Underline => write!(f, "{}", UNDERLINE),
            Style::Blink => write!(f, "{}", BLINK),
            Style::Inverse => write!(f, "{}", INVERSE),
            Style::Hidden => write!(f, "{}", HIDDEN),
            Style::Strikethrough => write!(f, "{}", STRIKETHROUGH),
        }
    }
}

impl AsRef<Style> for Style {
    fn as_ref(&self) -> &Style {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ColorPalette {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub color: ColorPalette,
    pub bright: bool,
}
impl Color {
    pub const BLACK: Self = Self::new(ColorPalette::Black, false);
    pub const RED: Self = Self::new(ColorPalette::Red, false);
    pub const GREEN: Self = Self::new(ColorPalette::Green, false);
    pub const YELLOW: Self = Self::new(ColorPalette::Yellow, false);
    pub const BLUE: Self = Self::new(ColorPalette::Blue, false);
    pub const MAGENTA: Self = Self::new(ColorPalette::Magenta, false);
    pub const CYAN: Self = Self::new(ColorPalette::Cyan, false);
    pub const WHITE: Self = Self::new(ColorPalette::White, false);
    pub const BRIGHT_BLACK: Self = Self::new(ColorPalette::Black, true);
    pub const BRIGHT_RED: Self = Self::new(ColorPalette::Red, true);
    pub const BRIGHT_GREEN: Self = Self::new(ColorPalette::Green, true);
    pub const BRIGHT_YELLOW: Self = Self::new(ColorPalette::Yellow, true);
    pub const BRIGHT_BLUE: Self = Self::new(ColorPalette::Blue, true);
    pub const BRIGHT_MAGENTA: Self = Self::new(ColorPalette::Magenta, true);
    pub const BRIGHT_CYAN: Self = Self::new(ColorPalette::Cyan, true);
    pub const BRIGHT_WHITE: Self = Self::new(ColorPalette::White, true);

    pub const fn new(color: ColorPalette, bright: bool) -> Self {
        Self { color, bright }
    }
    pub fn to_ansi_escape_sequence(&self) -> String {
        let code = self.color as u8 + if self.bright { 60 } else { 0 };
        format!("\u{1b}[{}m", code)
    }
    pub fn from_ansi_code(code: u8) -> Option<Self> {
        Some(match code {
            30 => Self::new(ColorPalette::Black, false),
            31 => Self::new(ColorPalette::Red, false),
            32 => Self::new(ColorPalette::Green, false),
            33 => Self::new(ColorPalette::Yellow, false),
            34 => Self::new(ColorPalette::Blue, false),
            35 => Self::new(ColorPalette::Magenta, false),
            36 => Self::new(ColorPalette::Cyan, false),
            37 => Self::new(ColorPalette::White, false),
            90 => Self::new(ColorPalette::Black, true),
            91 => Self::new(ColorPalette::Red, true),
            92 => Self::new(ColorPalette::Green, true),
            93 => Self::new(ColorPalette::Yellow, true),
            94 => Self::new(ColorPalette::Blue, true),
            95 => Self::new(ColorPalette::Magenta, true),
            96 => Self::new(ColorPalette::Cyan, true),
            97 => Self::new(ColorPalette::White, true),
            _ => return None,
        })
    }
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ansi_escape_sequence())
    }
}
impl TryFrom<RgbColor> for Color {
    type Error = ();

    fn try_from(value: RgbColor) -> Result<Self, Self::Error> {
        match (value.r, value.g, value.b) {
            (12, 12, 12) => Ok(Self::new(ColorPalette::Black, false)),
            (197, 15, 12) => Ok(Self::new(ColorPalette::Red, false)),
            (19, 161, 14) => Ok(Self::new(ColorPalette::Green, false)),
            (193, 156, 0) => Ok(Self::new(ColorPalette::Yellow, false)),
            (0, 55, 218) => Ok(Self::new(ColorPalette::Blue, false)),
            (136, 23, 152) => Ok(Self::new(ColorPalette::Magenta, false)),
            (58, 150, 221) => Ok(Self::new(ColorPalette::Cyan, false)),
            (204, 204, 204) => Ok(Self::new(ColorPalette::White, false)),
            (118, 118, 118) => Ok(Self::new(ColorPalette::Black, true)),
            (231, 72, 86) => Ok(Self::new(ColorPalette::Red, true)),
            (22, 198, 12) => Ok(Self::new(ColorPalette::Green, true)),
            (249, 241, 165) => Ok(Self::new(ColorPalette::Yellow, true)),
            (59, 120, 255) => Ok(Self::new(ColorPalette::Blue, true)),
            (180, 0, 255) => Ok(Self::new(ColorPalette::Magenta, true)),
            (97, 214, 214) => Ok(Self::new(ColorPalette::Cyan, true)),
            (242, 242, 242) => Ok(Self::new(ColorPalette::White, true)),
            _ => Err(()),
        }
    }
}

impl AsRef<Color> for Color {
    fn as_ref(&self) -> &Color {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    pub(super) r: u8,
    pub(super) g: u8,
    pub(super) b: u8,
}
impl RgbColor {
    pub const BLACK: Self = Self::new(12, 12, 12);
    pub const RED: Self = Self::new(197, 15, 12);
    pub const GREEN: Self = Self::new(19, 161, 14);
    pub const YELLOW: Self = Self::new(193, 156, 0);
    pub const BLUE: Self = Self::new(0, 55, 218);
    pub const MAGENTA: Self = Self::new(136, 23, 152);
    pub const CYAN: Self = Self::new(58, 150, 221);
    pub const WHITE: Self = Self::new(204, 204, 204);
    pub const BRIGHT_BLACK: Self = Self::new(118, 118, 118);
    pub const BRIGHT_RED: Self = Self::new(231, 72, 86);
    pub const BRIGHT_GREEN: Self = Self::new(22, 198, 12);
    pub const BRIGHT_YELLOW: Self = Self::new(249, 241, 165);
    pub const BRIGHT_BLUE: Self = Self::new(59, 120, 255);
    pub const BRIGHT_MAGENTA: Self = Self::new(180, 0, 255);
    pub const BRIGHT_CYAN: Self = Self::new(97, 214, 214);
    pub const BRIGHT_WHITE: Self = Self::new(242, 242, 242);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub fn to_ansi_escape_sequence(&self) -> String {
        format!("\u{1b}[38;2;{};{};{}m", self.r, self.g, self.b)
    }
    pub fn from_ansi_code(code: u8) -> Option<Self> {
        Some(match code {
            // Using the "Windows 10 Console" colors
            30 => Self::new(12, 12, 12),    // Black
            31 => Self::new(197, 15, 12),   // Red
            32 => Self::new(19, 161, 14),   // Green
            33 => Self::new(193, 156, 0),   // Yellow
            34 => Self::new(0, 55, 218),    // Blue
            35 => Self::new(136, 23, 152),  // Magenta
            36 => Self::new(58, 150, 221),  // Cyan
            37 => Self::new(204, 204, 204), // White
            90 => Self::new(118, 118, 118), // Bright Black (Gray)
            91 => Self::new(231, 72, 86),   // Bright Red
            92 => Self::new(22, 198, 12),   // Bright Green
            93 => Self::new(249, 241, 165), // Bright Yellow
            94 => Self::new(59, 120, 255),  // Bright Blue
            95 => Self::new(180, 0, 255),   // Bright Magenta
            96 => Self::new(97, 214, 214),  // Bright Cyan
            97 => Self::new(242, 242, 242), // Bright White
            _ => return None,
        })
    }
    pub fn to_ansi_code(&self) -> Option<u8> {
        Color::try_from(*self)
            .ok()
            .map(|c| c.color as u8 + if c.bright { 60 } else { 0 })
    }
}
impl Display for RgbColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ansi_escape_sequence())
    }
}
impl<I: Into<Color>> From<I> for RgbColor {
    fn from(value: I) -> Self {
        let color = value.into();
        match (color.color, color.bright) {
            (ColorPalette::Black, false) => Self::BLACK,
            (ColorPalette::Red, false) => Self::RED,
            (ColorPalette::Green, false) => Self::GREEN,
            (ColorPalette::Yellow, false) => Self::YELLOW,
            (ColorPalette::Blue, false) => Self::BLUE,
            (ColorPalette::Magenta, false) => Self::MAGENTA,
            (ColorPalette::Cyan, false) => Self::CYAN,
            (ColorPalette::White, false) => Self::WHITE,
            (ColorPalette::Black, true) => Self::BRIGHT_BLACK,
            (ColorPalette::Red, true) => Self::BRIGHT_RED,
            (ColorPalette::Green, true) => Self::BRIGHT_GREEN,
            (ColorPalette::Yellow, true) => Self::BRIGHT_YELLOW,
            (ColorPalette::Blue, true) => Self::BRIGHT_BLUE,
            (ColorPalette::Magenta, true) => Self::BRIGHT_MAGENTA,
            (ColorPalette::Cyan, true) => Self::BRIGHT_CYAN,
            (ColorPalette::White, true) => Self::BRIGHT_WHITE,
        }
    }
}
impl Deref for RgbColor {
    type Target = (u8, u8, u8);

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const RgbColor as *const (u8, u8, u8)) }
    }
}
impl DerefMut for RgbColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut RgbColor as *mut (u8, u8, u8)) }
    }
}

impl AsRef<RgbColor> for RgbColor {
    fn as_ref(&self) -> &RgbColor {
        self
    }
}

pub fn colors() -> impl Iterator<Item = RgbColor> {
    // VIP SECTION: Color generation
    // Just a placeholder for debugging purposes
    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = if (0.0..60.0).contains(&h) {
            (c, x, 0.0)
        } else if (60.0..120.0).contains(&h) {
            (x, c, 0.0)
        } else if (120.0..180.0).contains(&h) {
            (0.0, c, x)
        } else if (180.0..240.0).contains(&h) {
            (0.0, x, c)
        } else if (240.0..300.0).contains(&h) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        (
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
        )
    }

    let rand_start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
        % 360;

    // Generate distinct colors for each label if not set
    (rand_start..).map(|i| {
        let hue = (i as f32 * 137.508) % 360.0; // use golden angle approximation
        let (r, g, b) = hsl_to_rgb(hue, 0.5, 0.5);
        RgbColor::new(r, g, b)
    })
}
