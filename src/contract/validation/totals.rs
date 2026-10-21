//! Aggregate arithmetic: totals that must reconcile across summary, breakdown and file records.
use super::*;

pub(super) fn validate_totals(s: &Summary, context: &str) -> Result<()> {
    s.all_counted.validate(context)?;
    s.source.validate(context)?;
    for counts in [&s.all_counted, &s.source]
        .into_iter()
        .chain(s.by_kind_purpose.values())
        .chain(s.source_purpose.values())
        .chain(s.source_categories.values())
        .chain(s.source_languages.values())
    {
        ensure!(
            counts.files > 0 || counts.lines == 0,
            "{context}: nonzero totals need physical files"
        );
    }
    for counts in s.source_languages.values() {
        ensure!(
            counts.files <= s.source.files,
            "{context}: language participation exceeds source files"
        );
    }
    for c in s
        .by_kind_purpose
        .values()
        .chain(s.source_purpose.values())
        .chain(s.source_categories.values())
        .chain(s.source_languages.values())
    {
        c.validate(context)?;
    }
    for key in s.by_kind_purpose.keys() {
        let (kind, purpose) = key.split_once(':').context("kind:purpose key required")?;
        ensure!(
            Kind::ALL.iter().any(|k| k.to_string() == kind),
            "invalid kind {kind}"
        );
        purpose_key(purpose, PurposeKey::Purpose)?;
    }
    for key in s.source_categories.keys() {
        purpose_key(key, PurposeKey::Category)?;
    }
    ensure!(
        total(s.by_kind_purpose.values())? == s.all_counted,
        "{context}: by_kind_purpose totals conflict with all_counted"
    );
    ensure!(
        total(s.source_purpose.values())? == s.source,
        "{context}: source_purpose totals conflict with source"
    );
    ensure!(
        total(s.source_categories.values())? == s.source,
        "{context}: source_categories totals conflict with source"
    );
    ensure!(same_lines(total(s.source_languages.values())?,s.source), "{context}: source_languages line totals conflict with source (file participation may overlap)");
    for (purpose, inline) in &s.inline_tests {
        inline.validate(context)?;
        let source = s.purpose(*purpose);
        ensure!(
            *purpose != Purpose::Test
                && inline.lines > 0
                && inline.files <= source.files
                && inline.code <= source.code
                && inline.comments <= source.comments
                && inline.blanks <= source.blanks,
            "{context}: inline_tests must lie within non-test source of their purpose"
        );
    }
    for p in Purpose::ALL {
        let code = s.source_purpose.get(&p).copied().unwrap_or_default();
        ensure!(
            s.by_kind_purpose
                .get(&format!("source:{p}"))
                .copied()
                .unwrap_or_default()
                == code,
            "{context}: source kind/purpose totals conflict"
        );
        let categories = total(
            s.source_categories
                .iter()
                .filter(|(key, _)| key.starts_with(&format!("{p}:")))
                .map(|(_, c)| c),
        )?;
        ensure!(
            categories == code,
            "{context}: category totals conflict for {p}"
        );
    }
    Ok(())
}

pub(super) fn add<K: Ord>(map: &mut BTreeMap<K, Counts>, key: K, counts: &Counts) -> Result<()> {
    let value = map.entry(key).or_default();
    *value = value.checked_add(counts)?;
    Ok(())
}

pub(super) fn totals_from_rows<'a>(
    rows: impl Iterator<Item = &'a crate::BreakdownRow>,
) -> Result<Summary> {
    let mut s = Summary::default();
    for r in rows {
        s.all_counted = s.all_counted.checked_add(&r.counts)?;
        add(
            &mut s.by_kind_purpose,
            crate::report::kind_purpose_key(r.kind, r.purpose),
            &r.counts,
        )?;
        if r.kind == Kind::Source {
            s.source = s.source.checked_add(&r.counts)?;
            add(&mut s.source_purpose, r.purpose, &r.counts)?;
            add(
                &mut s.source_categories,
                crate::report::category_key(r.purpose, &r.category),
                &r.counts,
            )?;
            add(&mut s.source_languages, r.language.clone(), &r.counts)?;
        }
    }
    Ok(s)
}

pub(super) fn merge_totals(into: &mut Summary, s: &Summary) -> Result<()> {
    into.all_counted = into.all_counted.checked_add(&s.all_counted)?;
    into.source = into.source.checked_add(&s.source)?;
    for (k, v) in &s.by_kind_purpose {
        add(&mut into.by_kind_purpose, k.clone(), v)?;
    }
    for (k, v) in &s.source_purpose {
        add(&mut into.source_purpose, *k, v)?;
    }
    for (k, v) in &s.source_categories {
        add(&mut into.source_categories, k.clone(), v)?;
    }
    for (k, v) in &s.source_languages {
        add(&mut into.source_languages, k.clone(), v)?;
    }
    Ok(())
}

/// How file counts must agree: exactly, or at least as many when the expected
/// side derives from breakdown rows, which count a file only for its own
/// language, not for each language it contains.
#[derive(Clone, Copy)]
pub(super) enum Files {
    Exact,
    AtLeast,
}

pub(super) fn same_map<K: Ord>(
    actual: &BTreeMap<K, Counts>,
    expected: &BTreeMap<K, Counts>,
    files: Files,
) -> bool {
    actual.keys().chain(expected.keys()).all(|key| {
        let a = actual.get(key).copied().unwrap_or_default();
        let b = expected.get(key).copied().unwrap_or_default();
        match files {
            Files::Exact => a == b,
            Files::AtLeast => same_lines(a, b) && a.files >= b.files,
        }
    })
}

pub(super) fn same_totals(
    actual: &Summary,
    expected: &Summary,
    language_files: Files,
    context: &str,
) -> Result<()> {
    ensure!(
        actual.all_counted == expected.all_counted
            && actual.source == expected.source
            && same_map(
                &actual.by_kind_purpose,
                &expected.by_kind_purpose,
                Files::Exact
            )
            && same_map(
                &actual.source_purpose,
                &expected.source_purpose,
                Files::Exact
            )
            && same_map(
                &actual.source_categories,
                &expected.source_categories,
                Files::Exact
            ),
        "{context}: aggregate totals conflict"
    );
    ensure!(
        same_map(
            &actual.source_languages,
            &expected.source_languages,
            language_files
        ),
        "{context}: language counts conflict"
    );
    Ok(())
}
