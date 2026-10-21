use super::*;

/// One-based line ranges, as an editor shows them.
fn lines(text: &str) -> Vec<(usize, usize)> {
    item_lines(text)
        .into_iter()
        .map(|range| (range.start() + 1, range.end() + 1))
        .collect()
}

#[test]
fn a_gated_module_spans_from_its_attribute_to_its_closing_brace() {
    let text = "fn a() {}\n#[cfg(test)]\nmod tests {\n    fn b() {}\n}\nfn c() {}\n";
    assert_eq!(lines(text), [(2, 5)]);
}

#[test]
fn bodiless_items_and_compound_conditions_span_nothing() {
    for text in [
        "#[cfg(test)]\nmod tests;\n",
        "#[cfg(test)]\nuse std::fmt;\n",
        "#[cfg(test)]\nconst LIMIT: [u8; 2] = [0; 2];\n",
        "#[cfg(not(test))]\nmod real {}\n",
        "#[cfg(all(test, unix))]\nmod unix {}\n",
        "#![cfg(test)]\nfn whole() {}\n",
    ] {
        assert_eq!(lines(text), [], "{text}");
    }
}

#[test]
fn further_attributes_and_signatures_stay_inside_the_item() {
    let text =
        "#[cfg(test)]\n#[allow(dead_code)]\n/// Helper.\nfn helper() -> [u8; 2] {\n    [0; 2]\n}\n";
    assert_eq!(lines(text), [(1, 6)]);
}

#[test]
fn gated_items_nest_inside_impls_and_count_once_inside_modules() {
    let text = "impl A {\n    #[cfg(test)]\n    fn probe(&self) {}\n}\n#[cfg(test)]\nmod tests {\n    #[cfg(test)]\n    fn inner() {}\n}\n";
    assert_eq!(lines(text), [(2, 3), (5, 9)]);
}

#[test]
fn a_gated_field_ends_without_a_body() {
    let text = "struct S {\n    #[cfg(test)]\n    probe: u8,\n    real: u8,\n}\n";
    assert_eq!(lines(text), []);
}

#[test]
fn braces_and_attributes_in_comments_and_literals_are_inert() {
    let text = r####"// #[cfg(test)] mod a {}
/* #[cfg(test)] /* nested } */ mod b {} */
const S: &str = "#[cfg(test)] mod c { \" }";
const R: &str = r#"}"# ;
const B: &[u8] = br##"} "# }"##;
const C: char = '}';
const Q: char = '\'';
const U: char = '\u{7D}';
const D: u8 = b'}';
fn life<'a>(x: &'a str) -> &'a str { x }
#[cfg(test)]
mod tests {
    const T: &str = "\"}";
    const R: &str = r#"}"#;
    const Q: char = '\'';
    // }
    /* } */
    fn r#type<'a>() -> char { '{' }
}
"####;
    assert_eq!(lines(text), [(11, 19)]);
}

#[test]
fn counts_follow_tokei_line_categories_including_doc_comments() {
    let text =
        "fn a() {}\n\n#[cfg(test)]\nmod tests {\n    /// Doc.\n    // Note.\n\n    fn b() {}\n}\n";
    let counts = counts(text, &tokei::Config::default());
    assert_eq!(
        (
            counts.files,
            counts.code,
            counts.comments,
            counts.blanks,
            counts.lines
        ),
        (1, 4, 2, 1, 7)
    );
    let none = super::counts(
        "fn a() {}\n#[cfg(test)]\nmod tests;\n",
        &tokei::Config::default(),
    );
    assert_eq!(none, Counts::default());
}
