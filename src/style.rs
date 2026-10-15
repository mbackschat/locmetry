//! Semantic presentation roles shared by human renderers; never used for JSON.

#[derive(Clone, Copy)]
pub enum Role {
    Heading,
    Primary,
    Secondary,
    Muted,
    Increase,
    Decrease,
    Information,
    Warning,
    Error,
}

#[derive(Clone, Copy)]
struct Style(&'static str, &'static str);

/// A small palette of semantic roles. Renderers choose meaning, not escape codes.
/// Padding must be computed before styling so terminal controls do not affect alignment.
#[derive(Clone, Copy)]
pub struct Theme {
    heading: Style,
    primary: Style,
    secondary: Style,
    muted: Style,
    increase: Style,
    decrease: Style,
    information: Style,
    warning: Style,
    error: Style,
}

const PLAIN: Style = Style("", "");
const BOLD: Style = Style("**", "**");

impl Theme {
    pub const PLAIN: Self = Self {
        heading: PLAIN,
        primary: PLAIN,
        secondary: PLAIN,
        muted: PLAIN,
        increase: PLAIN,
        decrease: PLAIN,
        information: PLAIN,
        warning: PLAIN,
        error: PLAIN,
    };
    pub const TERMINAL: Self = Self {
        heading: Style("\x1b[1m", "\x1b[0m"),
        primary: Style("\x1b[1;36m", "\x1b[0m"),
        secondary: Style("\x1b[2m", "\x1b[0m"),
        muted: Style("\x1b[90m", "\x1b[0m"),
        increase: Style("\x1b[36m", "\x1b[0m"),
        decrease: Style("\x1b[35m", "\x1b[0m"),
        information: Style("\x1b[36m", "\x1b[0m"),
        warning: Style("\x1b[33m", "\x1b[0m"),
        error: Style("\x1b[31m", "\x1b[0m"),
    };
    pub const MARKDOWN: Self = Self {
        heading: BOLD,
        primary: BOLD,
        muted: Style("*", "*"),
        information: BOLD,
        warning: BOLD,
        error: BOLD,
        ..Self::PLAIN
    };

    pub fn terminal(enabled: bool) -> Self {
        match enabled {
            true => Self::TERMINAL,
            false => Self::PLAIN,
        }
    }

    /// Style already escaped text; callers retain responsibility for their output context.
    pub fn paint(self, role: Role, text: &str) -> String {
        let Style(start, end) = match role {
            Role::Heading => self.heading,
            Role::Primary => self.primary,
            Role::Secondary => self.secondary,
            Role::Muted => self.muted,
            Role::Increase => self.increase,
            Role::Decrease => self.decrease,
            Role::Information => self.information,
            Role::Warning => self.warning,
            Role::Error => self.error,
        };
        format!("{start}{text}{end}")
    }

    pub fn delta(self, value: i64, width: usize) -> String {
        let role = match value.cmp(&0) {
            std::cmp::Ordering::Greater => Role::Increase,
            std::cmp::Ordering::Less => Role::Decrease,
            std::cmp::Ordering::Equal => Role::Muted,
        };
        self.paint(role, &format!("{value:>+width$}"))
    }
}
