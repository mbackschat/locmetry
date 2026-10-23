//! Java's `@Generated` on the first top-level type. Java SE documents that
//! the annotation can also mark generated parts of a hand-written file
//! (`javax.annotation.processing.Generated`), so only the type position
//! states that the whole file is generated.

/// The packages whose `Generated` annotation Java SE and Jakarta document.
const PACKAGES: [&str; 3] = [
    "javax.annotation.processing",
    "javax.annotation",
    "jakarta.annotation",
];

/// Keywords that start a top-level type declaration; `record` is contextual.
const DECLARATIONS: [&str; 4] = ["class", "interface", "enum", "record"];

#[derive(Debug, PartialEq)]
enum Token<'a> {
    Word(&'a str),
    Punct(char),
}

/// `@<name>` as written, when the annotations before the first top-level
/// type include the documented `Generated`, by qualified name or by a
/// single-type import.
pub(super) fn generated_annotation(text: &str) -> Option<String> {
    let tokens = tokens(text);
    let mut imported = false;
    let mut annotations = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tokens[index] {
            Token::Word(keyword @ ("package" | "import")) => {
                let end = statement_end(&tokens, index);
                if keyword == "import" {
                    imported |= is_documented(&qualified_name(&tokens[index + 1..end]));
                }
                annotations.clear();
                index = end + 1;
            }
            Token::Punct('@') => match tokens.get(index + 1) {
                Some(Token::Word("interface")) => break,
                _ => {
                    let start = index + 1;
                    let mut end = start;
                    while matches!(tokens.get(end), Some(Token::Word(_)))
                        || (tokens.get(end) == Some(&Token::Punct('.'))
                            && matches!(tokens.get(end + 1), Some(Token::Word(_))))
                    {
                        end += 1;
                    }
                    annotations.push(qualified_name(&tokens[start..end]));
                    index = match tokens.get(end) {
                        Some(Token::Punct('(')) => balanced_end(&tokens, end) + 1,
                        _ => end,
                    };
                }
            },
            Token::Word(word) if DECLARATIONS.contains(&word) => break,
            _ => index += 1,
        }
    }
    annotations
        .into_iter()
        .find(|name| (imported && name == "Generated") || is_documented(name))
        .map(|name| format!("@{name}"))
}

/// Whether `name` is the qualified name of a documented `Generated`.
fn is_documented(name: &str) -> bool {
    name.strip_suffix(".Generated")
        .is_some_and(|package| PACKAGES.contains(&package))
}

fn qualified_name(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|token| match token {
            Token::Word(word) => *word,
            Token::Punct('.') => ".",
            _ => "",
        })
        .collect()
}

/// Index of the `;` that ends the statement starting at `start`.
fn statement_end(tokens: &[Token], start: usize) -> usize {
    (start..tokens.len())
        .find(|&index| tokens[index] == Token::Punct(';'))
        .unwrap_or(tokens.len())
}

/// Index of the `)` closing the `(` at `open`, skipping nested pairs; string
/// contents never reach the tokens, so braces and slashes inside annotation
/// arguments cannot end it early.
fn balanced_end(tokens: &[Token], open: usize) -> usize {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::Punct('(') => depth += 1,
            Token::Punct(')') => {
                depth -= 1;
                if depth == 0 {
                    return index;
                }
            }
            _ => {}
        }
    }
    tokens.len()
}

/// Words and punctuation, without comments, string, text-block and character
/// literals.
fn tokens(text: &str) -> Vec<Token<'_>> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let rest = &text[index..];
        let byte = bytes[index];
        if rest.starts_with("//") {
            index += rest.find('\n').unwrap_or(rest.len());
        } else if let Some(comment) = rest.strip_prefix("/*") {
            index += comment.find("*/").map_or(rest.len(), |end| end + 4);
        } else if let Some(block) = rest.strip_prefix("\"\"\"") {
            index += block.find("\"\"\"").map_or(rest.len(), |end| end + 6);
        } else if byte == b'"' || byte == b'\'' {
            index += literal_length(rest, byte);
        } else {
            let first = rest.chars().next().expect("index is inside the text");
            if is_word(first) {
                let length = rest.find(|c: char| !is_word(c)).unwrap_or(rest.len());
                tokens.push(Token::Word(&rest[..length]));
                index += length;
            } else {
                if !first.is_whitespace() {
                    tokens.push(Token::Punct(first));
                }
                index += first.len_utf8();
            }
        }
    }
    tokens
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Length of the literal opening `rest` with `quote`, through its closing
/// quote or the end of the line.
fn literal_length(rest: &str, quote: u8) -> usize {
    let bytes = rest.as_bytes();
    let mut index = 1;
    while index < bytes.len() && bytes[index] != quote && bytes[index] != b'\n' {
        index += if bytes[index] == b'\\' { 2 } else { 1 };
    }
    (index + 1).min(bytes.len())
}
