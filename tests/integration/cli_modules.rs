use crate::cli::*;
use serde_json::{json, Value};

fn classifications(document: &Value) -> Vec<(String, String, String)> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap().into(),
                file["purpose"].as_str().unwrap().into(),
                file["rule"].as_str().unwrap().into(),
            )
        })
        .collect()
}

fn triple(path: &str, purpose: &str, rule: &str) -> (String, String, String) {
    (path.into(), purpose.into(), rule.into())
}

/// Build descriptors declare modules, and each build system's source layout
/// inside a module is production; tests, build scripts, and anything outside
/// a declared source root keep their own classification.
#[test]
fn declared_modules_make_their_source_production() {
    let temp = project(&[
        ("packages/core/package.json", "{}\n"),
        ("packages/core/src/a.ts", "let a = 1;\n"),
        ("packages/core/src/a.test.ts", "let b = 1;\n"),
        ("packages/core/scripts/release.ts", "let c = 1;\n"),
        ("app/build.gradle.kts", "plugins {}\n"),
        ("app/src/main/kotlin/App.kt", "fun main() {}\n"),
        ("app/src/commonMain/kotlin/Shared.kt", "val x = 1\n"),
        ("app/src/test/kotlin/AppTest.kt", "fun t() {}\n"),
        ("legacy/pom.xml", "<project/>\n"),
        ("legacy/src/main/java/A.java", "class A {}\n"),
        ("crates/util/Cargo.toml", "[package]\n"),
        ("crates/util/src/lib.rs", "fn util() {}\n"),
        ("crates/util/src/parse/tests.rs", "fn case() {}\n"),
        ("svc/go.mod", "module svc\n"),
        ("svc/main.go", "package main\n"),
        ("svc/internal/store.go", "package internal\n"),
        ("svc/main_test.go", "package main\n"),
        ("docs/probe.js", "let d = 1;\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| {
            !path.ends_with(".json")
                && !path.ends_with(".xml")
                && !path.ends_with(".mod")
                && !path.ends_with(".toml")
        })
        .collect();
    assert_eq!(
        source,
        [
            triple("app/build.gradle.kts", "production", "builtin:build-script"),
            triple(
                "app/src/commonMain/kotlin/Shared.kt",
                "production",
                "builtin:gradle-module"
            ),
            triple(
                "app/src/main/kotlin/App.kt",
                "production",
                "builtin:gradle-module"
            ),
            triple(
                "app/src/test/kotlin/AppTest.kt",
                "test",
                "builtin:test-directory"
            ),
            triple(
                "crates/util/src/lib.rs",
                "production",
                "builtin:cargo-module"
            ),
            triple(
                "crates/util/src/parse/tests.rs",
                "test",
                "builtin:rust-test-module"
            ),
            triple("docs/probe.js", "other", "fallback"),
            triple(
                "legacy/src/main/java/A.java",
                "production",
                "builtin:maven-module"
            ),
            triple("packages/core/scripts/release.ts", "other", "fallback"),
            triple(
                "packages/core/src/a.test.ts",
                "test",
                "builtin:js-test-file"
            ),
            triple("packages/core/src/a.ts", "production", "builtin:npm-module"),
            triple("svc/internal/store.go", "production", "builtin:go-module"),
            triple("svc/main.go", "production", "builtin:go-module"),
            triple("svc/main_test.go", "test", "builtin:go-test-file"),
        ]
    );
    assert_eq!(
        document["metadata"]["modules"],
        json!([
            {"path": "app", "detector": "gradle"},
            {"path": "crates/util", "detector": "cargo"},
            {"path": "legacy", "detector": "maven"},
            {"path": "packages/core", "detector": "npm"},
            {"path": "svc", "detector": "go"}
        ])
    );
}

/// The innermost module decides: an outer module's layout never claims files
/// of a module nested inside it, while build systems sharing one folder
/// combine their layouts.
#[test]
fn the_innermost_module_decides_nested_layouts() {
    let temp = project(&[
        ("package.json", "{}\n"),
        ("src/index.ts", "let a = 1;\n"),
        ("src/plugins/fancy/package.json", "{}\n"),
        ("src/plugins/fancy/src/fancy.ts", "let b = 1;\n"),
        ("src/plugins/fancy/scripts/build.ts", "let c = 1;\n"),
        ("app/build.gradle.kts", "plugins {}\n"),
        ("app/package.json", "{}\n"),
        ("app/src/main/kotlin/App.kt", "fun main() {}\n"),
        ("app/src/web/ui.ts", "let d = 1;\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| path.ends_with(".ts") || path.ends_with(".kt"))
        .collect();
    assert_eq!(
        source,
        [
            triple(
                "app/src/main/kotlin/App.kt",
                "production",
                "builtin:gradle-module"
            ),
            triple("app/src/web/ui.ts", "production", "builtin:npm-module"),
            triple("src/index.ts", "production", "builtin:npm-module"),
            triple("src/plugins/fancy/scripts/build.ts", "other", "fallback"),
            triple(
                "src/plugins/fancy/src/fancy.ts",
                "production",
                "builtin:npm-module"
            ),
        ]
    );
}

/// Locks the same-folder tie-break (docs/PRODUCT.md, Module Detection): when
/// layouts of build systems in one folder overlap, the detector whose ID sorts
/// first decides, so provenance never depends on file or detector order.
#[test]
fn overlapping_layouts_in_one_folder_resolve_in_detector_id_order() {
    let temp = project(&[
        ("package.json", "{}\n"),
        ("Cargo.toml", "[package]\n"),
        ("build.gradle.kts", "plugins {}\n"),
        ("src/lib.rs", "fn a() {}\n"),
        ("src/main/kotlin/App.kt", "fun main() {}\n"),
        ("src/web/ui.ts", "let d = 1;\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| path.starts_with("src/"))
        .collect();
    assert_eq!(
        source,
        [
            triple("src/lib.rs", "production", "builtin:cargo-module"),
            triple(
                "src/main/kotlin/App.kt",
                "production",
                "builtin:cargo-module"
            ),
            triple("src/web/ui.ts", "production", "builtin:cargo-module"),
        ]
    );
}

/// Locks determinism: module detection depends on the set of selected files,
/// never on the order a file system or caller lists them.
#[test]
fn module_detection_ignores_the_order_files_are_listed_in() {
    let files = [
        "web/package.json",
        "web/src/a.ts",
        "Cargo.toml",
        "app/build.gradle.kts",
        "app/package.json",
        "lib/pyproject.toml",
        "lib/shop/__init__.py",
    ];
    let mut reversed = files;
    reversed.reverse();
    let mut rotated = files;
    rotated.rotate_left(3);
    let detected = locmetry::detect_modules(&files);
    assert_eq!(locmetry::detect_modules(&reversed), detected);
    assert_eq!(locmetry::detect_modules(&rotated), detected);
}

/// Locks determinism end to end: the same files measured in trees created in
/// opposite orders give identical reports apart from the root path and the
/// measurement time, so directory enumeration order never leaks into results.
#[test]
fn identical_trees_give_identical_reports_regardless_of_creation_order() {
    let mut files = vec![
        ("package.json", "{}\n"),
        ("src/index.ts", "let a = 1;\n"),
        ("src/index.test.ts", "let t = 1;\n"),
        ("app/build.gradle.kts", "plugins {}\n"),
        ("app/package.json", "{}\n"),
        ("app/src/main/kotlin/App.kt", "fun main() {}\n"),
        ("scripts/z.ts", "let z = 1;\n"),
        ("scripts/a.ts", "let y = 1;\n"),
        ("docs/guide.md", "# Guide\n"),
        ("data/b.json", "{}\n"),
        ("notes.unknownext", "x\n"),
    ];
    let measure = |files: &[(&str, &str)]| {
        let temp = project(files);
        let (success, mut document, stderr) = report(&temp.path().join("project"), &[]);
        assert!(success, "{stderr}");
        let metadata = document["metadata"].as_object_mut().unwrap();
        metadata.remove("root");
        metadata.remove("measured_at_unix_seconds");
        serde_json::to_string(&document).unwrap()
    };
    let forward = measure(&files);
    files.reverse();
    assert_eq!(measure(&files), forward);
}

/// Python modules follow setuptools' discovery: the `src` layout, or flat
/// top-level packages with `__init__.py`, never names such as `scripts`.
#[test]
fn python_modules_follow_setuptools_package_discovery() {
    let temp = project(&[
        ("flat/pyproject.toml", "[project]\n"),
        ("flat/shop/__init__.py", "\n"),
        ("flat/shop/cart.py", "total = 0\n"),
        ("flat/scripts/__init__.py", "\n"),
        ("flat/scripts/seed.py", "seed = 1\n"),
        ("flat/tests/test_cart.py", "def test(): pass\n"),
        ("flat/notebook.py", "x = 1\n"),
        ("layered/setup.py", "setup()\n"),
        ("layered/src/pkg/core.py", "core = 1\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| {
            path.ends_with(".py") && !path.ends_with("__init__.py") && !path.ends_with("setup.py")
        })
        .collect();
    assert_eq!(
        source,
        [
            triple("flat/notebook.py", "other", "fallback"),
            triple("flat/scripts/seed.py", "other", "fallback"),
            triple("flat/shop/cart.py", "production", "builtin:python-module"),
            triple("flat/tests/test_cart.py", "test", "builtin:test-directory"),
            triple(
                "layered/src/pkg/core.py",
                "production",
                "builtin:python-module"
            ),
        ]
    );
}

/// Go modules claim what `./...` builds: `go help packages` excludes
/// `vendor` and `testdata` folders and names starting with `_` or `.`, at any
/// depth, so vendored dependencies and test inputs stay open for review.
#[test]
fn go_modules_claim_only_what_the_go_tool_builds() {
    let temp = project(&[
        ("go.mod", "module svc\n"),
        ("main.go", "package main\n"),
        ("internal/store/store.go", "package store\n"),
        ("vendor/github.com/lib/pq/conn.go", "package pq\n"),
        ("internal/vendor/copied.go", "package vendor\n"),
        ("internal/store/testdata/fixture.go", "package fixture\n"),
        ("_examples/demo/demo.go", "package main\n"),
        (".tools/gen.go", "package main\n"),
        ("internal/_scratch.go", "package internal\n"),
        ("internal/.hidden.go", "package internal\n"),
        ("internal/vendored/ok.go", "package vendored\n"),
        ("internal/my_data/ok.go", "package my_data\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| path.ends_with(".go"))
        .collect();
    assert_eq!(
        source,
        [
            triple(".tools/gen.go", "other", "fallback"),
            triple("_examples/demo/demo.go", "other", "fallback"),
            triple("internal/.hidden.go", "other", "fallback"),
            triple("internal/_scratch.go", "other", "fallback"),
            triple("internal/my_data/ok.go", "production", "builtin:go-module"),
            triple("internal/store/store.go", "production", "builtin:go-module"),
            triple("internal/store/testdata/fixture.go", "other", "fallback"),
            triple("internal/vendor/copied.go", "other", "fallback"),
            triple("internal/vendored/ok.go", "production", "builtin:go-module"),
            triple("main.go", "production", "builtin:go-module"),
            triple("vendor/github.com/lib/pq/conn.go", "other", "fallback"),
        ]
    );
}

/// Swift packages: `Sources/` is production, and test targets in `Tests/` or
/// Xcode-style `<Name>Tests/` folders are tests wherever they live.
#[test]
fn swift_packages_and_test_targets_are_recognized() {
    let temp = project(&[
        ("Package.swift", "// swift-tools-version:5.9\n"),
        ("Sources/App/main.swift", "print(1)\n"),
        ("Tests/AppTests/AppTests.swift", "import XCTest\n"),
        ("Plugins/Gen/plugin.swift", "let p = 1\n"),
        ("ios/MyAppTests/LoginTests.swift", "import XCTest\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| path != "Package.swift")
        .collect();
    assert_eq!(
        source,
        [
            triple("Plugins/Gen/plugin.swift", "other", "fallback"),
            triple(
                "Sources/App/main.swift",
                "production",
                "builtin:swift-module"
            ),
            triple(
                "Tests/AppTests/AppTests.swift",
                "test",
                "builtin:swift-test-target"
            ),
            triple(
                "ios/MyAppTests/LoginTests.swift",
                "test",
                "builtin:swift-test-target"
            ),
        ]
    );
}

/// Dart and Flutter packages: `lib/` and `bin/` are production as the pub
/// package layout defines them (https://dart.dev/tools/pub/package-layout),
/// `test/` and Flutter's `integration_test/` are tests, and a local package
/// nested in an app is its own module.
#[test]
fn dart_packages_and_flutter_integration_tests_are_recognized() {
    let temp = project(&[
        ("app/pubspec.yaml", "name: app\n"),
        ("app/lib/main.dart", "void main() {}\n"),
        ("app/lib/src/model.dart", "class Model {}\n"),
        ("app/bin/seed.dart", "void main() {}\n"),
        ("app/test/model_test.dart", "void main() {}\n"),
        ("app/integration_test/login_test.dart", "void main() {}\n"),
        ("app/tool/generate.dart", "void main() {}\n"),
        ("app/packages/ui/pubspec.yaml", "name: ui\n"),
        ("app/packages/ui/lib/button.dart", "class Button {}\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| !path.ends_with("pubspec.yaml"))
        .collect();
    assert_eq!(
        source,
        [
            triple("app/bin/seed.dart", "production", "builtin:dart-module"),
            triple(
                "app/integration_test/login_test.dart",
                "test",
                "builtin:flutter-integration-test"
            ),
            triple("app/lib/main.dart", "production", "builtin:dart-module"),
            triple(
                "app/lib/src/model.dart",
                "production",
                "builtin:dart-module"
            ),
            triple(
                "app/packages/ui/lib/button.dart",
                "production",
                "builtin:dart-module"
            ),
            triple("app/test/model_test.dart", "test", "builtin:test-directory"),
            triple("app/tool/generate.dart", "other", "fallback"),
        ]
    );
    assert_eq!(
        document["metadata"]["modules"],
        json!([
            {"path": "app", "detector": "dart"},
            {"path": "app/packages/ui", "detector": "dart"}
        ])
    );
}

/// Lean 4 Lake packages keep their sources at the package root, so every
/// `.lean` file below a lakefile is production, except the lakefile itself,
/// test libraries named like mathlib4's `MathlibTest/`, and the `.lake/`
/// build and dependency folder.
#[test]
fn lake_packages_and_test_libraries_are_recognized() {
    let temp = project(&[
        (
            "lakefile.lean",
            "import Lake\nopen Lake DSL\npackage demo\n",
        ),
        ("Demo.lean", "import Demo.Core\n"),
        ("Demo/Core.lean", "def answer : Nat := 42\n"),
        ("DemoTest/Basic.lean", "example : 1 + 1 = 2 := rfl\n"),
        (".lake/packages/dep/lakefile.toml", "name = \"dep\"\n"),
        (".lake/packages/dep/Dep.lean", "def dep : Nat := 1\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        classifications(&document),
        [
            triple("Demo.lean", "production", "builtin:lake-module"),
            triple("Demo/Core.lean", "production", "builtin:lake-module"),
            triple("DemoTest/Basic.lean", "test", "builtin:lean-test-library"),
            triple("lakefile.lean", "production", "builtin:build-script"),
        ]
    );
    assert_eq!(
        document["metadata"]["modules"],
        json!([{"path": ".", "detector": "lake"}])
    );
}

/// Project knowledge beats generic detection, and detection can be disabled.
#[test]
fn project_rules_precede_module_detection_which_can_be_disabled() {
    let legacy = "version = 1\n[[rules]]\nid = \"legacy\"\nmatch = [\"lib/src/legacy/**\"]\npurpose = \"other\"\ncategory = \"legacy\"\n";
    let disabled = "version = 1\n[classification]\nbuiltin_modules = false\n";
    for (config, expected) in [
        (
            legacy,
            [
                triple("lib/src/legacy/old.ts", "other", "legacy"),
                triple("lib/src/new.ts", "production", "builtin:npm-module"),
            ],
        ),
        (
            disabled,
            [
                triple("lib/src/legacy/old.ts", "other", "fallback"),
                triple("lib/src/new.ts", "other", "fallback"),
            ],
        ),
    ] {
        let temp = project(&[
            (".locmetryrc", config),
            ("lib/package.json", "{}\n"),
            ("lib/src/legacy/old.ts", "let a = 1;\n"),
            ("lib/src/new.ts", "let b = 1;\n"),
        ]);
        let (success, document, stderr) = report(&temp.path().join("project"), &[]);
        assert!(success, "{stderr}");
        let source: Vec<_> = classifications(&document)
            .into_iter()
            .filter(|(path, _, _)| path.ends_with(".ts"))
            .collect();
        assert_eq!(source, expected);
    }
}
