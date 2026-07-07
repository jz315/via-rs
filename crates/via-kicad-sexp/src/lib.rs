use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sexp {
    Atom(Atom),
    List(Vec<Sexp>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atom {
    text: String,
    quoted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    message: String,
    offset: usize,
}

impl Sexp {
    pub fn atom(text: impl Into<String>) -> Self {
        Self::Atom(Atom {
            text: text.into(),
            quoted: false,
        })
    }

    pub fn string(text: impl Into<String>) -> Self {
        Self::Atom(Atom {
            text: text.into(),
            quoted: true,
        })
    }

    pub fn list(items: impl Into<Vec<Sexp>>) -> Self {
        Self::List(items.into())
    }

    pub fn as_atom(&self) -> Option<&str> {
        match self {
            Self::Atom(atom) => Some(&atom.text),
            Self::List(_) => None,
        }
    }

    pub fn list_name(&self) -> Option<&str> {
        match self {
            Self::List(items) => items.first().and_then(Self::as_atom),
            Self::Atom(_) => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}

impl error::Error for ParseError {}

pub fn parse_one(input: &str) -> Result<Sexp, ParseError> {
    let mut parser = Parser { input, offset: 0 };
    parser.skip_ws_and_comments();
    let sexp = parser.parse_expr()?;
    parser.skip_ws_and_comments();
    if !parser.is_eof() {
        return Err(parser.error("unexpected trailing input"));
    }
    Ok(sexp)
}

pub fn render(sexp: &Sexp, indent: usize) -> String {
    let mut out = String::new();
    out.push_str(&" ".repeat(indent));
    render_into(sexp, indent, &mut out);
    out.push('\n');
    out
}

struct Parser<'a> {
    input: &'a str,
    offset: usize,
}

impl Parser<'_> {
    fn parse_expr(&mut self) -> Result<Sexp, ParseError> {
        self.skip_ws_and_comments();
        match self.peek_char() {
            Some('(') => self.parse_list(),
            Some('"') => self.parse_string().map(Sexp::string),
            Some(')') => Err(self.error("unexpected ')'")),
            Some(_) => self.parse_atom().map(Sexp::atom),
            None => Err(self.error("expected expression")),
        }
    }

    fn parse_list(&mut self) -> Result<Sexp, ParseError> {
        self.expect_char('(')?;
        let mut items = Vec::new();
        loop {
            self.skip_ws_and_comments();
            match self.peek_char() {
                Some(')') => {
                    self.next_char();
                    return Ok(Sexp::list(items));
                }
                Some(_) => items.push(self.parse_expr()?),
                None => return Err(self.error("unterminated list")),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect_char('"')?;
        let mut text = String::new();
        loop {
            match self.next_char() {
                Some('"') => return Ok(text),
                Some('\\') => match self.next_char() {
                    Some('n') => text.push('\n'),
                    Some('r') => text.push('\r'),
                    Some('t') => text.push('\t'),
                    Some('"') => text.push('"'),
                    Some('\\') => text.push('\\'),
                    Some(ch) => {
                        text.push('\\');
                        text.push(ch);
                    }
                    None => return Err(self.error("unterminated string escape")),
                },
                Some(ch) => text.push(ch),
                None => return Err(self.error("unterminated string")),
            }
        }
    }

    fn parse_atom(&mut self) -> Result<String, ParseError> {
        let start = self.offset;
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() || matches!(ch, '(' | ')' | '"') {
                break;
            }
            self.next_char();
        }
        if self.offset == start {
            return Err(self.error("expected atom"));
        }
        Ok(self.input[start..self.offset].to_owned())
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while self.peek_char().is_some_and(char::is_whitespace) {
                self.next_char();
            }
            if self.peek_char() != Some(';') {
                break;
            }
            while let Some(ch) = self.next_char() {
                if ch == '\n' {
                    break;
                }
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        match self.next_char() {
            Some(ch) if ch == expected => Ok(()),
            _ => Err(self.error(format!("expected '{expected}'"))),
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn is_eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            offset: self.offset,
        }
    }
}

fn render_into(sexp: &Sexp, indent: usize, out: &mut String) {
    if let Some(inline) = render_inline(sexp) {
        out.push_str(&inline);
        return;
    }

    let Sexp::List(items) = sexp else {
        out.push_str(&render_atom(sexp));
        return;
    };
    out.push('(');
    let mut child_start = 0usize;
    for item in items {
        if matches!(item, Sexp::Atom(_)) {
            if child_start > 0 {
                out.push(' ');
            }
            render_into(item, indent + 1, out);
            child_start += 1;
        } else {
            break;
        }
    }
    for item in items.iter().skip(child_start) {
        out.push('\n');
        out.push_str(&" ".repeat(indent + 2));
        render_into(item, indent + 2, out);
    }
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push(')');
}

fn render_inline(sexp: &Sexp) -> Option<String> {
    match sexp {
        Sexp::Atom(_) => Some(render_atom(sexp)),
        Sexp::List(items) => {
            if items.is_empty() {
                return Some("()".to_owned());
            }
            if items.iter().any(|item| matches!(item, Sexp::List(_))) {
                return None;
            }
            let parts = items
                .iter()
                .map(render_inline)
                .collect::<Option<Vec<_>>>()?;
            let text = format!("({})", parts.join(" "));
            (text.len() <= 120).then_some(text)
        }
    }
}

fn render_atom(sexp: &Sexp) -> String {
    let Sexp::Atom(atom) = sexp else {
        return String::new();
    };
    if atom.quoted || needs_quotes(&atom.text) {
        format!("\"{}\"", escape_string(&atom.text))
    } else {
        atom.text.clone()
    }
}

fn needs_quotes(text: &str) -> bool {
    text.is_empty()
        || text.chars().any(|ch| {
            ch.is_whitespace() || matches!(ch, '(' | ')' | '"' | '\\' | ';') || ch.is_control()
        })
}

fn escape_string(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_parentheses_without_splitting_lists() {
        let sexp = parse_one(r#"(descr "value with (parentheses) and \"quotes\"")"#).unwrap();

        assert_eq!(
            sexp,
            Sexp::list(vec![
                Sexp::atom("descr"),
                Sexp::string("value with (parentheses) and \"quotes\""),
            ])
        );
        assert_eq!(
            render(&sexp, 0).trim(),
            r#"(descr "value with (parentheses) and \"quotes\"")"#
        );
    }

    #[test]
    fn rejects_trailing_input() {
        let err = parse_one("(a) (b)").unwrap_err();

        assert!(err.to_string().contains("unexpected trailing input"));
    }

    #[test]
    fn preserves_backslash_for_unknown_string_escapes() {
        let sexp = parse_one(r#"(descr "path\foo")"#).unwrap();

        assert_eq!(
            sexp,
            Sexp::list(vec![Sexp::atom("descr"), Sexp::string(r"path\foo")])
        );
        assert_eq!(render(&sexp, 0).trim(), r#"(descr "path\\foo")"#);
        assert_eq!(parse_one(render(&sexp, 0).trim()).unwrap(), sexp);
    }
}
