//! Guards the human output wording rules (docs/help-design.md, Human Output
//! Wording) on real output: each scenario's prose lines must avoid joining
//! facts with ';', bare flags, double quotes and backticks, wrong plurals,
//! signed zeros, and unquoted values that start a sentence or precede a count.
use crate::common::{self, write};
use std::path::Path;

/// A project and a second workspace member that trigger most notes: build
/// modules, inline tests, folded languages, unclaimed source, data and
/// documentation, unrecognized files, unused entries, a Git submodule and
/// folder rows with more detail than fits.
fn fixture(root: &Path) {
    let project = root.join("project");
    write(
        &project,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    );
    write(
        &project,
        "src/lib.rs",
        "pub fn a() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n",
    );
    write(
        &project,
        "src/deep/more/big.rs",
        &"pub fn b() {}\n".repeat(40),
    );
    write(&project, "tests/it.rs", "#[test]\nfn x() {}\n");
    write(&project, "web/package.json", "{\"name\": \"web\"}\n");
    write(&project, "web/src/view.tsx", "export const V = 1;\n");
    write(&project, "web/src/model.ts", "export const m = 1;\n");
    write(&project, "scripts/tool.py", "print(1)\n");
    write(&project, "data.json", "{}\n");
    write(
        &project,
        "README.md",
        "# Demo\n\n```rust\nfn main() {}\n```\n",
    );
    write(&project, "blob.xyz", "x\n");
    write(
        &project,
        ".gitmodules",
        "[submodule \"lib\"]\n\tpath = vendor/lib\n\turl = https://example.invalid/lib\n",
    );
    write(&project, "vendor/lib/x.c", "int x;\n");
    write(
        &project,
        ".locmetryrc",
        "version = 1\n[scope]\nexclude = [\"nothing-here/**\"]\n[[rules]]\nid = \"ghost\"\nmatch = [\"ghost/**\"]\npurpose = \"production\"\n",
    );
    let worker = root.join("worker");
    write(&worker, "go.mod", "module worker\n\ngo 1.21\n");
    write(&worker, "main.go", "package main\n\nfunc main() {}\n");
    write(&worker, "tools/run.py", "print(2)\n");
}

/// Values the fixture puts into output; they must be quoted inside sentences.
const VALUES: [&str; 12] = [
    "Rust",
    "TypeScript",
    "TSX",
    "Python",
    "JSON",
    "Markdown",
    "cargo",
    "npm",
    "project",
    "worker",
    "scripts",
    "vendor/lib",
];

fn scenarios() -> Vec<Vec<&'static str>> {
    vec![
        vec!["--root", "project", "--format", "table"],
        vec!["--root", "project", "--format", "table", "--verbose"],
        vec![
            "--root",
            "project",
            "--format",
            "table",
            "--table-purpose",
            "production",
            "--table-language",
            "Rust",
        ],
        vec![
            "--root",
            "project",
            "--format",
            "table",
            "--show-folders",
            "--folder-rows",
            "2",
            "--show-categories",
        ],
        vec!["--root", "project", "--format", "table", "--show-bars"],
        vec![
            "--root",
            "project",
            "--format",
            "table",
            "--show-bars=comments",
        ],
        vec![
            "workspace",
            "--root",
            "project",
            "--root",
            "worker",
            "--format",
            "table",
            "--verbose",
        ],
    ]
}

/// Lines a reader takes as sentences: not table rows, rules or aligned lists,
/// which separate their cells with runs of spaces, and not clap's usage syntax.
fn prose(text: &str) -> impl Iterator<Item = &str> {
    text.lines().map(str::trim).filter(|line| {
        !line.is_empty()
            && !line.contains('│')
            && !line.contains('─')
            && !line.contains("  ")
            && !line.starts_with("Usage:")
    })
}

/// The line without its single-quoted parts. A quote opens only after a space
/// or bracket and closes before one or punctuation, so apostrophes such as
/// "file's" stay text.
fn unquoted(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut kept = String::new();
    let mut index = 0;
    while index < chars.len() {
        let opens =
            chars[index] == '\'' && (index == 0 || matches!(chars[index - 1], ' ' | '(' | '['));
        let close = opens
            .then(|| {
                (index + 1..chars.len()).find(|&end| {
                    chars[end] == '\''
                        && chars
                            .get(end + 1)
                            .is_none_or(|next| next.is_whitespace() || ".,:;)!?".contains(*next))
                })
            })
            .flatten();
        match close {
            Some(end) => {
                kept.push_str("'…'");
                index = end + 1;
            }
            None => {
                kept.push(chars[index]);
                index += 1;
            }
        }
    }
    kept
}

/// Rule violations in one prose line.
fn violations(line: &str) -> Vec<&'static str> {
    let text = unquoted(line);
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut found = Vec::new();
    if text.contains(';') {
        found.push("';' joins facts: write separate sentences");
    }
    if words
        .iter()
        .any(|word| word.trim_start_matches('(').starts_with("--"))
    {
        found.push("flag or command outside single quotes");
    }
    if text.contains('"') || text.contains('`') {
        found.push("double quotes or backticks: use single quotes");
    }
    for pair in words.windows(2) {
        let unit = pair[1].trim_end_matches(|c: char| !c.is_alphanumeric());
        if pair[0] == "1"
            && [
                "lines", "files", "comments", "blanks", "modules", "folders", "members",
            ]
            .contains(&unit)
        {
            found.push("'1' with a plural unit");
        }
    }
    for triple in words.windows(3) {
        if triple[0] == "1" && triple[1] == "code" && triple[2].starts_with("lines") {
            found.push("'1' with a plural unit");
        }
    }
    if words
        .iter()
        .any(|word| ["-0", "+0", "−0"].contains(&word.trim_end_matches([',', '.', ';', ')'])))
    {
        found.push("signed zero");
    }
    for value in VALUES {
        let starts_sentence = text.starts_with(&format!("{value} "))
            || [". ", "! ", "? "]
                .iter()
                .any(|stop| text.contains(&format!("{stop}{value} ")));
        let before_count = words
            .windows(2)
            .any(|pair| pair[0] == value && pair[1].starts_with(|c: char| c.is_ascii_digit()));
        let before_purpose = words
            .windows(2)
            .any(|pair| pair[0] == value && ["production", "test", "other"].contains(&pair[1]));
        if starts_sentence || before_count || before_purpose {
            found.push("value used as a word: single-quote it");
        }
    }
    found
}

#[test]
fn human_output_follows_the_wording_rules() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let mut report = Vec::new();
    for args in scenarios() {
        let output = common::locmetry()
            .current_dir(temp.path())
            .env("COLUMNS", "120")
            .args(&args)
            .output()
            .unwrap();
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for line in prose(&text) {
            for violation in violations(line) {
                report.push(format!("{}\n    {line}\n    {violation}", args.join(" ")));
            }
        }
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}

#[test]
fn the_guard_flags_each_rule_and_spares_compliant_text() {
    for (line, rule) in [
        ("Lines = Code; Test/prod = Test / Production", "';'"),
        ("Use --files --output - to list them.", "flag"),
        ("Folder \"src\" appears.", "double quotes"),
        ("Counts: 1 code, 0 comments, 1 lines", "plural"),
        ("matched +1, leaving -0", "signed zero"),
        ("Production from build modules: npm 12 code lines.", "value"),
        ("Rust production includes 3 code lines.", "value"),
    ] {
        assert!(
            violations(line).iter().any(|v| v.contains(rule)),
            "{line}: {rule}"
        );
    }
    for line in [
        "Lines = Code + Comments + Blanks · Test/prod = Test / Production (Code)",
        "Production in 'Rust' includes 3 code lines. 'locmetry --files --output -' names them.",
        "Not shown (languages): Python and Rust (7 code lines).",
        "Each file's rule is in 'locmetry --files --output -'.",
        "Production from build modules: 12 code lines in 1 'npm' module.",
    ] {
        assert!(
            violations(line).is_empty(),
            "{line}: {:?}",
            violations(line)
        );
    }
}
