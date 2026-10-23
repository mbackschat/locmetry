//! Module detection: build descriptors declare where a project's production
//! source lives, so production is evidence from the build, not a guess.
//!
//! Each build system is one [`ModuleDetector`] in [`DETECTORS`]; supporting
//! another build system means adding a detector there. Detectors only look at
//! the selected files and never run a build (CLAUDE.md, reference projects
//! stay read-only).
use crate::config::{literal_glob, module_detector};
use crate::paths::{ancestors, below};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};

/// Recognizes one build system's modules and their production source.
pub trait ModuleDetector: Sync {
    /// Build-system name; files it claims record `builtin:<id>-module`.
    fn id(&self) -> &'static str;
    /// File names that make their directory a module; `*.ext` matches any
    /// file name with that extension, as .NET project files and gemspecs
    /// carry the project's name.
    fn descriptors(&self) -> &'static [&'static str];
    /// The production layout in Markdown, for the configuration schema.
    fn layout(&self) -> String;
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

    fn descriptors(&self) -> &'static [&'static str] {
        self.descriptors
    }

    fn layout(&self) -> String {
        self.sources
            .iter()
            .map(|glob| format!("`{glob}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn production_sources(&self, _: &str, _: &[&str]) -> Vec<String> {
        self.sources.iter().map(|glob| glob.to_string()).collect()
    }
}

/// A build system whose production layout depends on which folders exist.
struct ComputedLayout {
    id: &'static str,
    descriptors: &'static [&'static str],
    layout: &'static str,
    /// Globs from the selected paths relative to the module's directory.
    sources: fn(&[&str]) -> Vec<String>,
}

impl ModuleDetector for ComputedLayout {
    fn id(&self) -> &'static str {
        self.id
    }

    fn descriptors(&self) -> &'static [&'static str] {
        self.descriptors
    }

    fn layout(&self) -> String {
        self.layout.into()
    }

    fn production_sources(&self, directory: &str, files: &[&str]) -> Vec<String> {
        let prefix = match directory {
            "." => String::new(),
            _ => format!("{directory}/"),
        };
        let relative: Vec<&str> = files
            .iter()
            .filter_map(|path| path.strip_prefix(prefix.as_str()))
            .collect();
        (self.sources)(&relative)
    }
}

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

/// The flat layout's top-level packages, as setuptools' automatic discovery
/// finds them.
fn python_flat_layout(files: &[&str]) -> Vec<String> {
    let packages: BTreeSet<&str> = files
        .iter()
        .filter_map(|path| path.strip_suffix("/__init__.py"))
        .filter(|package| !package.contains('/') && !PYTHON_NON_PACKAGES.contains(package))
        .collect();
    packages
        .into_iter()
        .map(|package| format!("{}/**", literal_glob(package)))
        .collect()
}

fn python_layout(files: &[&str]) -> Vec<String> {
    std::iter::once("src/**".to_string())
        .chain(python_flat_layout(files))
        .collect()
}

/// Laravel's documented folders, and each folder in `resources/` except
/// `lang/`: translations held 54% of one application's source as PHP arrays
/// (docs/research/framework-layouts.md, precision pass).
fn laravel_layout(files: &[&str]) -> Vec<String> {
    let resources: BTreeSet<&str> = files
        .iter()
        .filter_map(|path| path.strip_prefix("resources/"))
        .filter_map(|path| path.split_once('/'))
        .map(|(folder, _)| folder)
        .filter(|folder| *folder != "lang")
        .collect();
    [
        "app/**",
        "bootstrap/**",
        "config/**",
        "database/migrations/**",
        "routes/**",
    ]
    .into_iter()
    .map(String::from)
    .chain(
        resources
            .into_iter()
            .map(|folder| format!("resources/{}/**", literal_glob(folder))),
    )
    .collect()
}

/// Built-in detectors. Layouts follow each tool's documented defaults: npm
/// and Cargo `src/`, Maven and Gradle `src/main/` plus Kotlin Multiplatform
/// `src/<target>Main/`, Go packages anywhere below `go.mod`, Python's `src`
/// or flat layout, Lean sources anywhere below a Lake lakefile, Swift
/// Package Manager's `Sources/`, and the pub package layout's `lib/` and
/// `bin/` for Dart and Flutter; framework detectors follow the documentation
/// cited at each. Test conventions run earlier, so test files inside these
/// trees stay tests.
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
    &ComputedLayout {
        id: "python",
        descriptors: &["pyproject.toml", "setup.py", "setup.cfg"],
        layout: "`src/**`, plus each top-level folder with an `__init__.py` except names such as `docs`, `scripts`, and `tools`",
        sources: python_layout,
    },
    // Lake's `lean_lib` defaults to `srcDir = "."`, so Lean sources sit at the
    // package root; the lakefile itself is a build script, matched earlier.
    &DefaultLayout {
        id: "lake",
        descriptors: &["lakefile.lean", "lakefile.toml"],
        sources: &["**/*.lean"],
    },
    &DefaultLayout {
        id: "swift",
        descriptors: &["Package.swift"],
        sources: &["Sources/**"],
    },
    // https://dart.dev/tools/pub/package-layout: libraries in `lib/`, public
    // executables in `bin/`; `tool/`, `example/` and `benchmark/` stay open.
    &DefaultLayout {
        id: "dart",
        descriptors: &["pubspec.yaml"],
        sources: &["lib/**", "bin/**"],
    },
    // Frameworks whose source is not in `src/`, with the folders their primary
    // documentation names; whole-package layouts were rejected because they
    // claim scripts, mocks and vendored code (docs/research/framework-layouts.md).
    // https://nextjs.org/docs/app/getting-started/project-structure: `app`,
    // `pages`, `src` and the root files are conventions; the other folders
    // are the docs' "generalized placeholders", adopted as what the docs show.
    &DefaultLayout {
        id: "next",
        descriptors: &[
            "next.config.js",
            "next.config.cjs",
            "next.config.mjs",
            "next.config.ts",
        ],
        sources: &[
            "{app,pages,src}/**",
            "{instrumentation,instrumentation-client,proxy,middleware,mdx-components}.{js,jsx,ts,tsx}",
            "{components,lib,ui,utils,hooks,styles}/**",
        ],
    },
    // https://nuxt.com/docs/4.x/directory-structure, plus the Nuxt 3 root
    // folders that Nuxt 4 moved into `app/`.
    &DefaultLayout {
        id: "nuxt",
        descriptors: &["nuxt.config.js", "nuxt.config.mjs", "nuxt.config.ts"],
        sources: &[
            "{app,server,shared,layers,modules}/**",
            "{assets,components,composables,layouts,middleware,pages,plugins,utils}/**",
            "{app,error}.vue",
            "app.config.*",
        ],
    },
    // React Router framework mode's default `appDirectory`.
    &DefaultLayout {
        id: "react-router",
        descriptors: &["react-router.config.js", "react-router.config.ts"],
        sources: &["app/**"],
    },
    // The SDK's default compile items and Razor files
    // (https://learn.microsoft.com/dotnet/core/project-sdk/overview), Blazor's
    // scoped `.razor.css`, which the build rewrites and bundles
    // (https://learn.microsoft.com/aspnet/core/blazor/components/css-isolation),
    // and XAML, compiled by MAUI and WPF
    // (https://learn.microsoft.com/dotnet/maui/xaml/xamlc). Static files in
    // `wwwroot/` are not built and stay open.
    &DefaultLayout {
        id: "dotnet",
        descriptors: &["*.csproj", "*.fsproj", "*.vbproj"],
        sources: &["**/*.{cs,fs,vb,razor,cshtml,xaml}", "**/*.razor.css"],
    },
    // PSR-4's `src/`, which Symfony and most Composer packages use.
    &DefaultLayout {
        id: "composer",
        descriptors: &["composer.json"],
        sources: &["src/**"],
    },
    // https://symfony.com/doc/current/best_practices.html
    &DefaultLayout {
        id: "symfony",
        descriptors: &["symfony.lock"],
        sources: &["{src,assets,config,migrations,templates}/**"],
    },
    // https://laravel.com/docs/12.x/structure
    &ComputedLayout {
        id: "laravel",
        descriptors: &["artisan"],
        layout: "`app/**`, `bootstrap/**`, `config/**`, `database/migrations/**`, `routes/**`, and each folder in `resources/` except `lang/`",
        sources: laravel_layout,
    },
    // `rails new` creates `config.ru`; Rails autoloads `app/` and `lib/`.
    &DefaultLayout {
        id: "rails",
        descriptors: &["config.ru"],
        sources: &["{app,lib,config}/**", "db/migrate/**"],
    },
    // The RubyGems layout: libraries in `lib/`, executables in `exe/`.
    &DefaultLayout {
        id: "gem",
        descriptors: &["*.gemspec"],
        sources: &["lib/**", "exe/**"],
    },
    // Django projects often lack packaging metadata; `manage.py` sits beside
    // the project and app packages that `startproject` and `startapp` create.
    &ComputedLayout {
        id: "django",
        descriptors: &["manage.py"],
        layout: "each top-level folder with an `__init__.py` except names such as `docs`, `scripts`, and `tools`",
        sources: python_flat_layout,
    },
];

/// Whether a file `name` is the build `descriptor`, which may be `*.ext`.
fn declares(descriptor: &str, name: &str) -> bool {
    match descriptor.strip_prefix('*') {
        Some(extension) => name.len() > extension.len() && name.ends_with(extension),
        None => descriptor == name,
    }
}

/// A module declared by a build descriptor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Module {
    /// Root-relative directory, `.` for the scan root.
    pub path: String,
    pub detector: &'static str,
    /// Root-relative globs of the module's production source.
    #[serde(skip)]
    pub sources: Vec<String>,
}

impl Module {
    /// Whether the root-relative `path` lies in this module's directory.
    pub(crate) fn contains(&self, path: &str) -> bool {
        below(path, &self.path).is_some()
    }

    /// Directory depth, `0` for the scan root; deeper modules nest in shallower ones.
    pub(crate) fn depth(&self) -> usize {
        depth(&self.path)
    }
}

fn depth(folder: &str) -> usize {
    match folder {
        "." => 0,
        path => path.split('/').count(),
    }
}

/// Finds the module that owns a file: the innermost module of the detector
/// that decided it. Borrowed identities keep live summaries and saved-report
/// validation on the same ownership rule without requiring detection-only
/// layout data.
pub(crate) struct ModuleOwners<'a> {
    /// Module index by detector and directory; a later duplicate wins.
    modules: HashMap<(&'a str, &'a str), usize>,
}

impl<'a> ModuleOwners<'a> {
    /// Indexes `(directory, detector)` identities by their position.
    pub(crate) fn new(modules: impl Iterator<Item = (&'a str, &'a str)>) -> Self {
        let modules = modules
            .enumerate()
            .map(|(index, (folder, detector))| ((detector, folder), index))
            .collect();
        Self { modules }
    }

    /// Index of the module owning `path`, decided with provenance `rule`.
    ///
    /// Walking the path's folders costs its depth rather than the module
    /// count, which reached a thousand modules per file in a 2026-10-18
    /// profile of a synthetic 101k-file monorepo.
    pub(crate) fn owner(&self, path: &str, rule: &str) -> Option<usize> {
        let detector = module_detector(rule)?;
        ancestors(path).find_map(|folder| self.modules.get(&(detector, folder)).copied())
    }
}

/// Modules declared among the selected root-relative `files`, sorted by path, then detector ID.
/// Detector IDs in detection order, which documentation checks compare with
/// the built-in detection reference.
pub fn detector_ids() -> impl Iterator<Item = &'static str> {
    DETECTORS.iter().map(|detector| detector.id())
}

pub fn detect(files: &[&str]) -> Vec<Module> {
    let mut declared = BTreeSet::new();
    for path in files {
        let (directory, name) = path.rsplit_once('/').unwrap_or((".", path));
        for (index, detector) in DETECTORS.iter().enumerate() {
            if detector
                .descriptors()
                .iter()
                .any(|descriptor| declares(descriptor, name))
            {
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
    // Path, then detector ID: build systems sharing a folder are checked in
    // detector-ID order, the documented tie-break for overlapping layouts
    // (docs/PRODUCT.md, Module Detection), independent of the `DETECTORS` order.
    modules.sort_by(|a, b| (&a.path, a.detector).cmp(&(&b.path, b.detector)));
    modules
}
