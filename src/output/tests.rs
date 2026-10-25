use super::terminal::TerminalNote;
use super::*;

/// A row with this Code per purpose and no comments.
fn code(purposes: [u64; 3]) -> RowCode {
    RowCode {
        purposes,
        comments: 0,
    }
}

/// `columns` over `body` and `total` as a text table.
fn text_table(
    columns: &[Column],
    body: &[Vec<String>],
    total: Option<&[String]>,
    theme: Theme,
) -> String {
    shared_table(&Columns::of(columns), body, total, false, theme)
}

/// Remove every SGR sequence, whatever the palette emits.
fn unstyled(text: &str) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("\x1b[") {
        output += before;
        rest = after.split_once('m').unwrap().1;
    }
    output + rest
}

#[test]
fn terminal_notes_sanitize_before_applying_their_own_style() {
    for theme in [Theme::PLAIN, Theme::TERMINAL] {
        assert_eq!(
            TerminalNote::Text("path\x1b[31m\nvalue\t".into()).render(theme),
            "path [31m value "
        );
        assert_eq!(
            TerminalNote::CoverageGap("ext\x1b[31m\rvalue".into()).render(theme),
            format!(
                "{} ext [31m value",
                theme.paint(Role::Warning, "Not counted:")
            )
        );
    }
}

#[test]
fn table_sanitizes_every_cell_before_padding_and_styling() {
    for theme in [Theme::PLAIN, Theme::terminal(true)] {
        let actual = text_table(
            &[
                Column::new("Na\tme", ColumnRole::Label),
                Column::new("Co\nde", ColumnRole::Count),
            ],
            &[vec!["a\x1b[31mred".into(), "1\r2".into()]],
            Some(&["To\ttal".into(), "3\x07".into()]),
            theme,
        );
        let expected = text_table(
            &[
                Column::new("Na me", ColumnRole::Label),
                Column::new("Co de", ColumnRole::Count),
            ],
            &[vec!["a [31mred".into(), "1 2".into()]],
            Some(&["To tal".into(), "3 ".into()]),
            theme,
        );
        assert_eq!(actual, expected);
        assert_eq!(actual.lines().count(), 5);
    }
}

#[test]
fn legends_are_muted_in_root_and_workspace_tables_without_changing_plain_output() {
    let rows = [];
    let summary = crate::build_report(&rows);
    let coverage = Coverage::default();
    let member = [MemberView {
        name: "api".into(),
        unclaimed_files: 0,
        unclaimed_code: 0,
        unread_files: 0,
        parse_failed_files: 0,
    }];
    for members in [&[][..], &member[..]] {
        let findings = Findings {
            subfolder: None,
            coverage: &coverage,
            modules: &[],
            module_files: &[],
            submodules: &[],
            unused_entries: &[],
            policy_entries: &[],
            category_meanings: &Default::default(),
            unused_exclusions: &[],
            members,
        };
        let render = |theme| {
            let options = TableOptions {
                categories: true,
                folders: true,
                verbose: false,
                order: RowOrder::Name,
                theme,
                folder_rows: None,
                bars: Bars::Off,
                purposes: Purpose::ALL.to_vec(),
                category_selection: Vec::new(),
                languages: Vec::new(),
                tables_only: false,
                variants: LanguageVariants::Split,
            };
            render_table_with_options(&summary, &rows, &findings, options)
        };
        let (styled, plain) = (render(Theme::TERMINAL), render(Theme::PLAIN));
        let (plain_start, styled_start) = (" ─", "\x1b[90m ─");
        assert!(plain.starts_with(plain_start), "{plain}");
        assert!(!plain.contains("Code by purpose"));
        assert!(styled.starts_with(styled_start), "{styled}");
        let legend = ALL_PURPOSES.legend(Some("Lines = Code + Comments + Blanks"), None);
        assert!(styled.contains(&format!("\x1b[90m{legend}\x1b[0m")));
        if !members.is_empty() {
            assert!(!plain.contains('▸'), "{plain}");
            assert!(plain.contains("\n Member "), "{plain}");
            let legend = ALL_PURPOSES.legend(None, Some("Share = Code / workspace Code"));
            assert!(styled.contains(&format!("\x1b[90m{legend}\x1b[0m")));
        }
        assert_eq!(unstyled(&styled), plain);
    }
}

#[test]
fn grouped_inserts_a_comma_before_every_third_digit_from_the_right() {
    let cases = [
        (0, "0"),
        (999, "999"),
        (1_000, "1,000"),
        (12_345, "12,345"),
        (123_456, "123,456"),
        (1_234_567, "1,234,567"),
    ];
    for (count, expected) in cases {
        assert_eq!(grouped(count), expected);
    }
}

#[test]
fn table_shades_secondary_counts_and_zeros_without_judging_purpose() {
    let table = text_table(
        &[
            Column::new("Name", ColumnRole::Label),
            Column::new("Code", ColumnRole::Code),
            Column::new("Comments", ColumnRole::Secondary).separated(),
            Column::new("Blanks", ColumnRole::Secondary),
            Column::new("Other", ColumnRole::Count).separated(),
        ],
        &[vec![
            "api".into(),
            "12".into(),
            "3".into(),
            "0".into(),
            "4".into(),
        ]],
        None,
        Theme::TERMINAL,
    );
    assert!(table.contains("\x1b[1mapi \x1b[0m"), "{table}");
    assert!(table.contains("\x1b[2m       3\x1b[0m"), "{table}");
    assert!(table.contains("\x1b[90m     0\x1b[0m"), "{table}");
    assert!(table.contains("    4\n"), "{table}");
    assert!(!table.contains("\x1b[33m"));
}

/// Each purpose keeps one hue from header to total, so the header doubles as
/// the legend; zeros stay gray so an absent purpose recedes.
#[test]
fn purpose_columns_carry_their_purpose_hue_from_header_to_total() {
    let mut columns = vec![
        Column::new("Name", ColumnRole::Label),
        Column::new("Code", ColumnRole::Code),
    ];
    columns.extend(
        Purpose::ALL
            .map(|purpose| Column::new(purpose_title(purpose), ColumnRole::Purpose(purpose))),
    );
    let table = text_table(
        &columns,
        &[vec![
            "api".into(),
            "9".into(),
            "5".into(),
            "4".into(),
            "0".into(),
        ]],
        Some(&[
            "Total".into(),
            "9".into(),
            "5".into(),
            "4".into(),
            "0".into(),
        ]),
        Theme::TERMINAL,
    );
    let lines: Vec<_> = table.lines().collect();
    let (header, body, total) = (lines[0], lines[2], lines[4]);
    assert!(header.contains("\x1b[1;36mCode\x1b[0m"), "{header}");
    assert!(header.contains("\x1b[1;32mProduction\x1b[0m"), "{header}");
    assert!(header.contains("\x1b[1;35mTest\x1b[0m"), "{header}");
    assert!(header.contains("\x1b[1;34mOther\x1b[0m"), "{header}");
    assert!(body.contains("\x1b[32m         5\x1b[0m"), "{body}");
    assert!(body.contains("\x1b[35m   4\x1b[0m"), "{body}");
    assert!(body.contains("\x1b[90m    0\x1b[0m"), "{body}");
    assert!(total.contains("\x1b[1;32m         5\x1b[0m"), "{total}");
    assert!(total.contains("\x1b[1;35m   4\x1b[0m"), "{total}");
}

/// Bars join a table only when the minimum inline bar fits beside it, and
/// never narrower than their column title; the threshold follows the
/// table's width, not a fixed terminal size.
#[test]
fn inline_bars_need_the_minimum_bar_beside_the_table() {
    let threshold = 90 + 2 + 10;
    assert_eq!(inline_bar_width(90, "Code share", threshold - 1), None);
    assert_eq!(inline_bar_width(90, "Code share", threshold), Some(10));
    assert_eq!(inline_bar_width(90, "Code share", threshold + 7), Some(17));
    assert_eq!(
        inline_bar_width(90, "Code share", 1000),
        Some(*BAR_CELLS.end())
    );

    let threshold = 90 + 2 + "Comment density".len();
    assert_eq!(inline_bar_width(90, "Comment density", threshold - 1), None);
    assert_eq!(inline_bar_width(90, "Comment density", threshold), Some(15));
}

/// Automatic bars only decorate a table they fit beside: otherwise the
/// table stays exactly as without bars, with neither Share column nor chart.
/// The oracle is the language table's former hand-written layout: Code
/// accented, Comments and the purposes and ratio each starting a group,
/// counts beside Code and the ratio secondary.
#[test]
fn columns_derive_the_layout_their_roles_describe() {
    let header = |columns: &Columns| {
        let table = shared_table(columns, &[], None, false, Theme::PLAIN);
        table.lines().next().unwrap().to_string()
    };
    let language = Columns::with_purposes(&LANGUAGE_COLUMNS, &ALL_PURPOSES);
    assert_eq!(
        header(&language),
        " Language  Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod"
    );
    let styled = shared_table(&language, &[], None, false, Theme::TERMINAL);
    assert!(styled.contains("\x1b[1;36mCode\x1b[0m"), "{styled}");
    assert!(styled.contains("\x1b[1;34mOther\x1b[0m"), "{styled}");

    // One purpose leaves only the ratio, which still starts its group.
    let production = PurposeColumns {
        shown: &[Purpose::Production],
        sliced: false,
    };
    let folder = Columns::with_purposes(&code_columns("Folder"), &production);
    assert_eq!(header(&folder), " Folder  Lines  Code │ Comment %");
}

/// Total rows emphasize every hue, so a total's change stays as bold as
/// the rest of its row while keeping its direction color.
#[test]
fn total_rows_emphasize_changes_like_their_other_figures() {
    let columns = Columns::of(&[
        Column::new("Kind", ColumnRole::Label),
        Column::new("Lines", ColumnRole::Count),
        Column::new("Change", ColumnRole::Change),
    ]);
    let body = [vec!["Source".into(), "2".into(), "+1".into()]];
    let total = vec!["All".into(), "2".into(), "-1".into()];
    let table = shared_table(&columns, &body, Some(&total), false, Theme::TERMINAL);
    assert!(table.contains("\x1b[32m    +1\x1b[0m"), "{table:?}");
    assert!(table.contains("\x1b[1;31m    -1\x1b[0m"), "{table:?}");
    assert!(table.contains("\x1b[1m    2\x1b[0m"), "{table:?}");
}

/// A group name is printed once, centred over its columns; groups end at
/// a `│`, widen to fit their name, and Markdown keeps the full titles.
#[test]
fn group_headings_span_their_columns_and_widen_to_fit_their_name() {
    let columns = Columns::of(&[
        Column::new("Kind", ColumnRole::Label),
        Column::new("Lines before", ColumnRole::Count).under("Lines", "before"),
        Column::new("Lines after", ColumnRole::Count).under("Lines", "after"),
        Column::new("Files change", ColumnRole::Count)
            .under("Changes in files", "n")
            .separated(),
    ]);
    let body = [vec!["Source".into(), "1".into(), "2".into(), "+1".into()]];
    let table = shared_table(&columns, &body, None, false, Theme::PLAIN);
    let lines: Vec<_> = table.lines().collect();
    assert_eq!(
        lines[0], "             Lines     │ Changes in files",
        "{table}"
    );
    assert_eq!(
        lines[1], " Kind    before  after │                n",
        "{table}"
    );
    assert!(shared_table(&columns, &body, None, true, Theme::MARKDOWN)
        .starts_with("| Kind | Lines before | Lines after | Files change |"));
}

/// Centring pads a group name on both sides, and an ungrouped column
/// pads to its width; neither may leave blanks after the line's last
/// name, styled or plain, whether a group or an ungrouped column closes it.
#[test]
fn group_lines_end_at_their_last_name() {
    let grouped = [
        Column::new("Kind", ColumnRole::Label),
        Column::new("Lines before", ColumnRole::Count).under("Lines", "before"),
        Column::new("Lines after", ColumnRole::Count).under("Lines", "after"),
    ];
    let ungrouped = [
        grouped[0],
        grouped[1],
        grouped[2],
        Column::new("Files", ColumnRole::Count).separated(),
    ];
    let body = [vec![
        "Source".into(),
        "1,000".into(),
        "2,000".into(),
        "1".into(),
    ]];
    for (columns, expected) in [
        (&grouped[..], "             Lines"),
        (&ungrouped[..], "             Lines     │"),
    ] {
        let body = [body[0][..columns.len()].to_vec()];
        for theme in [Theme::PLAIN, Theme::TERMINAL] {
            let table = shared_table(&Columns::of(columns), &body, None, false, theme);
            let first = unstyled(table.lines().next().unwrap());
            assert_eq!(first, expected, "{table:?}");
        }
    }
}

/// A short title replaces a full one only while the full title would be
/// wider than every value; Markdown keeps the full title regardless.
#[test]
fn short_titles_apply_only_when_the_full_title_outgrows_its_values() {
    let columns = Columns::with_purposes(&code_columns("Folder"), &ALL_PURPOSES);
    let row = |production: &str| {
        vec![
            "src".into(),
            "1".into(),
            "1".into(),
            production.into(),
            "0".into(),
            "0".into(),
            "-".into(),
        ]
    };
    let heading =
        |production| PlainTable::new(&columns, &[row(production)], None).header[3].clone();
    assert_eq!(heading("9,978"), "Prod.");
    assert_eq!(heading("1,234,567,890"), "Production");
    assert!(columns.markdown_header().contains("| Production |"));
}

#[test]
fn markdown_headers_bold_code_and_right_align_figures() {
    assert_eq!(
        Columns::with_purposes(&code_columns("Folder"), &ALL_PURPOSES).markdown_header(),
        "| Folder | Lines | **Code** | Production | Test | Other | Test/prod |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
}

#[test]
fn fitting_bars_leave_tables_untouched_when_they_do_not_fit() {
    let body = [
        vec!["api".to_string(), "9".into()],
        vec!["web".into(), "3".into()],
    ];
    let total = vec!["Total".to_string(), "12".into()];
    let columns = Columns(vec![
        Column::new("Name", ColumnRole::Label),
        Column::new("Code", ColumnRole::Code),
    ]);
    let codes = [code([6, 3, 0]), code([0, 0, 3])];
    let render = |bars| {
        let rows = BarRows {
            rows: &codes,
            total: code([12, 0, 0]),
            columns: &ALL_PURPOSES,
            bars,
            key: true,
        };
        let (table, after) = bar_table(&columns, &body, Some(total.clone()), &rows, Theme::PLAIN);
        (table.text, after)
    };
    let plain = shared_table(&columns, &body, Some(&total), false, Theme::PLAIN);
    assert_eq!(render(Bars::Fit(30)), (plain, String::new()));
    let (wide, key) = render(Bars::Fit(60));
    assert!(
        wide.contains(" api       9 │  75.0%  █████████████████████████▓▓▓▓▓▓▓▓▓▓▓▓\n"),
        "{wide}"
    );
    assert!(key.contains("Bars:"), "{key}");
    let (_, chart) = render(Bars::Always(30, BarKind::Purpose));
    assert!(chart.contains("Code share"), "{chart}");
}

/// Five excluded languages are named, largest first; a tail of two or more
/// is counted, and the excluded code still covers all of it.
#[test]
fn language_note_names_the_largest_exclusions_and_counts_the_rest() {
    let selection = TableSelection {
        rows: None,
        purposes_excluded: None,
        categories_excluded: Vec::new(),
        sliced: false,
        languages_excluded: ["A", "B", "C", "D", "E", "F", "G"]
            .into_iter()
            .map(|language| (language.to_string(), 1_000))
            .collect(),
    };
    assert_eq!(
        selection.notes(&Purpose::ALL),
        ["Not shown (languages): A, B, C, D, E and 2 more (7,000 code lines)."]
    );
    let six = TableSelection {
        languages_excluded: selection.languages_excluded[..6].to_vec(),
        ..selection
    };
    assert_eq!(
        six.notes(&Purpose::ALL),
        ["Not shown (languages): A, B, C, D, E and F (6,000 code lines)."]
    );
}

/// The largest row fills the bar; others scale to it, split by purpose into
/// whole cells that add up to their length. Percentages are of total Code,
/// and the glyphs keep purposes apart without color.
#[test]
fn share_bars_split_each_row_by_purpose_within_the_available_width() {
    let rows = [
        ("api".to_string(), code([6, 3, 1])),
        ("web".to_string(), code([0, 0, 5])),
        ("ops".to_string(), code([4, 0, 0])),
        ("dev".to_string(), code([0, 1, 0])),
    ];
    let plain = share_bars(
        &rows,
        code([20, 0, 0]),
        BarKind::Purpose,
        33,
        &Purpose::ALL,
        Theme::PLAIN,
    );
    assert_eq!(
        plain,
        " Code share  █ Production  ▓ Test  ░ Other\n \
         api  50.0%  ████████████▓▓▓▓▓▓░░\n \
         web  25.0%  ░░░░░░░░░░\n \
         ops  20.0%  ████████\n \
         dev   5.0%  ▓▓\n"
    );
    let styled = share_bars(
        &rows,
        code([20, 0, 0]),
        BarKind::Purpose,
        33,
        &Purpose::ALL,
        Theme::TERMINAL,
    );
    assert!(
        styled.starts_with(" \x1b[1;36mCode share\x1b[0m"),
        "{styled}"
    );
    assert!(styled.contains("\x1b[32m████████████\x1b[0m\x1b[35m▓▓▓▓▓▓\x1b[0m\x1b[34m░░\x1b[0m"));
    assert_eq!(unstyled(&styled), plain);
}

/// A row too small for one cell still shows a sliver; empty rows show none.
/// The bar keeps a readable minimum and stops at 40 cells.
#[test]
fn share_bars_mark_slivers_and_clamp_their_width() {
    let rows = [
        ("big".to_string(), code([999, 0, 0])),
        ("one".to_string(), code([0, 1, 0])),
        ("nil".to_string(), code([0, 0, 0])),
    ];
    let text = share_bars(
        &rows,
        code([1000, 0, 0]),
        BarKind::Purpose,
        33,
        &Purpose::ALL,
        Theme::PLAIN,
    );
    assert!(text.contains("\n one   0.1%  ▏\n"), "{text}");
    assert!(text.contains("\n nil   0.0%\n"), "{text}");
    let bar = |columns| {
        let text = share_bars(
            &rows,
            code([1000, 0, 0]),
            BarKind::Purpose,
            columns,
            &Purpose::ALL,
            Theme::PLAIN,
        );
        let line = text.lines().nth(1).unwrap().to_string();
        line.chars().filter(|c| *c == '█').count()
    };
    assert_eq!(bar(10), 10);
    assert_eq!(bar(500), 40);
}

/// Cyan marks only the strong column, so the Total row stays bold without
/// lifting the columns the body dims.
#[test]
fn total_row_is_bold_but_keeps_each_column_role() {
    let table = text_table(
        &[
            Column::new("Name", ColumnRole::Label),
            Column::new("Code", ColumnRole::Code),
            Column::new("Comments", ColumnRole::Secondary),
            Column::new("Test", ColumnRole::Count),
            Column::new("Blanks", ColumnRole::Count),
        ],
        &[vec![
            "api".into(),
            "12".into(),
            "3".into(),
            "4".into(),
            "0".into(),
        ]],
        Some(&[
            "Total".into(),
            "12".into(),
            "3".into(),
            "4".into(),
            "0".into(),
        ]),
        Theme::TERMINAL,
    );
    let total = table.lines().last().unwrap();
    assert!(total.contains("\x1b[1mTotal\x1b[0m"), "{total}");
    assert!(total.contains("\x1b[1;36m  12\x1b[0m"), "{total}");
    assert!(total.contains("\x1b[2m       3\x1b[0m"), "{total}");
    assert!(total.contains("\x1b[1m   4\x1b[0m"), "{total}");
    assert!(total.contains("\x1b[90m     0\x1b[0m"), "{total}");
    assert!(!table.contains("\x1b[36m"), "{table}");
}

/// Bold must not shift alignment: without its escape codes, an
/// emphasized table equals the plain one.
#[test]
fn emphasis_bolds_the_strong_column_without_shifting_alignment() {
    let columns = [
        Column::new("Name", ColumnRole::Label),
        Column::new("Code", ColumnRole::Code),
        Column::new("Test", ColumnRole::Count).separated(),
    ];
    let body = [vec!["a".to_string(), "12".into(), "3".into()]];
    let total = ["Total".to_string(), "12".into(), "3".into()];
    let plain = text_table(&columns, &body, Some(&total), Theme::PLAIN);
    let bold = text_table(&columns, &body, Some(&total), Theme::TERMINAL);
    assert!(
        bold.contains("\x1b[90m"),
        "separators should recede: {bold}"
    );
    assert!(bold.contains("\x1b[1;36mCode\x1b[0m"), "{bold}");
    assert!(bold.contains("\x1b[1;36m  12\x1b[0m"), "{bold}");
    assert_eq!(unstyled(&bold), plain);
    assert!(!plain.contains('\x1b'));
}

/// A consonant before a final `y` takes `ies`, as in "categories"; a vowel
/// keeps the `s`, as in "keys".
#[test]
fn counts_pluralize_final_y_after_a_consonant() {
    assert_eq!(counted(2, "category"), "2 categories");
    assert_eq!(counted(1, "category"), "1 category");
    assert_eq!(counted(2, "key"), "2 keys");
    assert_eq!(counted(3, "file"), "3 files");
}
