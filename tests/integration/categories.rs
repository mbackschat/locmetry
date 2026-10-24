//! Locmetry's own documents, help and policies name categories from the
//! reference in docs/guides/categories.md, so they model what the guide
//! recommends (docs/proposals/archive/category-reference.md). The guards parse TOML
//! instead of searching text, so reasons and captured output never trigger them.
use crate::common;
use globset::Glob;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Policy,
    Workspace,
}
use Kind::{Policy, Workspace};

/// Every TOML example in document order. A scanned source missing here must
/// have none, so an example that disappears, changes kind or appears unlisted
/// fails instead of silently leaving the guard.
const INVENTORY: &[(&str, &[Kind])] = &[
    ("README.md", &[Workspace, Policy]),
    ("docs/PRODUCT.md", &[Policy, Policy]),
    ("docs/cli/explain.md", &[Policy, Policy]),
    ("docs/cli/root.md", &[Policy, Policy, Policy]),
    (
        "docs/guides/configure.md",
        &[Policy, Policy, Policy, Policy, Policy],
    ),
    ("docs/guides/reports.md", &[Workspace, Workspace]),
    ("docs/showcase/tour.md", &[Policy]),
    ("locmetry --help", &[Policy]),
    ("locmetry --schema config", &[Policy]),
    (".locmetryrc", &[Policy]),
    ("examples/cli-workspace/acme-orders/.locmetryrc", &[Policy]),
    (
        "examples/cli-workspace/acme-orders/service/.locmetryrc",
        &[Policy],
    ),
    (
        "examples/cli-workspace/acme-orders/web/.locmetryrc",
        &[Policy],
    ),
    ("examples/showcase/immich/overlay/.locmetryrc", &[Policy]),
];

const POLICY_FILES: &[&str] = &[
    ".locmetryrc",
    "examples/cli-workspace/acme-orders/.locmetryrc",
    "examples/cli-workspace/acme-orders/service/.locmetryrc",
    "examples/cli-workspace/acme-orders/web/.locmetryrc",
    "examples/showcase/immich/overlay/.locmetryrc",
];

/// Folders whose Markdown is scanned; research records and archives are
/// frozen evidence and stay outside.
const DOCUMENT_FOLDERS: &[&str] = &["docs/guides", "docs/cli", "docs/showcase"];

/// A project-specific name one entry uses deliberately.
struct Exception {
    source: &'static str,
    entry: &'static str,
    category: &'static str,
    /// Why no reference name fits; required so each exception is reviewed.
    #[allow(dead_code)]
    reason: &'static str,
}

/// No document needs a project-specific name at present; an addition must say
/// why no reference name fits.
const EXCEPTIONS: &[Exception] = &[];

/// Labels Locmetry assigns itself and policies never should.
const ASSIGNED_ONLY: &[&str] = &["unclaimed", "third-party-wrapper"];

const FORBIDDEN_SUFFIXES: &[&str] = &["-test", "-tests", "-source", "-code", "-data"];

struct Vocabulary {
    names: BTreeSet<(String, String)>,
    replaced: BTreeSet<String>,
}

impl Vocabulary {
    fn read() -> Self {
        let guide = read("docs/guides/categories.md");
        let table = guide
            .split_once("<!-- category-vocabulary -->\n")
            .expect("the guide marks its vocabulary table")
            .1;
        let mut vocabulary = Self {
            names: BTreeSet::new(),
            replaced: BTreeSet::new(),
        };
        for row in table
            .lines()
            .take_while(|line| line.starts_with('|'))
            .skip(2)
        {
            let cells: Vec<_> = row
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            let [purpose, name, _, _, replaced] = cells[..] else {
                panic!("vocabulary row needs five cells: {row}");
            };
            vocabulary
                .names
                .insert((purpose.to_string(), name.trim_matches('`').to_string()));
            for synonym in replaced.split(", ").filter(|cell| !cell.is_empty()) {
                vocabulary
                    .replaced
                    .insert(synonym.trim_matches('`').to_string());
            }
        }
        assert!(vocabulary.names.len() > 20, "vocabulary table not found");
        vocabulary
    }

    fn contains(&self, purpose: &str, category: &str) -> bool {
        self.names
            .contains(&(purpose.to_string(), category.to_string()))
    }
}

fn read(path: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// TOML fences and `<<'TOML'` heredocs of a Markdown text, in document order.
fn examples(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (opening, closing) in [("```toml\n", "```"), ("<<'TOML'\n", "\nTOML\n")] {
        for (position, _) in text.match_indices(opening) {
            let body = &text[position + opening.len()..];
            let end = body
                .find(closing)
                .unwrap_or_else(|| panic!("unterminated {opening:?}"));
            found.push((position, body[..end].to_string()));
        }
    }
    found.sort();
    found.into_iter().map(|(_, body)| body).collect()
}

/// The indented TOML after a help line ending in `(TOML):`.
fn help_examples(help: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut lines = help.lines();
    while let Some(line) = lines.next() {
        if line.trim_end().ends_with("(TOML):") {
            let indent = line.len() - line.trim_start().len() + 2;
            let body: Vec<_> = lines
                .clone()
                .take_while(|line| {
                    line.is_empty() || line.len() - line.trim_start().len() >= indent
                })
                .map(|line| line.get(indent..).unwrap_or(""))
                .collect();
            found.push(body.join("\n"));
        }
    }
    found
}

fn output(args: &[&str]) -> String {
    let out = common::locmetry().args(args).output().unwrap();
    assert!(out.status.success(), "{args:?}");
    String::from_utf8(out.stdout).unwrap()
}

/// Every scanned source with its examples; sources without examples are kept
/// so that one gaining an example is noticed.
fn sources() -> Vec<(String, Vec<String>)> {
    let mut sources = Vec::new();
    let mut documents = vec!["README.md".to_string(), "docs/PRODUCT.md".to_string()];
    for folder in DOCUMENT_FOLDERS {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(folder);
        let mut names: Vec<_> = std::fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .filter(|name| name.ends_with(".md"))
            .collect();
        names.sort();
        documents.extend(names.into_iter().map(|name| format!("{folder}/{name}")));
    }
    for path in documents {
        let found = examples(&read(&path));
        sources.push((path, found));
    }
    sources.push((
        "locmetry --help".into(),
        help_examples(&output(&["--help"])),
    ));
    let schema: Value = serde_json::from_str(&output(&["--schema", "config"])).unwrap();
    sources.push((
        "locmetry --schema config".into(),
        examples(schema["description"].as_str().unwrap()),
    ));
    for path in POLICY_FILES {
        sources.push((path.to_string(), vec![read(path)]));
    }
    sources
}

fn kind(example: &str) -> Kind {
    if example
        .lines()
        .any(|line| line.starts_with("roots") || line.starts_with("project ="))
    {
        Workspace
    } else {
        Policy
    }
}

fn source_error(source: &str, examples: &[String]) -> Option<String> {
    let found: Vec<_> = examples.iter().map(|example| kind(example)).collect();
    let listed = INVENTORY
        .iter()
        .find(|(name, _)| *name == source)
        .map_or(&[][..], |(_, kinds)| *kinds);
    (found != listed).then(|| format!("{source}: found {found:?}, inventory lists {listed:?}"))
}

fn inventory_errors(sources: &[(String, Vec<String>)]) -> Vec<String> {
    let mut errors: Vec<_> = sources
        .iter()
        .filter_map(|(source, examples)| source_error(source, examples))
        .collect();
    for (name, _) in INVENTORY {
        if !sources.iter().any(|(source, _)| source == name) {
            errors.push(format!("{name}: listed in the inventory but not scanned"));
        }
    }
    errors
}

fn naming_errors(name: &str, declared: bool, vocabulary: &Vocabulary) -> Vec<String> {
    let mut errors = Vec::new();
    let kebab = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--");
    if !kebab {
        errors.push(format!("`{name}` is not lowercase kebab-case"));
    }
    if let Some(suffix) = FORBIDDEN_SUFFIXES.iter().find(|s| name.ends_with(*s)) {
        errors.push(format!("`{name}` ends in `{suffix}`"));
    }
    if vocabulary.replaced.contains(name) || ASSIGNED_ONLY.contains(&name) {
        errors.push(format!("`{name}` is a replaced or assigned-only name"));
    }
    if !declared {
        errors.push(format!("`{name}` is not declared in [categories]"));
    }
    errors
}

/// Category errors of one policy, and the exceptions it used.
fn conformity_errors(
    source: &str,
    policy: &str,
    vocabulary: &Vocabulary,
    exceptions: &[Exception],
    used: &mut BTreeSet<(String, String)>,
) -> Vec<String> {
    let table: toml::Table = toml::from_str(policy).unwrap();
    let declared = table
        .get("categories")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::new();
    for stage in ["overrides", "rules", "refinements"] {
        let entries = table.get(stage).and_then(toml::Value::as_array);
        for entry in entries.into_iter().flatten() {
            let field = |key: &str| entry.get(key).and_then(toml::Value::as_str);
            let (id, purpose) = (field("id").unwrap(), field("purpose").unwrap());
            let category = field("category").unwrap_or(purpose);
            if vocabulary.contains(purpose, category) {
                continue;
            }
            let pair = format!("{purpose}:{category}");
            match exceptions
                .iter()
                .find(|e| e.source == source && e.entry == id && e.category == pair)
            {
                Some(_) => {
                    used.insert((source.to_string(), id.to_string()));
                    let naming =
                        naming_errors(category, declared.contains_key(category), vocabulary);
                    errors.extend(naming.into_iter().map(|e| format!("{source} {id}: {e}")));
                }
                None => errors.push(format!(
                    "{source} {id}: `{pair}` is neither a reference name nor a reviewed exception"
                )),
            }
        }
    }
    errors
}

fn unused_exceptions(exceptions: &[Exception], used: &BTreeSet<(String, String)>) -> Vec<String> {
    exceptions
        .iter()
        .filter(|e| !used.contains(&(e.source.to_string(), e.entry.to_string())))
        .map(|e| format!("unused exception {} {} {}", e.source, e.entry, e.category))
        .collect()
}

#[test]
fn documents_help_and_policies_keep_their_example_inventory() {
    let sources = sources();
    let errors = inventory_errors(&sources);
    assert!(errors.is_empty(), "{errors:#?}");
    let workspace: Value = serde_json::from_str(&output(&["--schema", "workspace"])).unwrap();
    let validator = jsonschema::validator_for(&workspace).unwrap();
    for (source, examples) in &sources {
        for example in examples {
            match kind(example) {
                Policy => {
                    if let Err(error) = locmetry::Policy::from_toml(example) {
                        panic!("{source}: invalid policy: {error}\n{example}");
                    }
                }
                Workspace => {
                    let value =
                        serde_json::to_value(toml::from_str::<toml::Table>(example).unwrap())
                            .unwrap();
                    assert!(
                        validator.is_valid(&value),
                        "{source}: invalid workspace\n{example}"
                    );
                }
            }
        }
    }
}

#[test]
fn inventory_detects_a_lost_example_beside_a_remaining_one() {
    let text = read("docs/cli/explain.md");
    let second = text.rfind("<<'TOML'\n").unwrap();
    let mutated = format!("{}<<'EOF'\n{}", &text[..second], &text[second + 9..]);
    assert!(source_error("docs/cli/explain.md", &examples(&text)).is_none());
    assert!(source_error("docs/cli/explain.md", &examples(&mutated)).is_some());
}

#[test]
fn documents_help_and_policies_use_reference_categories() {
    let vocabulary = Vocabulary::read();
    let mut used = BTreeSet::new();
    let mut errors = Vec::new();
    for (source, examples) in sources() {
        for example in examples.iter().filter(|example| kind(example) == Policy) {
            errors.extend(conformity_errors(
                &source,
                example,
                &vocabulary,
                EXCEPTIONS,
                &mut used,
            ));
        }
    }
    errors.extend(unused_exceptions(EXCEPTIONS, &used));
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn conformity_rejects_names_outside_the_reference() {
    let vocabulary = Vocabulary::read();
    let entry = |purpose: &str, category: &str, declared: &str| {
        format!(
            "version = 1\n[categories]\n{declared}\n[[rules]]\nid = \"e\"\nmatch = [\"a/**\"]\npurpose = \"{purpose}\"\ncategory = \"{category}\"\n"
        )
    };
    for (purpose, category, declared) in [
        ("test", "helpers", ""),
        ("production", "packaging", "packaging = \"Packaging\""),
        ("test", "tooling", "tooling = \"Tooling\""),
        ("other", "unclaimed", ""),
        ("production", "production", ""),
    ] {
        let policy = entry(purpose, category, declared);
        let errors = conformity_errors("fixture", &policy, &vocabulary, &[], &mut BTreeSet::new());
        assert_eq!(errors.len(), 1, "{purpose}:{category}: {errors:#?}");
    }
    let walkthrough = "```bash\ncat > proposed.toml <<'TOML'\nversion = 1\n\n[[refinements]]\nid = \"orders\"\npurpose = \"test\"\nmatch = [\"web/**\"]\ncategory = \"order-rules\"\nTOML\n```\n";
    let found = examples(walkthrough);
    assert_eq!(found.len(), 1);
    let errors = conformity_errors("fixture", &found[0], &vocabulary, &[], &mut BTreeSet::new());
    assert_eq!(errors.len(), 1, "{errors:#?}");
}

#[test]
fn exceptions_must_follow_naming_rules_be_declared_and_be_used() {
    let vocabulary = Vocabulary::read();
    let exception = |entry, category| Exception {
        source: "fixture",
        entry,
        category,
        reason: "fixture",
    };
    let exceptions = [
        exception("proofs", "test:proof"),
        exception("spelling", "test:Proof"),
        exception("suffix", "test:proof-data"),
        exception("undeclared", "test:lemma"),
        exception("absent", "test:law"),
    ];
    let policy = "version = 1\n[categories]\nproof = \"Machine-checked proofs\"\nProof = \"x\"\n\"proof-data\" = \"x\"\n\
        [[refinements]]\nid = \"proofs\"\npurpose = \"test\"\nmatch = [\"a/**\"]\ncategory = \"proof\"\n\
        [[refinements]]\nid = \"spelling\"\npurpose = \"test\"\nmatch = [\"b/**\"]\ncategory = \"Proof\"\n\
        [[refinements]]\nid = \"suffix\"\npurpose = \"test\"\nmatch = [\"d/**\"]\ncategory = \"proof-data\"\n\
        [[refinements]]\nid = \"undeclared\"\npurpose = \"test\"\nmatch = [\"c/**\"]\ncategory = \"lemma\"\n";
    let mut used = BTreeSet::new();
    let errors = conformity_errors("fixture", policy, &vocabulary, &exceptions, &mut used);
    assert_eq!(
        errors.len(),
        3,
        "kebab-case, suffix and declaration: {errors:#?}"
    );
    assert!(errors.iter().all(|error| !error.contains("proofs")));
    assert_eq!(unused_exceptions(&exceptions, &used).len(), 1);
}

/// `--table-category` values in commands of the scanned documents.
fn selectors(text: &str) -> Vec<String> {
    text.match_indices("--table-category")
        .filter_map(|(position, flag)| {
            let rest = text[position + flag.len()..].trim_start_matches([' ', '=']);
            let value = match rest.strip_prefix('\'') {
                Some(quoted) => quoted.split('\'').next().unwrap(),
                None => rest.split([' ', '`', '\n', '\\']).next().unwrap(),
            };
            let placeholder = value.chars().all(|c| c.is_ascii_uppercase());
            (!value.is_empty() && !placeholder).then(|| value.to_string())
        })
        .collect()
}

#[test]
fn documented_category_selectors_select_reference_names() {
    let vocabulary = Vocabulary::read();
    let mut checked = 0;
    for (source, _) in sources()
        .iter()
        .filter(|(source, _)| source.ends_with(".md"))
    {
        for selector in selectors(&read(source)) {
            for item in selector.split(',') {
                checked += 1;
                let (purpose, category) = item.split_once(':').unwrap_or((item, "*"));
                let matcher = |glob: &str| Glob::new(glob).unwrap().compile_matcher();
                let (purpose, category) = (matcher(purpose), matcher(category));
                assert!(
                    vocabulary
                        .names
                        .iter()
                        .any(|(p, c)| purpose.is_match(p) && category.is_match(c)),
                    "{source}: `{item}` selects no reference name"
                );
            }
        }
    }
    assert!(checked >= 5, "selectors not found");
}
