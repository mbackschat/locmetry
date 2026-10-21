//! Behavior that differs between operating systems or the shells users run.

use crate::common::{json, write};
use crate::compare::{compare, measure, save};
use std::{fs, path::Path};

/// Locks the displayed root to the form users type: on Windows,
/// `canonicalize` returns verbatim `\\?\C:\…` paths
/// (https://doc.rust-lang.org/std/fs/fn.canonicalize.html#platform-specific-behavior).
#[test]
fn report_roots_are_paths_as_users_write_them() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "main.py", "print(1)\n");
    let report = measure(temp.path());
    let root = report["metadata"]["root"].as_str().unwrap();
    assert!(!root.starts_with(r"\\?\"), "{root}");
    assert!(Path::new(root).is_absolute(), "{root}");
}

/// Windows PowerShell 5.1 writes UTF-16LE with a byte-order mark for `>`
/// and UTF-8 with one for `Out-File -Encoding utf8`
/// (https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_character_encoding).
#[test]
fn compare_reads_reports_saved_by_windows_powershell() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let plain = save(temp.path(), "before.json", &measure(&root));
    let text = fs::read_to_string(&plain).unwrap();
    let utf16: Vec<u8> = [0xFF, 0xFE]
        .into_iter()
        .chain(text.encode_utf16().flat_map(u16::to_le_bytes))
        .collect();
    let utf8 = [b"\xEF\xBB\xBF".as_slice(), text.as_bytes()].concat();
    for (name, bytes) in [("utf16.json", utf16), ("utf8-bom.json", utf8)] {
        let path = temp.path().join(name);
        fs::write(&path, bytes).unwrap();
        let comparison = json(&compare(&plain, &path, &["--output", "-"]), 0);
        assert_eq!(
            comparison["summary"]["source"]["delta"]["code"], 0,
            "{name}"
        );
    }
}
