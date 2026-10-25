//! Wording and hints for configuration errors. Agents repairing broken
//! `.locmetryrc` files read the schema or guessed after the parser's bare
//! messages (docs/research/agents/experiments-2026-10-25.md), so each error
//! names the likely fix where the fault allows one.
use tokei::LanguageType;

/// Fields people write for another one, with the hint that names it. A typo
/// close to a valid field is found by edit distance instead.
const SYNONYMS: &[(&[&str], &str, &str)] = &[
    (
        &["language_mappings", "languages_mappings", "mappings"],
        "languages",
        "Did you mean '[[languages.mappings]]'?",
    ),
    (
        &["language_corrections", "corrections"],
        "languages",
        "Did you mean '[[languages.corrections]]'?",
    ),
    (
        &["exclude", "excludes", "exclusions", "ignore"],
        "except",
        "Did you mean 'except'? It leaves files to later stages, while 'exclude' under '[scope]' removes them from measurement.",
    ),
    (
        &["exclude", "excludes", "exclusions", "ignore"],
        "scope",
        "Did you mean 'exclude' under '[scope]'?",
    ),
    (
        &["pattern", "patterns", "glob", "globs", "path", "paths", "files", "include"],
        "match",
        "Did you mean 'match'?",
    ),
];

/// The parser's `message` in the wording of the TOML a user writes, with a
/// hint where one applies; `line` is the source line the error points at.
pub(super) fn toml_message(message: &str, line: Option<&str>) -> String {
    let mut text = message.replace('`', "'");
    // Keep a quote character the message names, as in "expected '\"'".
    if !text.contains("'\"'") {
        text = text.replace('"', "'");
    }
    for (from, to) in [
        ("expected a sequence", "expected an array"),
        ("invalid type: map", "invalid type: table"),
        ("expected a map", "expected a table"),
        ("expected u32", "expected an integer"),
        ("expected u64", "expected an integer"),
    ] {
        text = text.replace(from, to);
    }
    let hint = unknown_field_hint(&text).or_else(|| {
        let table = line?.trim().strip_prefix('[')?.strip_suffix(']')?;
        (text.contains("invalid type: table, expected an array") && !table.starts_with('[')).then(
            || format!("Write '[[{table}]]': each entry is a '[[{table}]]' table of its own."),
        )
    });
    match hint {
        Some(hint) => format!("{text}. {hint}"),
        None => text,
    }
}

/// A hint for `unknown field 'x', expected one of 'a', 'b'`: a synonym or a
/// near typo of an expected field, otherwise that a newer release may define
/// it, since a configuration written for one is the other common cause. An
/// unknown value of a fixed set, `unknown variant 'x'`, gets only the typo.
fn unknown_field_hint(text: &str) -> Option<String> {
    let (field, rest) = text
        .strip_prefix("unknown field '")
        .map(|rest| (true, rest))
        .or_else(|| Some((false, text.strip_prefix("unknown variant '")?)))?;
    let (given, rest) = rest.split_once('\'')?;
    let expected: Vec<&str> = rest.split('\'').skip(1).step_by(2).collect();
    let synonym = SYNONYMS.iter().find_map(|(names, target, hint)| {
        (field && names.contains(&given) && expected.contains(target)).then(|| hint.to_string())
    });
    let typo =
        closest(given, expected.iter().copied()).map(|name| format!("Did you mean '{name}'?"));
    synonym.or(typo).or_else(|| {
        field.then(|| {
            format!(
                "Locmetry {} does not know this field. A newer release may.",
                env!("CARGO_PKG_VERSION")
            )
        })
    })
}

/// Why `name` is no counter language, with the name meant where one is close.
pub(super) fn unknown_language(name: &str) -> String {
    let names = LanguageType::list()
        .iter()
        .map(|(language, _)| language.name());
    let suggestion =
        closest(name, names).map_or(String::new(), |meant| format!(" Did you mean '{meant}'?"));
    format!(
        "Unknown language '{name}'. Language names are case-sensitive.{suggestion} '--schema config --section LanguageName' lists them."
    )
}

/// The candidate `given` most likely meant: one equal ignoring case, else the
/// nearest within two edits and less than half its length, so short names
/// never suggest unrelated ones.
fn closest<'a>(given: &str, candidates: impl Iterator<Item = &'a str> + Clone) -> Option<&'a str> {
    let lower = given.to_lowercase();
    candidates
        .clone()
        .find(|candidate| candidate.eq_ignore_ascii_case(given))
        .or_else(|| {
            candidates
                .map(|candidate| (edit_distance(&lower, &candidate.to_lowercase()), candidate))
                .filter(|(distance, candidate)| *distance <= 2 && *distance < candidate.len() / 2)
                .min()
                .map(|(_, candidate)| candidate)
        })
}

/// Levenshtein distance over characters.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(ca != *cb));
            diagonal = above;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distance_counts_single_character_edits() {
        assert_eq!(edit_distance("builtin_test", "builtin_tests"), 1);
        assert_eq!(edit_distance("kotlinn", "kotlin"), 1);
        assert_eq!(edit_distance("match", "pattern"), 5);
        assert_eq!(edit_distance("", "abc"), 3);
    }

    #[test]
    fn an_unknown_field_without_a_close_match_points_to_newer_releases() {
        let hint = unknown_field_hint("unknown field 'content', expected one of 'id', 'match'");
        assert!(hint.unwrap().contains("A newer release may."));
    }
}
