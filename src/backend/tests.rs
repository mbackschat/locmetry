use super::*;

#[test]
fn selected_file_statistics_match_tokei_including_embedded_languages() {
    let temp = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(temp.path()).unwrap();
    let mut paths = Vec::new();
    for (name, source) in [
        ("a.rs", "// comment\n\nfn main() {}\n"),
        ("b.rs", "fn other() {}\n"),
        ("tool", "#!/usr/bin/env python\nprint(1)\n"),
        (
            "page.html",
            "<script>\nconst x = 1;\n</script>\n<style>\na { color: red; }\n</style>\n",
        ),
        ("second.html", "<script>\nconst y = 2;\n</script>\n"),
        ("README.md", "# Example\n\n```rust\nfn item() {}\n```\n"),
        ("unknown.zzz", "unrecognized\n"),
    ] {
        let path = root.join(name);
        std::fs::write(&path, source).unwrap();
        paths.push(path);
    }
    let config = tokei::Config {
        hidden: Some(true),
        no_ignore: Some(true),
        ..Default::default()
    };
    let mut expected = tokei::Languages::new();
    expected.get_statistics(&paths, &[], &config);
    let actual = count_selected(&root, &paths, &config, &mut Coverage::default()).unwrap();
    let ordered = |mut languages: tokei::Languages| {
        for language in languages.values_mut() {
            language.reports.sort_by(|a, b| a.name.cmp(&b.name));
            for reports in language.children.values_mut() {
                reports.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
        serde_json::to_value(languages).unwrap()
    };
    assert_eq!(ordered(actual), ordered(expected));
}

#[test]
fn a_selected_file_removed_before_counting_marks_counts_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("removed.rs");
    std::fs::write(&path, "fn main() {}\n").unwrap();
    let policy = Policy::load_layers(None, None).unwrap();
    let selected = select_paths(temp.path(), policy.scope(), None).unwrap();
    std::fs::remove_file(path).unwrap();
    let languages = count_selected(
        temp.path(),
        &selected.paths,
        &tokei::Config::default(),
        &mut Coverage::default(),
    )
    .unwrap();
    assert!(languages
        .values()
        .all(|language| language.reports.is_empty()));
    assert!(languages.values().any(|language| language.inaccurate));
}

#[test]
fn counting_does_not_expand_a_selected_file_replaced_by_a_directory() {
    for filename in ["chosen.rs", "chosen.py"] {
        let temp = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(temp.path()).unwrap();
        let path = root.join(filename);
        std::fs::write(&path, "original\n").unwrap();
        let policy = Policy::load_layers(None, None).unwrap();
        let selected = select_paths(&root, policy.scope(), None).unwrap();
        assert_eq!(selected.paths.as_slice(), std::slice::from_ref(&path));
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("unselected.rs"), "fn added() {}\n").unwrap();
        let languages = count_selected(
            &root,
            &selected.paths,
            &tokei::Config::default(),
            &mut Coverage::default(),
        )
        .unwrap();
        assert!(languages
            .values()
            .all(|language| language.reports.is_empty()));
        assert!(languages.values().any(|language| language.inaccurate));
    }
}
