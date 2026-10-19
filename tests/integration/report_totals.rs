use crate::common;
use serde_json::Value;
use std::{fs, path::Path};

fn run(root: &Path, args: &[&str]) -> String {
    let output = common::locmetry()
        .arg("--no-user-config")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn human_count_tables_expose_lines_before_code() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("src/lib.rs"),
        "// comment\n\nfn item() {}\n",
    )
    .unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname = 'example'\n",
    )
    .unwrap();
    for extra in [vec!["--folders"], vec!["--folder-rows", "8"]] {
        let mut args = vec!["--format", "table", "--categories"];
        args.extend(extra);
        let table = run(root.path(), &args);
        for heading in ["Language", "Folder", "Category"] {
            let header = table
                .lines()
                .find(|line| line.trim_start().starts_with(heading))
                .unwrap();
            assert!(header.contains("Lines"), "{header}");
            assert!(header.find("Lines") < header.find("Code"), "{header}");
        }
        let rust = table
            .lines()
            .find(|line| line.trim_start().starts_with("Rust "))
            .unwrap();
        let cells: Vec<_> = rust.split_whitespace().filter(|c| *c != "│").collect();
        assert_eq!(&cells[..6], &["Rust", "1", "3", "1", "1", "1"]);
        let folder = table
            .lines()
            .find(|line| line.trim_start().starts_with("src "))
            .unwrap();
        let cells: Vec<_> = folder.split_whitespace().filter(|c| *c != "│").collect();
        assert_eq!(&cells[..3], &["src", "3", "1"]);
        let category = table
            .lines()
            .find(|line| line.trim_start().starts_with("production:implementation "))
            .unwrap();
        let cells: Vec<_> = category.split_whitespace().collect();
        assert_eq!(cells, ["production:implementation", "1", "3", "1"]);
    }
    let markdown = run(root.path(), &["--format", "markdown"]);
    assert!(markdown.contains("| cargo | 1 | 3 | 1 |"), "{markdown}");
    assert!(
        markdown.contains("| (scan root) | cargo | 3 | 1 |"),
        "{markdown}"
    );
    for heading in [
        "Purpose",
        "Category",
        "Kind:purpose",
        "Language",
        "Folder",
        "Path",
        "Build system",
        "Module",
    ] {
        let header = markdown
            .lines()
            .find(|line| line.starts_with(&format!("| {heading} |")))
            .unwrap();
        assert!(header.contains("Lines"), "{header}");
        assert!(header.find("Lines") < header.find("Code"), "{header}");
    }
}

#[test]
fn human_reports_end_with_the_recorded_measurement_time() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.rs"), "fn main() {}\n").unwrap();
    let before: Value = serde_json::from_str(&run(root.path(), &["--format", "json"])).unwrap();
    for format in ["table", "markdown"] {
        let text = run(root.path(), &["--format", format]);
        let footer = text.lines().last().unwrap().trim();
        assert!(footer.starts_with("Measured at: "), "{footer}");
        assert!(footer.ends_with(" UTC"), "{footer}");
        let recorded = chrono::NaiveDateTime::parse_from_str(
            footer.strip_prefix("Measured at: ").unwrap(),
            "%Y-%m-%d %H:%M:%S UTC",
        )
        .unwrap()
        .and_utc()
        .timestamp() as u64;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!((before["metadata"]["measured_at_unix_seconds"]
            .as_u64()
            .unwrap()..=now)
            .contains(&recorded));
    }
    assert!(
        before["metadata"]["measured_at_unix_seconds"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(before["summary"]["source"]["lines"], 1);
}

#[test]
fn adaptive_and_subfolder_lines_partition_source_including_remainder_rows() {
    let root = tempfile::tempdir().unwrap();
    for path in [
        "main.rs",
        "packages/a/a.rs",
        "packages/b/b.rs",
        "packages/c/c.rs",
        "packages/d/d.rs",
    ] {
        let path = root.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "// comment\n\nfn item() {}\n").unwrap();
    }
    fs::write(root.path().join("packages/README.md"), "# Not source\n").unwrap();
    for (extra, expected_lines, expected_code) in
        [(vec![], 15, 5), (vec!["--subfolder", "packages"], 12, 4)]
    {
        let mut args = vec!["--format", "table", "--folder-rows", "4"];
        args.extend(extra);
        let text = run(root.path(), &args);
        let folder_table = text
            .split("\n Folder ")
            .nth(1)
            .unwrap()
            .split("\n Production +")
            .next()
            .unwrap();
        let mut lines = 0;
        let mut code = 0;
        for row in folder_table.lines().skip(1) {
            let cells: Vec<_> = row.split_whitespace().filter(|c| *c != "│").collect();
            if cells.len() < 7 || cells[0] == "Total" {
                continue;
            }
            let numbers = &cells[cells.len() - 6..];
            lines += numbers[0].parse::<u64>().unwrap();
            code += numbers[1].parse::<u64>().unwrap();
        }
        assert_eq!((lines, code), (expected_lines, expected_code), "{text}");
    }
}

#[test]
fn recorded_dates_format_in_utc_without_overflow() {
    for (seconds, expected) in [
        (0, "1970-01-01 00:00:00 UTC"),
        (951_827_696, "2000-02-29 12:34:56 UTC"),
        (1_735_689_599, "2024-12-31 23:59:59 UTC"),
    ] {
        assert_eq!(locmetry::measurement_time(seconds), expected);
    }
    assert!(
        locmetry::measurement_time(u64::MAX).contains("outside the supported UTC calendar range")
    );
}

#[test]
fn workspace_reports_retain_total_lines_in_repository_and_language_grids() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["first", "second"] {
        let root = temp.path().join(name);
        fs::create_dir(&root).unwrap();
        fs::write(root.join("main.rs"), "// comment\n\nfn main() {}\n").unwrap();
    }
    for format in ["table", "markdown", "json"] {
        let output = common::locmetry()
            .current_dir(temp.path())
            .args([
                "workspace",
                "--no-user-config",
                "--root",
                "first",
                "--root",
                "second",
                "--format",
                format,
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        match format {
            "json" => {
                let value: Value = serde_json::from_str(&text).unwrap();
                assert_eq!(value["summary"]["source"]["lines"], 6);
                assert_eq!(value["summary"]["members"]["first"]["source"]["lines"], 3);
                assert!(
                    value["metadata"]["measured_at_unix_seconds"]
                        .as_u64()
                        .unwrap()
                        > 0
                );
            }
            "markdown" => {
                assert!(
                    text.contains("| first | 3 | 1 | 0 | 0 | 1 | - | 50.0% |"),
                    "{text}"
                );
                assert!(text.contains("| Rust | 3 | 1 | 3 | 1 | 6 | 2 |"), "{text}");
                assert!(text.lines().last().unwrap().starts_with("Measured at: "));
            }
            "table" => {
                let table = text.split("\n Repository").nth(1).unwrap();
                let row = table
                    .lines()
                    .find(|line| line.trim_start().starts_with("first "))
                    .unwrap();
                let cells: Vec<_> = row.split_whitespace().filter(|c| *c != "│").collect();
                assert_eq!(&cells[..3], ["first", "3", "1"]);
                assert!(text.lines().last().unwrap().starts_with("Measured at: "));
            }
            _ => unreachable!(),
        }
    }
}
