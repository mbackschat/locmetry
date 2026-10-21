use super::*;

#[test]
fn no_color_disables_every_terminal_style() {
    let theme = Styling { no_color: true }.theme(true);
    assert_eq!(theme.paint(Role::Error, "error"), "error");
    assert_eq!(theme.strong(Role::Warning, "warning"), "warning");
}

#[test]
fn no_color_disables_clap_styling_before_help_and_error_exits() {
    for args in [
        vec!["locmetry", "--no-color", "--help"],
        vec!["locmetry", "compare", "--help", "--no-color"],
        vec!["locmetry", "--format", "invalid", "--no-color"],
    ] {
        let args: Vec<_> = args.into_iter().map(std::ffi::OsString::from).collect();
        let mut command = cli_command(&args);
        assert_eq!(command.get_color(), clap::ColorChoice::Never);
        assert!(command.try_get_matches_from_mut(args).is_err());
    }
    let args = ["locmetry", "--", "--no-color"].map(std::ffi::OsString::from);
    assert_eq!(cli_command(&args).get_color(), clap::ColorChoice::Auto);
}

/// Usage errors follow the palette: clap's own green for valid values
/// would give green a third meaning beside Production and increases.
#[test]
fn usage_errors_use_the_palette_instead_of_clap_colors() {
    let error = Cli::command()
        .styles(locmetry::style::Palette::ANSI.cli_styles())
        .color(clap::ColorChoice::Always)
        .try_get_matches_from(["locmetry", "--format", "nope"])
        .unwrap_err();
    let styled = error.render().ansi().to_string();
    assert!(styled.contains("\x1b[1mtable\x1b[0m"), "{styled:?}");
    assert!(!styled.contains("\x1b[32m"), "{styled:?}");
}

fn assert_help_layout(args: &[&str]) {
    for width in [60, 80, 100] {
        let error = Cli::command()
            .term_width(width)
            .color(clap::ColorChoice::Never)
            .try_get_matches_from(args)
            .unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
        let help = error.to_string();
        for line in help.lines() {
            assert!(
                line.chars().count() <= width,
                "{args:?}, width {width}: overlong help line {line:?}"
            );
        }
        for section in ["Commands:\n", "Arguments:\n", "Options:\n"] {
            if let Some((_, content)) = help.split_once(section) {
                let content = content.split("\n\n").next().unwrap();
                assert!(
                    content.lines().all(|line| line.starts_with("  ")),
                    "{args:?}, width {width}: unindented help in {section}{content}"
                );
            }
        }
    }
}

#[test]
fn root_help_wraps_and_indents_descriptions() {
    for flag in ["-h", "--help"] {
        assert_help_layout(&["locmetry", flag]);
    }
}

#[test]
fn subcommand_help_wraps_and_indents_descriptions() {
    for command in ["workspace", "compare", "explain"] {
        for flag in ["-h", "--help"] {
            assert_help_layout(&["locmetry", command, flag]);
        }
    }
}

#[test]
fn bars_are_automatic_only_on_a_terminal_unless_requested_or_refused() {
    let comments = Some(BarKind::Comments);
    assert_eq!(bar_mode(None, false, true, 150), Bars::Fit(150));
    assert_eq!(bar_mode(None, false, false, 150), Bars::Off);
    assert_eq!(
        bar_mode(comments, false, false, 150),
        Bars::Always(150, BarKind::Comments)
    );
    assert_eq!(
        bar_mode(comments, false, true, 150),
        Bars::Always(150, BarKind::Comments)
    );
    assert_eq!(bar_mode(None, true, true, 150), Bars::Off);
}

#[test]
fn emphasis_needs_a_capable_terminal() {
    assert!(emphasis(true, None, Some("xterm-256color".into()), None));
    assert!(emphasis(true, Some(String::new()), None, None));
    assert!(!emphasis(false, None, None, None));
    assert!(!emphasis(true, Some("1".into()), None, None));
    assert!(!emphasis(true, None, Some("dumb".into()), None));
}

/// A Windows console that refuses virtual-terminal processing would print
/// escape sequences literally, so output falls back to plain text.
#[test]
fn emphasis_needs_a_windows_console_that_accepts_escape_sequences() {
    assert!(emphasis(true, None, None, Some(true)));
    assert!(!emphasis(true, None, None, Some(false)));
}

#[test]
fn explicit_exports_default_to_full_json_and_ordinary_pipes_to_headlines() {
    assert_eq!(default_format(true, false), Format::Table);
    assert_eq!(default_format(false, false), Format::Headline);
    assert_eq!(default_format(true, true), Format::Json);
    assert_eq!(default_format(false, true), Format::Json);
}
