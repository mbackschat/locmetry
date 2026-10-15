use super::{decoded_text, ParseError, ParseFailureReason};
use std::path::Path;
use tokei::{CodeStats, Report};

pub(super) fn parse(path: &Path) -> Result<Report, ParseError> {
    let text = decoded_text(path, ParseFailureReason::InvalidEncoding, "Handlebars")?;
    let mut report = Report::new(path.to_owned());
    report.stats = count(&text);
    Ok(report)
}

#[derive(Default)]
enum State {
    #[default]
    Text,
    Expression(&'static str),
    Quoted {
        quote: u8,
        end: &'static str,
    },
    Literal(&'static str),
    Comment(&'static str),
    Raw(usize),
}

#[derive(Default)]
enum Html {
    #[default]
    Text,
    Tag,
    Attribute(u8),
}

/// Count template text without interpreting its apostrophes as string delimiters.
///
/// Tokei 15.0.0's languages.json declares global Handlebars quotes, but strings
/// belong to expressions: https://handlebarsjs.com/guide/expressions.html.
/// HTML attributes retain quote protection for literal comment markers.
/// Delimiters and escapes follow the reference lexer's `mu`, `com`, and `raw`
/// states: https://github.com/handlebars-lang/handlebars.js/blob/v4.7.8/src/handlebars.l.
fn count(text: &str) -> CodeStats {
    let mut stats = CodeStats::new();
    let mut state = State::default();
    let mut html = Html::default();
    for line in text.lines() {
        let mut code = matches!(state, State::Quoted { .. } | State::Literal(_))
            || matches!((&state, &html), (State::Text, Html::Attribute(_)));
        let mut comment = matches!(state, State::Comment(_));
        let mut offset = 0;
        while offset < line.len() {
            let rest = &line.as_bytes()[offset..];
            let mut consumed = 1;
            match &state {
                State::Comment(end) => {
                    comment = true;
                    let closing = if *end == "--}}" && rest.starts_with(b"--~}}") {
                        "--~}}"
                    } else {
                        end
                    };
                    if rest.starts_with(closing.as_bytes()) {
                        consumed = closing.len();
                        state = State::Text;
                    }
                }
                State::Quoted { quote, end } => {
                    code = true;
                    if rest[0] == b'\\' && rest.get(1) == Some(quote) {
                        consumed = 2;
                    } else if rest[0] == *quote {
                        state = State::Expression(end);
                    }
                }
                State::Literal(end) => {
                    code = true;
                    if rest.starts_with(b"\\]") {
                        consumed = 2;
                    } else if rest[0] == b']' {
                        state = State::Expression(end);
                    }
                }
                State::Expression(end) => {
                    code |= !rest[0].is_ascii_whitespace();
                    let closing = if *end == "}}}" && rest.starts_with(b"}~}}") {
                        "}~}}"
                    } else {
                        end
                    };
                    if rest.starts_with(closing.as_bytes()) {
                        consumed = closing.len();
                        state = if *end == "}}}}" {
                            State::Raw(1)
                        } else {
                            State::Text
                        };
                    } else {
                        match rest[0] {
                            b'\'' | b'"' => {
                                state = State::Quoted {
                                    quote: rest[0],
                                    end,
                                }
                            }
                            b'[' => state = State::Literal(end),
                            _ => {}
                        }
                    }
                }
                State::Raw(depth) => {
                    code |= !rest[0].is_ascii_whitespace();
                    if rest.starts_with(b"{{{{/") {
                        if let Some(end) = rest[5..].windows(4).position(|bytes| bytes == b"}}}}") {
                            consumed = 5 + end + 4;
                            state = if *depth == 1 {
                                State::Text
                            } else {
                                State::Raw(depth - 1)
                            };
                        }
                    } else if rest.starts_with(b"{{{{") {
                        consumed = 4;
                        state = State::Raw(depth + 1);
                    }
                }
                State::Text => {
                    if rest.starts_with(b"\\\\{{") {
                        code = true;
                        consumed = 2;
                    } else if rest.starts_with(b"\\{{") {
                        code = true;
                        consumed = 3;
                    } else if rest.starts_with(b"{{{{") {
                        code = true;
                        state = State::Expression("}}}}");
                        consumed = 4;
                    } else if rest.starts_with(b"{{") {
                        let opening = if rest.starts_with(b"{{~") { 3 } else { 2 };
                        let body = &rest[opening..];
                        if body.starts_with(b"!--") {
                            state = State::Comment("--}}");
                            comment = true;
                            consumed = opening + 3;
                        } else if body.starts_with(b"!") {
                            state = State::Comment("}}");
                            comment = true;
                            consumed = opening + 1;
                        } else {
                            let triple = body.starts_with(b"{");
                            state = State::Expression(if triple { "}}}" } else { "}}" });
                            code = true;
                            consumed = opening + usize::from(triple);
                        }
                    } else if matches!(html, Html::Text) && rest.starts_with(b"<!--") {
                        state = State::Comment("-->");
                        comment = true;
                        consumed = 4;
                    } else {
                        code |= !rest[0].is_ascii_whitespace();
                        match html {
                            Html::Text
                                if rest[0] == b'<'
                                    && rest.get(1).is_some_and(|byte| {
                                        byte.is_ascii_alphabetic() || *byte == b'/'
                                    }) =>
                            {
                                html = Html::Tag
                            }
                            Html::Tag => match rest[0] {
                                b'\'' | b'"' => html = Html::Attribute(rest[0]),
                                b'>' => html = Html::Text,
                                _ => {}
                            },
                            Html::Attribute(quote) if rest[0] == quote => html = Html::Tag,
                            _ => {}
                        }
                    }
                }
            }
            offset += consumed;
        }
        match (code, comment) {
            (true, _) => stats.code += 1,
            (false, true) => stats.comments += 1,
            (false, false) => stats.blanks += 1,
        }
    }
    stats
}
