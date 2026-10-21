use super::*;
use std::ffi::OsString;

/// An absolute path on the running platform, which needs a drive on Windows.
fn absolute(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

fn file_with(vars: &[(&str, &Path)]) -> Option<PathBuf> {
    defaults_file(|name| {
        vars.iter()
            .find(|(var, _)| *var == name)
            .map(|(_, value)| OsString::from(value))
    })
}

#[test]
fn an_absolute_xdg_config_home_wins_on_every_platform() {
    let xdg = absolute("xdg");
    let file = file_with(&[
        ("XDG_CONFIG_HOME", &xdg),
        ("HOME", &absolute("home")),
        ("APPDATA", &absolute("appdata")),
    ]);
    assert_eq!(file, Some(xdg.join("locmetry").join("config.toml")));
}

/// Windows has no `HOME` by default; Tokei, cloc and uv keep user
/// configuration in `%APPDATA%` there (docs/proposals/archive/windows-support.md).
#[test]
fn without_xdg_config_home_the_platform_config_folder_applies() {
    let (home, appdata) = (absolute("home"), absolute("appdata"));
    let file = file_with(&[
        ("XDG_CONFIG_HOME", Path::new("relative")),
        ("HOME", &home),
        ("APPDATA", &appdata),
    ]);
    let base = if cfg!(windows) {
        appdata
    } else {
        home.join(".config")
    };
    assert_eq!(file, Some(base.join("locmetry").join("config.toml")));
    let other = if cfg!(windows) { "HOME" } else { "APPDATA" };
    assert_eq!(file_with(&[(other, &absolute("other"))]), None);
}
