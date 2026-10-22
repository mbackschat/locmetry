//! Errors about an unusable input name it as the user typed it, by the human
//! output wording rules (docs/help-design.md, Human Output Wording). The 0.9.0
//! desk review (B7) found canonical absolute paths, inputs left unnamed and a
//! clap tip that pointed at an unrelated option.
use crate::common::{self, write_all};
use std::path::Path;

#[test]
fn errors_name_the_input_as_typed() {
    let temp = tempfile::tempdir().unwrap();
    write_all(
        temp.path(),
        &[
            ("proj/src/lib.rs", "pub fn a() {}\n"),
            ("a/x/main.py", "print(1)\n"),
            ("b/x/main.py", "print(2)\n"),
            ("cfg/missing.toml", "version = 1\nroots = [\"missing\"]\n"),
            ("cfg/ok.toml", "version = 1\nroots = [\"../proj\"]\n"),
            (
                "cfg/folders.toml",
                "version = 1\nproject = \"../proj\"\nfolders = [\"src/lib.rs\"]\n",
            ),
        ],
    );
    // A file's roots are joined to its folder, so Windows shows 'cfg\missing'.
    let joined = format!(
        "Workspace member root '{}' does not exist",
        Path::new("cfg").join("missing").display()
    );
    for (args, message) in [
        (
            vec!["workspace", "--workspace-file", "cfg/missing.toml"],
            joined.as_str(),
        ),
        (
            vec!["workspace", "--root", "proj/src/lib.rs"],
            "Workspace member root 'proj/src/lib.rs' must be a directory",
        ),
        (
            vec!["workspace", "--workspace-file", "proj"],
            "Cannot read workspace file 'proj'",
        ),
        (
            vec!["workspace", "--workspace-file", "cfg/folders.toml"],
            "Workspace folder 'src/lib.rs'",
        ),
        (
            vec!["workspace", "--root", "proj", "--root", "./proj"],
            "Workspace roots 'proj' and './proj' name the same folder",
        ),
        (
            vec!["workspace", "--root", "proj", "--root", "proj"],
            "Workspace root 'proj' is given twice",
        ),
        (
            vec!["workspace", "--root", "proj", "--root", "proj/src"],
            "Workspace member 'proj/src' lies inside member 'proj'",
        ),
        (
            vec!["workspace", "--root", "a/x", "--root", "b/x"],
            "Workspace members 'a/x' and 'b/x' share the name 'x'",
        ),
        (
            vec!["--root", "proj", "--subfolder", "src/lib.rs"],
            "Subfolder 'src/lib.rs' must be a directory",
        ),
        (
            vec!["--root", "proj", "--output", "proj/out.json"],
            "Output 'proj/out.json' must be outside the read-only scan root",
        ),
        (
            vec![
                "workspace",
                "--workspace-file",
                "cfg/ok.toml",
                "--output",
                "cfg/ok.toml",
            ],
            "Output 'cfg/ok.toml' would overwrite the configuration",
        ),
    ] {
        let output = common::locmetry()
            .current_dir(temp.path())
            .args(&args)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {stderr}");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
        let scratch = temp.path().file_name().unwrap().to_string_lossy();
        assert!(!stderr.contains(&*scratch), "{args:?}: {stderr}");
    }
}

/// Clap suggested `--show-bars` for `workspace --show-folders`, and similar
/// unrelated options or a `--` escape for other root options given to a
/// command (0.9.0 desk review, B7).
#[test]
fn root_options_given_to_a_command_name_where_they_apply() {
    let temp = tempfile::tempdir().unwrap();
    for (args, flag, command) in [
        (
            vec!["workspace", "--root", ".", "--show-folders"],
            "--show-folders",
            "workspace",
        ),
        (
            vec!["compare", "a.json", "b.json", "--table-purpose", "test"],
            "--table-purpose",
            "compare",
        ),
        (
            vec!["explain", "x.rs", "--guide", "reports"],
            "--guide",
            "explain",
        ),
    ] {
        let output = common::locmetry()
            .current_dir(temp.path())
            .env("NO_COLOR", "1")
            .args(&args)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {stderr}");
        assert!(
            stderr.contains(&format!(
                "'{flag}' is only available without a command, not with '{command}'"
            )),
            "{args:?}: {stderr}"
        );
        assert!(!stderr.contains("similar argument"), "{args:?}: {stderr}");
        assert!(!stderr.contains("as a value"), "{args:?}: {stderr}");
    }
}
