//! Module detection: build descriptors declare where a project's production
//! source lives, so production is evidence from the build, not a guess.
//!
//! Each build system is one [`ModuleDetector`] in [`DETECTORS`]; supporting
//! another build system means adding a detector there. Detectors only look at
//! the selected files and never run a build (CLAUDE.md, reference projects
//! stay read-only).
use crate::config::literal_glob;
use serde::Serialize;
use std::collections::BTreeSet;

/// Recognizes one build system's modules and their production source.
pub trait ModuleDetector: Sync {
    /// Build-system name; files it claims record `builtin:<id>-module`.
    fn id(&self) -> &'static str;
    /// Whether a file with this name makes its directory a module.
    fn is_descriptor(&self, file_name: &str) -> bool;
    /// Globs of the production source of the module in `directory`, relative
    /// to it; `files` are all selected root-relative paths, for layouts that
    /// depend on what exists.
    fn production_sources(&self, directory: &str, files: &[&str]) -> Vec<String>;
}

/// A build system whose production source follows a fixed default layout.
struct DefaultLayout {
    id: &'static str,
    descriptors: &'static [&'static str],
    sources: &'static [&'static str],
}

impl ModuleDetector for DefaultLayout {
    fn id(&self) -> &'static str {
        self.id
    }

    fn is_descriptor(&self, file_name: &str) -> bool {
        self.descriptors.contains(&file_name)
    }

    fn production_sources(&self, _: &str, _: &[&str]) -> Vec<String> {
        self.sources.iter().map(|glob| glob.to_string()).collect()
    }
}

/// Python packaging: the `src` layout, or the flat layout's top-level packages,
/// as setuptools' automatic discovery finds them.
struct Python;

/// Top-level names setuptools' flat-layout discovery never treats as packages
/// (setuptools `discovery.FlatLayoutPackageFinder.DEFAULT_EXCLUDE`, a subset
/// relevant to source trees).
const PYTHON_NON_PACKAGES: &[&str] = &[
    "benchmark",
    "benchmarks",
    "bin",
    "build",
    "ci",
    "dist",
    "doc",
    "docs",
    "documentation",
    "env",
    "example",
    "examples",
    "scripts",
    "tasks",
    "tools",
    "util",
    "utils",
    "venv",
];

impl ModuleDetector for Python {
    fn id(&self) -> &'static str {
        "python"
    }

    fn is_descriptor(&self, file_name: &str) -> bool {
        ["pyproject.toml", "setup.py", "setup.cfg"].contains(&file_name)
    }

    fn production_sources(&self, directory: &str, files: &[&str]) -> Vec<String> {
        let prefix = match directory {
            "." => String::new(),
            _ => format!("{directory}/"),
        };
        let packages: BTreeSet<&str> = files
            .iter()
            .filter_map(|path| path.strip_prefix(prefix.as_str()))
            .filter_map(|path| path.strip_suffix("/__init__.py"))
            .filter(|package| !package.contains('/') && !PYTHON_NON_PACKAGES.contains(package))
            .collect();
        std::iter::once("src/**".to_string())
            .chain(
                packages
                    .into_iter()
                    .map(|package| format!("{}/**", literal_glob(package))),
            )
            .collect()
    }
}

/// Built-in detectors. Layouts follow each tool's documented defaults: npm
/// and Cargo `src/`, Maven and Gradle `src/main/` plus Kotlin Multiplatform
/// `src/<target>Main/`, Go packages anywhere below `go.mod`, and Python's
/// `src` or flat layout. Test conventions run earlier, so test files inside
/// these trees stay tests.
pub static DETECTORS: &[&dyn ModuleDetector] = &[
    &DefaultLayout {
        id: "npm",
        descriptors: &["package.json"],
        sources: &["src/**"],
    },
    &DefaultLayout {
        id: "gradle",
        descriptors: &["build.gradle", "build.gradle.kts"],
        sources: &["src/main/**", "src/*Main/**"],
    },
    &DefaultLayout {
        id: "maven",
        descriptors: &["pom.xml"],
        sources: &["src/main/**"],
    },
    &DefaultLayout {
        id: "cargo",
        descriptors: &["Cargo.toml"],
        sources: &["src/**"],
    },
    &DefaultLayout {
        id: "go",
        descriptors: &["go.mod"],
        sources: &["**/*.go"],
    },
    &Python,
];

/// A module declared by a build descriptor.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Module {
    /// Root-relative directory, `.` for the scan root.
    pub path: String,
    pub detector: &'static str,
    /// Root-relative globs of the module's production source.
    #[serde(skip)]
    pub sources: Vec<String>,
}

/// Modules declared among the selected root-relative `files`, sorted by path.
pub fn detect(files: &[&str]) -> Vec<Module> {
    let mut declared = BTreeSet::new();
    for path in files {
        let (directory, name) = path.rsplit_once('/').unwrap_or((".", path));
        for (index, detector) in DETECTORS.iter().enumerate() {
            if detector.is_descriptor(name) {
                declared.insert((directory, index));
            }
        }
    }
    let mut modules: Vec<Module> = declared
        .into_iter()
        .map(|(directory, index)| {
            let detector = DETECTORS[index];
            let prefix = match directory {
                "." => String::new(),
                _ => format!("{}/", literal_glob(directory)),
            };
            Module {
                path: directory.into(),
                detector: detector.id(),
                sources: detector
                    .production_sources(directory, files)
                    .into_iter()
                    .map(|glob| format!("{prefix}{glob}"))
                    .collect(),
            }
        })
        .collect();
    modules.sort();
    modules
}
