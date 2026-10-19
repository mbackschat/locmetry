use locmetry::git_submodules;
use std::fs;

#[test]
fn submodule_paths_decode_git_config_values_in_submodule_sections_only() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join(".gitmodules"),
        r#"
path = outside
[include]
path = unrelated.conf
[submodule "with ] and \" in name"]
    path = "vendor/a # b" ; trailing comment
[submodule "escaped"]
    path = "vendor/a\"b\\c"
[SUBMODULE "case"] PaTh = vendor/case # comment
[submodule.continued]
    path = vendor/\
continued
[other "not a submodule"]
    path = unrelated
"#,
    )
    .unwrap();
    assert_eq!(
        git_submodules(temp.path()),
        [
            "vendor/a # b",
            "vendor/a\"b\\c",
            "vendor/case",
            "vendor/continued"
        ]
    );
}

#[test]
fn malformed_submodule_values_do_not_become_scope_hints() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join(".gitmodules"),
        r#"
[submodule "bad"]
path = "unclosed
path = "bad\q"
path =
[submodule]
path = no-subsection
[submodule "ok"]
path = " leading and trailing "
[submodule "bad" "header"]
path = false-positive
"#,
    )
    .unwrap();
    assert_eq!(git_submodules(temp.path()), [" leading and trailing "]);
}
