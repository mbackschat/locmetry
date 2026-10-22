use crate::common;
use common::{stdout, write_all};
use serde_json::Value;
use std::{path::Path, process::Output};

fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

/// Two code lines per variant file and one per base file, so each folded
/// row's figures are sums an independent reader can check. `.h` is Tokei's
/// C Header, which stays apart because it also holds C++ and Objective-C.
fn project(root: &Path) {
    write_all(
        root,
        &[
            ("src/app.ts", "export const a = 1;\n"),
            ("src/view.tsx", "export const v = 1;\nexport const w = 2;\n"),
            ("src/main.js", "const m = 1;\n"),
            ("src/item.jsx", "const i = 1;\nconst j = 2;\n"),
            ("src/core.cpp", "int core() { return 1; }\n"),
            ("src/core.hpp", "int core();\nint more();\n"),
            ("src/util.h", "int util();\n"),
        ],
    );
}

/// Files, Lines and Code of the `label` row.
fn figures(text: &str, label: &str) -> Vec<String> {
    let skip = label.split_whitespace().count();
    common::cells(text, label)[skip..skip + 3].to_vec()
}

fn has_row(text: &str, label: &str) -> bool {
    text.contains(&format!(
        "
 {label} "
    ))
}

#[test]
fn tables_fold_variants_into_their_family_and_name_each_fold() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let table = ["--format", "table", "--no-color", "--no-bars"];
    let folded = stdout(run(temp.path(), &table));
    assert_eq!(figures(&folded, "TypeScript"), ["2", "3", "3"]);
    assert_eq!(figures(&folded, "JavaScript"), ["2", "3", "3"]);
    assert_eq!(figures(&folded, "C++"), ["2", "3", "3"]);
    assert_eq!(figures(&folded, "C Header"), ["1", "1", "1"]);
    for variant in ["TSX", "JSX"] {
        assert!(!has_row(&folded, variant), "{folded}");
    }
    assert!(
        folded.contains("\n Folded: C++ Header → C++, JSX → JavaScript, TSX → TypeScript.\n"),
        "{folded}"
    );

    let split = stdout(run(
        temp.path(),
        &[&table[..], &["--split-languages"]].concat(),
    ));
    assert_eq!(figures(&split, "TypeScript"), ["1", "1", "1"]);
    assert_eq!(figures(&split, "TSX"), ["1", "2", "2"]);
    assert!(!split.contains("Folded:"), "{split}");
}

/// Selection follows what the rows show: a variant name selects its family
/// while folded and only itself when split, and the note names only folds
/// inside the selection.
#[test]
fn language_selection_matches_the_shown_rows() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let table = ["--format", "table", "--no-color", "--no-bars"];
    let folded = stdout(run(
        temp.path(),
        &[&table[..], &["--table-language", "tsx"]].concat(),
    ));
    assert_eq!(figures(&folded, "TypeScript"), ["2", "3", "3"]);
    assert!(
        folded.contains("\n Folded: TSX → TypeScript.\n"),
        "{folded}"
    );
    assert!(
        folded.contains(" Not shown (languages): C++, JavaScript and C Header (7 code lines).\n"),
        "{folded}"
    );

    let split = stdout(run(
        temp.path(),
        &[
            &table[..],
            &["--table-language", "tsx", "--split-languages"],
        ]
        .concat(),
    ));
    assert_eq!(figures(&split, "TSX"), ["1", "2", "2"]);
    assert!(!has_row(&split, "TypeScript"), "{split}");
    assert!(!split.contains("Folded:"), "{split}");
}

#[test]
fn markdown_folds_language_tables_and_json_keeps_backend_names() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let markdown = stdout(run(temp.path(), &["--format", "markdown"]));
    let section = markdown.split("## Source languages\n").nth(1).unwrap();
    assert!(section.contains("| TypeScript | 2 | 3 | 3 |"), "{section}");
    assert!(!section.contains("| TSX |"), "{section}");
    assert!(
        section.contains("Folded: C++ Header → C++, JSX → JavaScript, TSX → TypeScript."),
        "{section}"
    );
    let split = stdout(run(
        temp.path(),
        &["--format", "markdown", "--split-languages"],
    ));
    assert!(split.contains("| TSX | 1 | 2 | 2 |"), "{split}");

    let json: Value =
        serde_json::from_str(&stdout(run(temp.path(), &["--files", "--output", "-"]))).unwrap();
    let languages: Vec<_> = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file["language"].as_str().unwrap())
        .collect();
    assert!(languages.contains(&"TSX") && languages.contains(&"C++ Header"));
}

#[test]
fn workspace_repository_languages_fold_in_markdown() {
    let temp = tempfile::tempdir().unwrap();
    project(&temp.path().join("web"));
    let markdown = stdout(run(
        temp.path(),
        &["workspace", "--root", "web", "--format", "markdown"],
    ));
    let section = markdown
        .split("Source languages by member")
        .nth(1)
        .unwrap()
        .split("\n## ")
        .next()
        .unwrap();
    assert!(section.contains("| TypeScript |"), "{section}");
    assert!(!section.contains("| TSX |"), "{section}");
}

#[test]
fn comparison_language_grids_fold_unless_split() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    project(&root);
    let save = |name: &str| {
        let path = temp.path().join(name);
        stdout(run(&root, &["--output", path.to_str().unwrap()]));
        path
    };
    let before = save("before.json");
    write_all(&root, &[("src/more.tsx", "export const m = 1;\n")]);
    let after = save("after.json");
    let compare = |args: &[&str]| {
        stdout(run(
            temp.path(),
            &[
                &["compare", before.to_str().unwrap(), after.to_str().unwrap()][..],
                args,
            ]
            .concat(),
        ))
    };
    let folded = compare(&["--format", "table", "--no-color"]);
    let typescript = folded
        .lines()
        .find(|line| line.starts_with(" TypeScript "))
        .unwrap();
    assert!(typescript.contains("3      4      +1"), "{folded}");
    assert!(
        folded.contains("\n Folded: TSX → TypeScript.\n"),
        "{folded}"
    );
    let split = compare(&["--format", "table", "--no-color", "--split-languages"]);
    assert!(
        split.lines().any(|line| line.starts_with(" TSX ")),
        "{split}"
    );
    assert!(!split.contains("Folded:"), "{split}");
    let markdown = compare(&["--format", "markdown"]);
    assert!(markdown.contains("| TypeScript |"), "{markdown}");
    assert!(markdown.contains("Folded: TSX → TypeScript."), "{markdown}");
}

#[test]
fn split_languages_warns_without_a_table_or_markdown_view() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let output = run(temp.path(), &["--format", "json", "--split-languages"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "'--split-languages' has no effect here. It applies only to table and Markdown output"
    ));
}
