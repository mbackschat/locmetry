//! Semantic presentation roles shared by human renderers; never used for JSON.
//!
//! Renderers say what text means ([`Role`]) and whether it is emphasized
//! ([`Theme::strong`]). A [`Palette`] alone decides how each role looks in a
//! terminal, so another palette, such as one tuned for light backgrounds,
//! changes no renderer.

use crate::Purpose;
use clap::builder::styling::{AnsiColor, Style as CliStyle, Styles};

/// Flatten control characters before adding terminal layout or styling.
pub fn plain(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

#[derive(Clone, Copy)]
pub enum Role {
    /// Titles, column headings, row labels and total rows.
    Heading,
    /// The Code column and the Code share chart: the measure purposes split.
    Code,
    /// Figures beside the main ones, such as Files, Comments, Blanks and ratios.
    Secondary,
    /// Zeros, legends, separators and notes that should recede.
    Muted,
    /// A net change between two reports; direction, not a judgement.
    Increase,
    Decrease,
    /// Labels of neutral notes and findings.
    Information,
    Warning,
    Error,
    /// A purpose's identity hue; never a judgement, so no purpose borrows Warning.
    Purpose(Purpose),
}

/// Colors a palette may use: the 16-color ANSI set, which terminals map to
/// their own light or dark scheme.
#[derive(Clone, Copy)]
pub enum Hue {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
}

impl Hue {
    const fn sgr(self) -> &'static str {
        match self {
            Hue::Red => "31",
            Hue::Green => "32",
            Hue::Yellow => "33",
            Hue::Blue => "34",
            Hue::Magenta => "35",
            Hue::Cyan => "36",
            Hue::Gray => "90",
        }
    }

    const fn ansi(self) -> AnsiColor {
        match self {
            Hue::Red => AnsiColor::Red,
            Hue::Green => AnsiColor::Green,
            Hue::Yellow => AnsiColor::Yellow,
            Hue::Blue => AnsiColor::Blue,
            Hue::Magenta => AnsiColor::Magenta,
            Hue::Cyan => AnsiColor::Cyan,
            Hue::Gray => AnsiColor::BrightBlack,
        }
    }
}

/// How one role looks in a terminal.
#[derive(Clone, Copy)]
pub struct Ink {
    hue: Option<Hue>,
    bold: bool,
    dim: bool,
}

impl Ink {
    const PLAIN: Self = Self {
        hue: None,
        bold: false,
        dim: false,
    };

    const fn hue(hue: Hue) -> Self {
        Self {
            hue: Some(hue),
            ..Self::PLAIN
        }
    }

    const fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    const fn dim(self) -> Self {
        Self { dim: true, ..self }
    }

    /// SGR parameters, `None` when the ink adds nothing; `strong` adds bold.
    fn sgr(self, strong: bool) -> Option<String> {
        let params: Vec<&str> = [
            (self.bold || strong).then_some("1"),
            self.dim.then_some("2"),
            self.hue.map(Hue::sgr),
        ]
        .into_iter()
        .flatten()
        .collect();
        (!params.is_empty()).then(|| params.join(";"))
    }

    fn cli(self) -> CliStyle {
        let style = CliStyle::new().fg_color(self.hue.map(|hue| hue.ansi().into()));
        let style = if self.bold { style.bold() } else { style };
        if self.dim {
            style.dimmed()
        } else {
            style
        }
    }
}

/// The terminal look of every role.
#[derive(Clone, Copy)]
pub struct Palette {
    heading: Ink,
    code: Ink,
    secondary: Ink,
    muted: Ink,
    increase: Ink,
    decrease: Ink,
    information: Ink,
    warning: Ink,
    error: Ink,
    /// In `Purpose::ALL` order.
    purposes: [Ink; 3],
}

impl Palette {
    /// The 16 ANSI colors; terminals map them to their light or dark scheme.
    /// Every hue has one meaning, except green and red: purposes never appear
    /// in comparison tables, so changes follow the diff convention there.
    pub const ANSI: Self = Self {
        heading: Ink::PLAIN.bold(),
        code: Ink::hue(Hue::Cyan).bold(),
        secondary: Ink::PLAIN.dim(),
        muted: Ink::hue(Hue::Gray),
        increase: Ink::hue(Hue::Green),
        decrease: Ink::hue(Hue::Red),
        information: Ink::PLAIN.bold(),
        warning: Ink::hue(Hue::Yellow),
        error: Ink::hue(Hue::Red).bold(),
        purposes: [
            Ink::hue(Hue::Green),
            Ink::hue(Hue::Magenta),
            Ink::hue(Hue::Blue),
        ],
    };

    fn ink(&self, role: Role) -> Ink {
        match role {
            Role::Heading => self.heading,
            Role::Code => self.code,
            Role::Secondary => self.secondary,
            Role::Muted => self.muted,
            Role::Increase => self.increase,
            Role::Decrease => self.decrease,
            Role::Information => self.information,
            Role::Warning => self.warning,
            Role::Error => self.error,
            Role::Purpose(purpose) => self.purposes[purpose.index()],
        }
    }

    /// Help and usage-error styles from the same roles, so clap's output
    /// follows the palette instead of its own (whose green marks valid values).
    pub fn cli_styles(&self) -> Styles {
        Styles::styled()
            .header(self.heading.cli())
            .usage(self.heading.cli())
            .literal(self.heading.cli())
            .placeholder(CliStyle::new())
            .valid(self.heading.cli())
            .invalid(self.warning.cli())
            .error(self.error.cli())
            .context(self.muted.cli())
            .context_value(self.heading.cli())
    }
}

/// Where human output goes: plain text, a styled terminal, or Markdown.
/// Renderers choose meaning, not escape codes; padding must be computed before
/// styling so terminal controls do not affect alignment.
#[derive(Clone, Copy)]
pub enum Theme {
    Plain,
    Terminal(Palette),
    Markdown,
}

impl Theme {
    pub const PLAIN: Self = Theme::Plain;
    pub const TERMINAL: Self = Theme::Terminal(Palette::ANSI);
    pub const MARKDOWN: Self = Theme::Markdown;

    pub fn terminal(enabled: bool) -> Self {
        match enabled {
            true => Self::TERMINAL,
            false => Self::PLAIN,
        }
    }

    /// Style already escaped text; callers retain responsibility for their output context.
    pub fn paint(self, role: Role, text: &str) -> String {
        self.style(role, false, text)
    }

    /// Like [`Theme::paint`], emphasized as column headings and total rows are.
    pub fn strong(self, role: Role, text: &str) -> String {
        self.style(role, true, text)
    }

    fn style(self, role: Role, strong: bool, text: &str) -> String {
        match self {
            Theme::Plain => text.to_string(),
            Theme::Terminal(palette) => match palette.ink(role).sgr(strong) {
                Some(params) => format!("\x1b[{params}m{text}\x1b[0m"),
                None => text.to_string(),
            },
            // Markdown has only bold and italic: bold carries labels, headings
            // and emphasis, italic marks what recedes, and hues are dropped.
            Theme::Markdown => match (role, strong) {
                (
                    Role::Heading | Role::Code | Role::Information | Role::Warning | Role::Error,
                    _,
                )
                | (_, true) => format!("**{text}**"),
                (Role::Muted, false) => format!("*{text}*"),
                _ => text.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Locks color consistency: each hue means one thing. Green and red are
    /// the documented exception, purposes in measurement tables and changes in
    /// comparisons, which never share a table.
    #[test]
    fn each_palette_hue_has_one_meaning() {
        let palette = Palette::ANSI;
        let mut roles = vec![
            ("heading", Role::Heading),
            ("code", Role::Code),
            ("secondary", Role::Secondary),
            ("muted", Role::Muted),
            ("increase", Role::Increase),
            ("decrease", Role::Decrease),
            ("information", Role::Information),
            ("warning", Role::Warning),
            ("error", Role::Error),
        ];
        roles.extend(
            Purpose::ALL
                .map(|purpose| (purpose.to_string().leak() as &str, Role::Purpose(purpose))),
        );
        let mut meanings: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
        for (name, role) in roles {
            if let Some(hue) = palette.ink(role).hue {
                meanings.entry(hue.sgr()).or_default().push(name);
            }
        }
        for (hue, names) in meanings {
            let allowed = match hue {
                "32" => vec!["increase", "production"],
                "31" => vec!["decrease", "error"],
                _ => vec![names[0]],
            };
            assert_eq!(names, allowed, "hue {hue}");
        }
    }

    #[test]
    fn strong_adds_bold_to_any_role_without_changing_its_hue() {
        let theme = Theme::TERMINAL;
        assert_eq!(theme.paint(Role::Increase, "+1"), "\x1b[32m+1\x1b[0m");
        assert_eq!(theme.strong(Role::Increase, "+1"), "\x1b[1;32m+1\x1b[0m");
        assert_eq!(theme.strong(Role::Code, "9"), theme.paint(Role::Code, "9"));
        assert_eq!(Theme::PLAIN.strong(Role::Decrease, "-1"), "-1");
        assert_eq!(Theme::MARKDOWN.strong(Role::Decrease, "-1"), "**-1**");
    }
}
