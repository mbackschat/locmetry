use std::str::Lines;

/// Read local scope hints using git-config's section, quoting, comment and
/// continuation rules: https://git-scm.com/docs/git-config#_syntax. Includes
/// are deliberately not followed: measurement only consults the root file.
pub(super) fn paths(text: &str) -> Vec<String> {
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    let mut in_submodule = false;
    let mut paths = Vec::new();
    while let Some(line) = lines.next() {
        let mut line = line.trim_start_matches([' ', '\t']);
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        if line.starts_with('[') {
            let Some((selected, rest)) = section(line) else {
                in_submodule = false;
                continue;
            };
            in_submodule = selected;
            line = rest.trim_start_matches([' ', '\t']);
        }
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        // Consume continuations even for unrelated keys, so a continued value
        // cannot masquerade as a fresh path declaration or section header.
        let value = value_text(value.trim_start_matches([' ', '\t']), &mut lines);
        if in_submodule
            && name
                .trim_end_matches([' ', '\t'])
                .eq_ignore_ascii_case("path")
        {
            if let Some(value) = value.filter(|value| !value.is_empty()) {
                paths.push(value.trim_end_matches('/').to_string());
            }
        }
    }
    paths.retain(|path| !path.is_empty());
    paths
}

fn section(line: &str) -> Option<(bool, &str)> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices().skip(1) {
        match (escaped, ch) {
            (true, _) => escaped = false,
            (false, '\\') if quoted => escaped = true,
            (false, '"') => quoted = !quoted,
            (false, ']') if !quoted => {
                let body = &line[1..index];
                let end = body.find([' ', '\t']).unwrap_or(body.len());
                let name = &body[..end];
                let subsection = body[end..].trim_matches([' ', '\t']);
                let selected = match name.split_once('.') {
                    Some((section, subsection)) => {
                        section.eq_ignore_ascii_case("submodule")
                            && !subsection.is_empty()
                            && name
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
                    }
                    None => name.eq_ignore_ascii_case("submodule") && quoted_subsection(subsection),
                };
                return Some((selected, &line[index + 1..]));
            }
            _ => {}
        }
    }
    None
}

fn quoted_subsection(text: &str) -> bool {
    let Some(text) = text.strip_prefix('"') else {
        return false;
    };
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return chars.next().is_none(),
            '\\' => {
                if chars.next().is_none_or(|ch| ch == '\0') {
                    return false;
                }
            }
            '\0' => return false,
            _ => {}
        }
    }
    false
}

fn value_text<'a>(mut line: &'a str, lines: &mut Lines<'a>) -> Option<String> {
    let mut value = String::new();
    let mut whitespace = String::new();
    let mut quoted = false;
    loop {
        let mut chars = line.chars();
        let mut continued = false;
        while let Some(ch) = chars.next() {
            match ch {
                '#' | ';' if !quoted => return Some(value),
                ' ' | '\t' if !quoted => whitespace.push(ch),
                '"' => {
                    value.push_str(&whitespace);
                    whitespace.clear();
                    quoted = !quoted;
                }
                '\\' => {
                    let Some(escaped) = chars.next() else {
                        continued = true;
                        break;
                    };
                    let decoded = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'b' => '\u{8}',
                        '\\' | '"' => escaped,
                        _ => return None,
                    };
                    value.push_str(&whitespace);
                    whitespace.clear();
                    value.push(decoded);
                }
                '\0' => return None,
                _ => {
                    value.push_str(&whitespace);
                    whitespace.clear();
                    value.push(ch);
                }
            }
        }
        if !continued {
            return (!quoted).then_some(value);
        }
        line = lines.next()?;
    }
}
