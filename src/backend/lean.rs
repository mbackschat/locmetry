use super::{decoded_text, ParseError, ParseFailureReason};
use std::path::Path;
use tokei::{CodeStats, Report};

pub(super) fn parse(path: &Path) -> Result<Report, ParseError> {
    let text = decoded_text(path, ParseFailureReason::InvalidEncoding, "Lean")?;
    let mut report = Report::new(path.to_owned());
    report.stats = count(&text);
    Ok(report)
}

#[derive(Default)]
enum State {
    #[default]
    Code,
    Str {
        interpolated: bool,
    },
    Raw(usize),
    Comment(usize),
    EscapedIdent,
}

/// Count Lean lines without reading comment markers inside literals.
///
/// Tokei 15.0.0's languages.json declares no Lean quotes, so `"/-"` opens a
/// comment. Literals, identifiers and comments follow `whitespace`,
/// `finishCommentBlock` and `tokenFnAux` in Lean's lexer:
/// https://github.com/leanprover/lean4/blob/v4.35.0/src/Lean/Parser/Basic.lean.
/// Interpolation depends on the syntax that takes the string, so only the
/// `s!`, `m!` and `f!` prefixes open `{…}` terms; strings after other
/// interpolating syntax, such as `throwError`, are read as plain, which
/// misreads only comment markers in strings nested inside their terms:
/// https://github.com/leanprover/lean4/blob/v4.35.0/src/Lean/Parser/StrInterpolation.lean.
///
/// Lines are classified by Tokei's `line_is_comment` rules, so only lexing
/// changes: a line inside a literal is code, a line that starts inside a
/// comment is a comment even with code after its end, and otherwise a line
/// is a comment when its first and last non-blank characters are comment.
fn count(text: &str) -> CodeStats {
    let mut stats = CodeStats::new();
    let mut state = State::default();
    // Brace depth inside each open interpolation term.
    let mut holes: Vec<usize> = Vec::new();
    let mut interpolates = false;
    for line in text.lines() {
        let started_in_literal = in_literal(&state);
        let started_in_comment = matches!(state, State::Comment(_));
        // Whether the first and the last non-blank characters are comment.
        let mut first = None;
        let mut last = false;
        let mut previous = ' ';
        let mut offset = 0;
        while let Some(current) = line[offset..].chars().next() {
            let rest = &line[offset..];
            let mut consumed = current.len_utf8();
            let mut comment = matches!(state, State::Comment(_));
            match &mut state {
                State::Comment(depth) => {
                    if rest.starts_with("-/") {
                        consumed = 2;
                        *depth -= 1;
                        if *depth == 0 {
                            state = State::Code;
                        }
                    } else if rest.starts_with("/-") {
                        consumed = 2;
                        *depth += 1;
                    }
                }
                State::Str { interpolated } => match current {
                    '\\' => consumed += rest[1..].chars().next().map_or(0, char::len_utf8),
                    '"' => state = State::Code,
                    '{' if *interpolated => {
                        holes.push(0);
                        state = State::Code;
                    }
                    _ => {}
                },
                State::Raw(hashes) => {
                    if current == '"'
                        && rest[1..].bytes().take_while(|b| *b == b'#').count() >= *hashes
                    {
                        consumed += *hashes;
                        state = State::Code;
                    }
                }
                State::EscapedIdent => {
                    if current == '»' {
                        state = State::Code;
                    }
                }
                State::Code if rest.starts_with("--") => {
                    comment = true;
                    consumed = rest.len();
                }
                State::Code if rest.starts_with("/-") => {
                    // `/--` and `/-!` are doc comment tokens, so `/--/` stays open.
                    comment = true;
                    consumed = if rest[2..].starts_with(['-', '!']) {
                        3
                    } else {
                        2
                    };
                    state = State::Comment(1);
                }
                State::Code => {
                    let boundary = !is_id_rest(previous);
                    let prefix = boundary
                        && rest.starts_with(['s', 'm', 'f'])
                        && rest[1..].starts_with('!')
                        && !rest[2..].chars().next().is_some_and(is_id_rest);
                    match current {
                        _ if prefix => consumed = 2,
                        '"' => {
                            state = State::Str {
                                interpolated: interpolates,
                            }
                        }
                        '\'' if boundary => consumed = char_literal(rest).unwrap_or(consumed),
                        'r' if boundary => {
                            if let Some(hashes) = raw_string_hashes(rest) {
                                consumed = hashes + 2;
                                state = State::Raw(hashes);
                            }
                        }
                        '«' => state = State::EscapedIdent,
                        '{' => {
                            if let Some(depth) = holes.last_mut() {
                                *depth += 1;
                            }
                        }
                        '}' => match holes.last_mut() {
                            Some(0) => {
                                holes.pop();
                                state = State::Str { interpolated: true };
                            }
                            Some(depth) => *depth -= 1,
                            None => {}
                        },
                        _ => {}
                    }
                    if !current.is_whitespace() {
                        interpolates = prefix;
                    }
                }
            }
            let segment = &line[offset..offset + consumed];
            if !segment.trim().is_empty() {
                first.get_or_insert(comment);
                last = comment;
            }
            previous = segment.chars().last().unwrap_or(previous);
            offset += consumed;
        }
        match first {
            None if started_in_comment => stats.comments += 1,
            None if started_in_literal => stats.code += 1,
            None => stats.blanks += 1,
            Some(_) if in_literal(&state) => stats.code += 1,
            Some(first) if started_in_comment || first && last => stats.comments += 1,
            Some(_) => stats.code += 1,
        }
    }
    stats
}

fn in_literal(state: &State) -> bool {
    matches!(
        state,
        State::Str { .. } | State::Raw(_) | State::EscapedIdent
    )
}

/// Length of the character literal starting `rest`, as `charLitFnAux` reads it.
fn char_literal(rest: &str) -> Option<usize> {
    let mut chars = rest.char_indices().skip(1);
    let (_, first) = chars.next()?;
    if first == '\'' {
        return None;
    }
    if first == '\\' {
        let (_, escape) = chars.next()?;
        let digits = match escape {
            'x' => 2,
            'u' => 4,
            _ => 0,
        };
        for _ in 0..digits {
            chars.next()?;
        }
    }
    let (index, close) = chars.next()?;
    (close == '\'').then_some(index + 1)
}

/// The `#` count of a raw string starting `rest`, as `isRawStrLitStart` accepts it.
fn raw_string_hashes(rest: &str) -> Option<usize> {
    let hashes = rest[1..].bytes().take_while(|b| *b == b'#').count();
    rest[1 + hashes..].starts_with('"').then_some(hashes)
}

/// Lean's `isIdRest`, from `Init/Meta/Defs.lean` at the same tag.
fn is_id_rest(c: char) -> bool {
    let value = u32::from(c);
    c.is_ascii_alphanumeric()
        || matches!(c, '_' | '\'' | '!' | '?')
        || matches!(value, 0x3b1..=0x3c9 if value != 0x3bb)
        || matches!(value, 0x391..=0x3a9 if value != 0x3a0 && value != 0x3a3)
        || matches!(value, 0x3ca..=0x3fb | 0x1f00..=0x1ffe | 0x2100..=0x214f | 0x1d49c..=0x1d59f | 0x100..=0x17f)
        || matches!(value, 0xc0..=0xff if value != 0xd7 && value != 0xf7)
        || matches!(value, 0x2080..=0x2089 | 0x2090..=0x209c | 0x1d62..=0x1d6a | 0x2c7c)
}
