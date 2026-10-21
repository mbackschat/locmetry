//! Inline Rust tests: items under an outer `#[cfg(test)]` attribute, which
//! whole-file classification counts with their file (docs/measurement.md,
//! Limits). Tokei counts the extracted lines, so the figures use the same
//! line categories as the file's own counts.
use super::{language_counts, Stats};
use crate::{Counts, Kind};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use tokei::LanguageType;

/// Counts of the lines `#[cfg(test)]` items span in Rust `text`; zero
/// without such items.
pub(super) fn counts(text: &str, config: &tokei::Config) -> Counts {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut total = Counts::default();
    for range in item_lines(text) {
        let stats = LanguageType::Rust.parse_from_str(lines[range].join("\n"), config);
        let stats: Stats = serde_json::to_value(stats)
            .and_then(serde_json::from_value)
            .expect("Tokei statistics have the backend's shape");
        let mut languages = BTreeMap::new();
        language_counts("Rust", &stats, Kind::Source, &mut languages);
        for counts in languages.values() {
            total.add(counts);
        }
    }
    total.files = u64::from(total.lines > 0);
    total
}

/// Zero-based line ranges of braced items with an outer `#[cfg(test)]`
/// attribute, outermost only. Bodiless items such as `mod tests;` have no
/// range, because their code is counted in their own file; compound
/// conditions such as `cfg(all(test, unix))` are not recognized.
pub(super) fn item_lines(text: &str) -> Vec<RangeInclusive<usize>> {
    let tokens = tokens(text);
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if cfg_test_at(&tokens, index) {
            let start = tokens[index].line;
            match body_end(&tokens, index + CFG_TEST.len()) {
                Some((end, next)) => {
                    ranges.push(start..=tokens[end].line);
                    index = next;
                }
                None => index += CFG_TEST.len(),
            }
        } else {
            index += 1;
        }
    }
    ranges
}

const CFG_TEST: [Token<'static>; 7] = [
    Token::Punct('#'),
    Token::Punct('['),
    Token::Ident("cfg"),
    Token::Punct('('),
    Token::Ident("test"),
    Token::Punct(')'),
    Token::Punct(']'),
];

fn cfg_test_at(tokens: &[Located], index: usize) -> bool {
    tokens.len() >= index + CFG_TEST.len()
        && tokens[index..index + CFG_TEST.len()]
            .iter()
            .zip(&CFG_TEST)
            .all(|(located, token)| located.token == *token)
}

/// The index of the closing brace of the item starting at `from`, and the
/// token after it; `None` when the item ends without a body at `;` or `,`,
/// or when its enclosing scope closes first, as after a gated field.
fn body_end(tokens: &[Located], from: usize) -> Option<(usize, usize)> {
    let mut nesting = 0usize;
    let mut index = from;
    while index < tokens.len() {
        match tokens[index].token {
            Token::Punct('(' | '[') => nesting += 1,
            Token::Punct(')' | ']') if nesting > 0 => nesting -= 1,
            Token::Punct('{') if nesting == 0 => return closing_brace(tokens, index),
            Token::Punct(';' | ',' | ')' | ']' | '}') if nesting == 0 => return None,
            _ => {}
        }
        index += 1;
    }
    None
}

fn closing_brace(tokens: &[Located], open: usize) -> Option<(usize, usize)> {
    let mut depth = 0usize;
    for (index, located) in tokens.iter().enumerate().skip(open) {
        match located.token {
            Token::Punct('{') => depth += 1,
            Token::Punct('}') => {
                depth -= 1;
                if depth == 0 {
                    return Some((index, index + 1));
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Token<'a> {
    Punct(char),
    Ident(&'a str),
    /// A literal, lifetime or number, whose content never delimits items.
    Literal,
}

struct Located<'a> {
    token: Token<'a>,
    line: usize,
}

/// Rust tokens with their zero-based lines. Comments, strings, raw strings
/// and character literals are skipped as units, so braces and attribute text
/// inside them neither open nor close an item.
fn tokens(text: &str) -> Vec<Located<'_>> {
    let mut lexer = Lexer {
        text,
        position: 0,
        line: 0,
    };
    let mut tokens = Vec::new();
    while let Some(character) = lexer.peek(0) {
        let line = lexer.line;
        let start = lexer.position;
        let token = match character {
            '/' if lexer.peek(1) == Some('/') => {
                lexer.skip_while(|c| c != '\n');
                continue;
            }
            '/' if lexer.peek(1) == Some('*') => {
                lexer.block_comment();
                continue;
            }
            '"' => {
                lexer.string();
                Token::Literal
            }
            '\'' => {
                lexer.quote();
                Token::Literal
            }
            c if c.is_alphabetic() || c == '_' => match lexer.string_prefix() {
                true => Token::Literal,
                false => {
                    lexer.skip_while(|c| c.is_alphanumeric() || c == '_');
                    Token::Ident(&text[start..lexer.position])
                }
            },
            c if c.is_ascii_digit() => {
                lexer.skip_while(|c| c.is_alphanumeric() || c == '_');
                Token::Literal
            }
            c if c.is_whitespace() => {
                lexer.bump();
                continue;
            }
            c => {
                lexer.bump();
                Token::Punct(c)
            }
        };
        tokens.push(Located { token, line });
    }
    tokens
}

struct Lexer<'a> {
    text: &'a str,
    position: usize,
    line: usize,
}

impl Lexer<'_> {
    fn peek(&self, ahead: usize) -> Option<char> {
        self.text[self.position..].chars().nth(ahead)
    }

    fn bump(&mut self) -> Option<char> {
        let character = self.peek(0)?;
        self.position += character.len_utf8();
        self.line += usize::from(character == '\n');
        Some(character)
    }

    fn skip_while(&mut self, keep: impl Fn(char) -> bool) {
        while self.peek(0).is_some_and(&keep) {
            self.bump();
        }
    }

    /// Block comments nest in Rust.
    fn block_comment(&mut self) {
        let mut depth = 0usize;
        while let Some(character) = self.bump() {
            match (character, self.peek(0)) {
                ('/', Some('*')) => {
                    self.bump();
                    depth += 1;
                }
                ('*', Some('/')) => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    /// A string from its opening quote, honouring backslash escapes.
    fn string(&mut self) {
        self.bump();
        while let Some(character) = self.bump() {
            match character {
                '\\' => {
                    self.bump();
                }
                '"' => return,
                _ => {}
            }
        }
    }

    /// A character literal such as `'}'` or `'\''`, or a lifetime or label
    /// such as `'a`, whose quote is never closed.
    fn quote(&mut self) {
        self.bump();
        match (self.peek(0), self.peek(1)) {
            (Some('\\'), _) => {
                self.bump();
                self.bump();
                self.skip_while(|c| c != '\'');
                self.bump();
            }
            (Some(_), Some('\'')) => {
                self.bump();
                self.bump();
            }
            _ => self.skip_while(|c| c.is_alphanumeric() || c == '_'),
        }
    }

    /// Consumes a literal with an identifier-like prefix: byte, C and raw
    /// strings (`b"…"`, `c"…"`, `r#"…"#`, `br"…"`) and byte characters
    /// (`b'}'`); a raw identifier such as `r#type` stays an identifier.
    fn string_prefix(&mut self) -> bool {
        let rest = &self.text[self.position..];
        let prefix = ["br", "cr", "b", "c", "r"]
            .into_iter()
            .find(|prefix| rest.starts_with(prefix))
            .unwrap_or("");
        let after = &rest[prefix.len()..];
        if prefix.ends_with('r') {
            let hashes = after.len() - after.trim_start_matches('#').len();
            if after[hashes..].starts_with('"') {
                self.position += prefix.len() + hashes;
                self.raw_string(hashes);
                return true;
            }
        } else if !prefix.is_empty() && after.starts_with('"') {
            self.position += prefix.len();
            self.string();
            return true;
        } else if prefix == "b" && after.starts_with('\'') {
            self.position += 1;
            self.quote();
            return true;
        }
        false
    }

    /// A raw string from its opening quote, closed by a quote and `hashes` `#`s.
    fn raw_string(&mut self, hashes: usize) {
        self.bump();
        let closing = format!("\"{}", "#".repeat(hashes));
        while self.peek(0).is_some() {
            if self.text[self.position..].starts_with(&closing) {
                self.position += closing.len();
                return;
            }
            self.bump();
        }
    }
}

#[cfg(test)]
mod tests;
