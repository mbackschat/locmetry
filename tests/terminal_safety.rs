use serde_json::Value;
use std::{fs, process::Command};

#[test]
fn table_and_unclaimed_note_flatten_path_controls_without_changing_json() {
    let root = tempfile::tempdir().unwrap();
    let names = ["a\x1b[31mred", "b\tcolumn", "c\nrow", "d\rreturn"];
    for name in names {
        fs::create_dir(root.path().join(name)).unwrap();
        fs::write(root.path().join(name).join("item.rs"), "fn item() {}\n").unwrap();
    }
    for extra in [vec![], vec!["--folder-rows", "10"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .args([
                "--root",
                root.path().to_str().unwrap(),
                "--no-user-config",
                "--format",
                "table",
            ])
            .args(extra)
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout).unwrap();
        let (tables, note) = text.split_once("Unclaimed source:").unwrap();
        for part in [tables, note] {
            assert!(
                !part.chars().any(|c| c.is_control() && c != '\n'),
                "{part:?}"
            );
            for label in ["a [31mred", "b column", "c row", "d return"] {
                assert!(part.contains(label), "missing {label:?} in {part:?}");
            }
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .args([
            "--root",
            root.path().to_str().unwrap(),
            "--no-user-config",
            "--files",
            "--output",
            "-",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    for name in names {
        assert!(json["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["path"] == format!("{name}/item.rs")));
        assert!(json["summary"]["unclassified_folders"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["folder"] == name));
    }
}

#[test]
fn coverage_notes_and_diagnostics_flatten_controls_but_preserve_json_evidence() {
    let root = tempfile::tempdir().unwrap();
    let notebook = "n\x1b[31mb\nline.ipynb";
    let module = "vendor\tpart\x1b[31mred";
    fs::write(root.path().join(notebook), "not json").unwrap();
    fs::create_dir(root.path().join(module)).unwrap();
    fs::write(root.path().join(module).join("item.rs"), "fn item() {}\n").unwrap();
    fs::write(
        root.path().join(".gitmodules"),
        format!("[submodule \"example\"]\npath = \"{module}\"\n"),
    )
    .unwrap();
    for format in ["table", "json"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_locmetry"));
        command.args([
            "--root",
            root.path().to_str().unwrap(),
            "--no-user-config",
            "--format",
            format,
        ]);
        if format == "json" {
            command.args(["--output", "-"]);
        }
        let output = command.env("NO_COLOR", "1").output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("n [31mb line.ipynb"), "{stderr:?}");
        assert_eq!(stderr.lines().count(), 1, "{stderr:?}");
        assert!(!stderr.contains('\x1b'), "{stderr:?}");
        if format == "table" {
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(
                text.contains("n [31mb line.ipynb: invalid_notebook"),
                "{text:?}"
            );
            assert!(
                text.contains("Git submodules: vendor part [31mred"),
                "{text:?}"
            );
            assert!(
                !text.chars().any(|c| c.is_control() && c != '\n'),
                "{text:?}"
            );
        } else {
            let json: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                json["metadata"]["coverage"]["parse_failures"][0]["path"],
                notebook
            );
            assert_eq!(json["summary"]["submodules"][0]["path"], module);
        }
    }
}
