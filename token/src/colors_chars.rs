use ::std::fmt::Display;

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
pub struct Color {
    pub(super) r: u8,
    pub(super) g: u8,
    pub(super) b: u8,
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub fn to_ansi_fg(&self) -> String {
        format!("\u{1b}[38;2;{};{};{}m", self.r, self.g, self.b)
    }
    pub fn from_ansi_code(code: u8) -> Option<Self> {
        Some(match code {
            30 => Self::new(0, 0, 0),       // Black
            31 => Self::new(255, 0, 0),     // Red
            32 => Self::new(0, 255, 0),     // Green
            33 => Self::new(255, 255, 0),   // Yellow
            34 => Self::new(0, 0, 255),     // Blue
            35 => Self::new(255, 0, 255),   // Magenta
            36 => Self::new(0, 255, 255),   // Cyan
            37 => Self::new(255, 255, 255), // White
            90 => Self::new(128, 128, 128), // Bright Black (Gray)
            91 => Self::new(255, 85, 85),   // Bright Red
            92 => Self::new(85, 255, 85),   // Bright Green
            93 => Self::new(255, 255, 85),  // Bright Yellow
            94 => Self::new(85, 85, 255),   // Bright Blue
            95 => Self::new(255, 85, 255),  // Bright Magenta
            96 => Self::new(85, 255, 255),  // Bright Cyan
            97 => Self::new(255, 255, 255), // Bright White
            _ => return None,
        })
    }
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ansi_fg())
    }
}

pub fn colors() -> impl Iterator<Item = Color> {
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
        Color::new(r, g, b)
    })
}
