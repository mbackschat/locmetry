use crate::common;
use common::write;
use serde_json::Value;
use std::{fs, path::Path, process::Output};

fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("absent-user-config"))
        .args(args)
        .output()
        .unwrap()
}

fn fixture(root: &Path) {
    for (path, contents) in [
        ("src/app.js", "const x = 1;\n"),
        ("package.json", "{}\n"),
        ("scripts/tool.py", "print(1)\n"),
        ("README.md", "# Example\n"),
        ("logo.png", "image\n"),
        ("vendor/lib.rs", "fn lib() {}\n"),
        (".gitmodules", "[submodule \"lib\"]\npath = vendor\n"),
        (".locmetryrc", "version = 1\n[scope]\nexclude = ['missing/**']\n[[rules]]\nid = 'ghost'\nmatch = ['absent/**']\npurpose = 'production'\n"),
    ] { write(root, path, contents); }
}

fn no_routine_notes(text: &str) {
    for note in [
        "Not source code:",
        "Production from build modules:",
        "Git submodules:",
        "Unclaimed source:",
        "Repositories to review:",
        "Unclaimed source by repository:",
        "Entries that decided",
        "Exclusions that excluded",
        "Unrecognized files:",
        "unrecognized file",
        "Classify them",
        "--verbose",
    ] {
        assert!(!text.contains(note), "unexpected {note}: {text}");
    }
}

#[test]
fn successful_tables_require_no_audit_in_root_subfolder_workspace_or_exports() {
    let temp = tempfile::tempdir().unwrap();
    fixture(&temp.path().join("project"));
    for args in [
        vec!["project", "--format", "table"],
        vec!["project", "--format", "table", "--folder-rows", "10"],
        vec!["project", "--format", "table", "--subfolder", "scripts"],
        vec!["workspace", "--root", "project", "--format", "table"],
    ] {
        let output = run(temp.path(), &args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("Other"));
        assert!(text.contains("Lines"));
        assert!(text.lines().last().unwrap().starts_with("Measured at:"));
        no_routine_notes(&text);
        assert!(!text.contains("Not counted:"));
        let mut export = args;
        export.extend(["--output", "report.txt"]);
        let output = run(temp.path(), &export);
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        no_routine_notes(&fs::read_to_string(temp.path().join("report.txt")).unwrap());
    }
}

#[test]
fn verbose_tables_expose_routine_findings_with_optional_classification_advice() {
    let temp = tempfile::tempdir().unwrap();
    fixture(&temp.path().join("project"));
    for args in [vec!["project"], vec!["workspace", "--root", "project"]] {
        let mut args = args;
        args.extend(["--format", "table", "--verbose"]);
        let output = run(temp.path(), &args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        for note in [
            "Not source code:",
            ". '--files --output -' lists each file's kind.",
            "Production from build modules:",
            "Git submodules:",
            "Unclaimed source:",
            "Entries that decided",
            "Exclusions that excluded",
            "Unrecognized files: 2 (no extension: 1, png: 1)",
            "To inspect or refine purpose classification, use '--guide configure'",
        ] {
            assert!(text.contains(note), "missing {note}: {text}");
        }
        assert!(!text.contains("Classify them"));
        assert!(!text.contains("Not counted:"));
        args.extend(["--output", "verbose.txt"]);
        let exported = run(temp.path(), &args);
        assert!(exported.status.success());
        assert!(exported.stdout.is_empty());
        assert!(exported.stderr.is_empty());
        let saved = fs::read_to_string(temp.path().join("verbose.txt")).unwrap();
        assert_eq!(
            text.rsplit_once("\nMeasured at: ").unwrap().0,
            saved.rsplit_once("\nMeasured at: ").unwrap().0
        );
    }
}

#[test]
fn mixed_failures_stay_visible_without_routine_notes_or_report_path_lists() {
    let temp = tempfile::tempdir().unwrap();
    fixture(&temp.path().join("api"));
    write(temp.path(), "api/bad.ipynb", "invalid");
    write(temp.path(), "web/bad.ipynb", "invalid");
    write(temp.path(), "web/tool.py", "print(1)\n");
    for base in [
        vec!["api"],
        vec!["workspace", "--root", "api", "--root", "web"],
    ] {
        let workspace = base[0] == "workspace";
        for verbose in [false, true] {
            let mut args = base.clone();
            args.extend(["--format", "table"]);
            if verbose {
                args.push("--verbose");
            }
            let output = run(temp.path(), &args);
            assert_eq!(output.status.code(), Some(2));
            let text = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains("bad.ipynb: invalid_notebook"), "{stderr}");
            assert!(text.contains("Backend reported incomplete counts. Inspect the JSON metadata."));
            let gap = text
                .lines()
                .find(|line| line.contains("Not counted:"))
                .unwrap();
            assert!(gap.contains("parsing failure"));
            assert!(!gap.contains("unrecognized"));
            assert_eq!(text.contains("bad.ipynb: invalid_notebook"), verbose);
            if !verbose {
                no_routine_notes(&text);
            }
            if workspace {
                let members = text
                    .lines()
                    .find(|line| line.contains("Unavailable counts:"))
                    .unwrap();
                assert!(members.contains("'api' has 1 file with a parsing failure"));
                assert!(members.contains("'web' has 1 file with a parsing failure"));
                assert!(!members.contains("unclaimed"));
                assert_eq!(text.contains("Unclaimed source by repository:"), verbose);
            }
        }
    }
}

#[test]
fn verbose_is_table_only_and_does_not_change_markdown_or_json_evidence() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    for format in ["markdown", "json", "headline"] {
        let plain = run(temp.path(), &["--format", format]);
        let detail = run(temp.path(), &["--format", format, "--verbose"]);
        assert!(plain.status.success());
        assert!(detail.status.success());
        assert!(String::from_utf8_lossy(&detail.stderr)
            .contains("'--verbose' has no effect here. It applies only to table output"));
        if format == "markdown" {
            let plain = String::from_utf8(plain.stdout).unwrap();
            let detail = String::from_utf8(detail.stdout).unwrap();
            assert_eq!(
                plain.split("\nMeasured at:").next(),
                detail.split("\nMeasured at:").next()
            );
            assert!(plain.contains("## Coverage"));
            assert!(plain.contains("Source requiring review"));
        } else {
            let mut plain: Value = serde_json::from_slice(&plain.stdout).unwrap();
            let mut detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
            plain["metadata"]["measured_at_unix_seconds"] = Value::Null;
            detail["metadata"]["measured_at_unix_seconds"] = Value::Null;
            assert_eq!(plain, detail);
            assert_eq!(plain["metadata"]["coverage"]["unrecognized_files"], 2);
        }
    }
    for query in [["--schema", "report"], ["--guide", "reports"]] {
        let args = [query.as_slice(), &["--verbose"]].concat();
        let output = run(temp.path(), &args);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
    }
    for args in [vec!["--help"], vec!["workspace", "--help"]] {
        let output = run(temp.path(), &args);
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("--verbose"));
        assert!(!help.contains("Start project configuration"));
    }
}

#[test]
fn unrecognized_files_alone_do_not_add_default_findings() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "logo.png", "image");
    let output = run(temp.path(), &["--format", "table"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    no_routine_notes(&text);
    assert!(!text.contains("Not counted:"));
    let after_legend = text.split("skew per-language Test/prod.\n").nth(1).unwrap();
    let footer: Vec<_> = after_legend.trim().lines().collect();
    assert_eq!(footer.len(), 2, "{text}");
    assert!(footer[0].starts_with("Root: ") && footer[1].starts_with("Measured at:"));
}

#[test]
fn unreadable_counts_and_member_failures_are_separate_from_optional_findings() {
    use locmetry::style::{Role, Theme};
    use locmetry::{
        build_report, render_table_with_options, Coverage, Findings, MemberView, RowOrder,
        TableOptions,
    };
    let coverage = Coverage {
        unread: vec!["api/private.rs".into()],
        unrecognized_files: 1,
        unrecognized_extensions: [("png".into(), 1)].into(),
        ..Coverage::default()
    };
    let members = [MemberView {
        name: "api".into(),
        unclaimed_files: 2,
        unclaimed_code: 3,
        unread_files: 1,
        parse_failed_files: 0,
    }];
    let findings = Findings {
        subfolder: None,
        coverage: &coverage,
        modules: &[],
        module_files: &[],
        submodules: &[],
        unused_entries: &[],
        unused_exclusions: &[],
        members: &members,
    };
    for verbose in [false, true] {
        for (theme, styled) in [(Theme::PLAIN, false), (Theme::TERMINAL, true)] {
            let text = render_table_with_options(
                &build_report(&[]),
                &[],
                &findings,
                TableOptions {
                    verbose,
                    categories: false,
                    folders: false,
                    order: RowOrder::Code,
                    theme,
                    folder_rows: None,
                    bars: locmetry::Bars::Off,
                    purposes: locmetry::Purpose::ALL.to_vec(),
                    languages: Vec::new(),
                    tables_only: false,
                    variants: locmetry::LanguageVariants::Split,
                },
            );
            assert!(
                text.contains("Unavailable counts: 'api' has 1 unreadable file."),
                "{text}"
            );
            assert!(
                text.contains(&format!(
                    "{} 1 unreadable file. Their lines are unknown, not zero.",
                    theme.paint(Role::Warning, "Not counted:")
                )),
                "{text}"
            );
            assert_eq!(
                text.contains(
                    "Unclaimed source by repository: 'api' has 2 unclaimed files (3 code lines)."
                ),
                verbose
            );
            assert_eq!(text.contains("Unrecognized files:"), verbose);
            assert!(!text.contains(&theme.paint(Role::Warning, "Unrecognized files:")) || !styled);
            if !verbose {
                no_routine_notes(&text);
            }
        }
    }
}

/// Snippet code stays out of source but is named in the verbose note, so a
/// documentation-heavy project shows how much code its documents carry. The
/// oracle is Tokei's split of the fenced block: three Rust code lines and one
/// comment inside a ten-line Markdown file; the host's own lines are prose.
#[test]
fn verbose_note_counts_code_in_documentation_snippets() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(
        &root,
        "README.md",
        "# Doc\n\nProse line.\n\n```rust\nfn main() {\n    // hi\n    println!(\"x\");\n}\n```\n",
    );
    write(&root, "GUIDE.md", "# Guide\n");
    write(&root, "src/lib.rs", "pub fn lib() {}\n");
    for args in [vec!["project"], vec!["workspace", "--root", "project"]] {
        let mut args = args;
        args.extend(["--format", "table", "--verbose", "--no-color"]);
        let output = run(temp.path(), &args);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(
            text.contains(
                " Not source code: 2 documentation files (11 lines, including 3 code lines in \
                 embedded snippets). '--files --output -' lists each file's kind.\n"
            ),
            "{text}"
        );
        assert!(text.contains("\n Total         1      1     1 │"), "{text}");
    }
}
