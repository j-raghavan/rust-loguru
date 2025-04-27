use std::env;

/// Check if the terminal supports colors
pub fn supports_colors() -> bool {
    if let Ok(term) = env::var("TERM") {
        !term.contains("dumb")
    } else {
        false
    }
}

/// Check if the terminal supports ANSI colors
pub fn supports_ansi_colors() -> bool {
    if let Ok(no_color) = env::var("NO_COLOR") {
        if !no_color.is_empty() {
            return false;
        }
    }

    if let Ok(clicolor) = env::var("CLICOLOR") {
        if clicolor == "0" {
            return false;
        }
    }

    supports_colors()
}

/// ANSI color codes
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";
    pub const BLINK: &str = "\x1b[5m";
    pub const REVERSE: &str = "\x1b[7m";
    pub const HIDDEN: &str = "\x1b[8m";

    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

    pub const BRIGHT_BLACK: &str = "\x1b[90m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m";

    pub const BG_BLACK: &str = "\x1b[40m";
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_GREEN: &str = "\x1b[42m";
    pub const BG_YELLOW: &str = "\x1b[43m";
    pub const BG_BLUE: &str = "\x1b[44m";
    pub const BG_MAGENTA: &str = "\x1b[45m";
    pub const BG_CYAN: &str = "\x1b[46m";
    pub const BG_WHITE: &str = "\x1b[47m";

    pub const BG_BRIGHT_BLACK: &str = "\x1b[100m";
    pub const BG_BRIGHT_RED: &str = "\x1b[101m";
    pub const BG_BRIGHT_GREEN: &str = "\x1b[102m";
    pub const BG_BRIGHT_YELLOW: &str = "\x1b[103m";
    pub const BG_BRIGHT_BLUE: &str = "\x1b[104m";
    pub const BG_BRIGHT_MAGENTA: &str = "\x1b[105m";
    pub const BG_BRIGHT_CYAN: &str = "\x1b[106m";
    pub const BG_BRIGHT_WHITE: &str = "\x1b[107m";
}

/// Apply ANSI color to text if colors are supported
pub fn colorize(text: &str, color: &str) -> String {
    if supports_ansi_colors() {
        format!("{}{}{}", color, text, colors::RESET)
    } else {
        text.to_string()
    }
}

/// Apply ANSI style to text if colors are supported
pub fn style(text: &str, style: &str) -> String {
    if supports_ansi_colors() {
        format!("{}{}{}", style, text, colors::RESET)
    } else {
        text.to_string()
    }
}
