use thp_diagnostics::{Diagnostic, SourceFile, Span};

use crate::{Token, TokenKind};

#[derive(Clone, Debug)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &SourceFile) -> LexOutput {
    Lexer {
        source: source.text(),
        offset: 0,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
    }
    .run()
}

struct Lexer<'source> {
    source: &'source str,
    offset: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl Lexer<'_> {
    fn run(mut self) -> LexOutput {
        while self.offset < self.source.len() {
            self.skip_trivia();
            if self.offset >= self.source.len() {
                break;
            }
            self.lex_token();
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::empty(self.source.len()),
        });
        LexOutput {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn bytes(&self) -> &[u8] {
        self.source.as_bytes()
    }

    fn skip_trivia(&mut self) {
        loop {
            while self
                .bytes()
                .get(self.offset)
                .is_some_and(u8::is_ascii_whitespace)
            {
                self.offset += 1;
            }
            if self.bytes().get(self.offset..self.offset + 2) == Some(b"//") {
                self.offset += 2;
                while self
                    .bytes()
                    .get(self.offset)
                    .is_some_and(|byte| *byte != b'\n')
                {
                    self.offset += 1;
                }
                continue;
            }
            if self.bytes().get(self.offset..self.offset + 2) == Some(b"/*") {
                let start = self.offset;
                self.offset += 2;
                while self.offset + 1 < self.source.len()
                    && self.bytes().get(self.offset..self.offset + 2) != Some(b"*/")
                {
                    self.offset += 1;
                }
                if self.offset + 1 >= self.source.len() {
                    self.diagnostics.push(Diagnostic::error(
                        "lexing",
                        "L0002",
                        Span::new(start, self.source.len()),
                        "unterminated block comment",
                    ));
                    self.offset = self.source.len();
                } else {
                    self.offset += 2;
                }
                continue;
            }
            break;
        }
    }

    fn lex_token(&mut self) {
        let start = self.offset;
        let rest = &self.source[start..];
        if rest.starts_with("<?thp") {
            self.offset += 5;
            self.push(TokenKind::OpenTag, start);
            return;
        }

        let byte = self.bytes()[self.offset];
        if byte == b'$' {
            self.offset += 1;
            if !self
                .bytes()
                .get(self.offset)
                .is_some_and(|byte| is_identifier_start(*byte))
            {
                self.diagnostics.push(Diagnostic::error(
                    "lexing",
                    "L0003",
                    Span::new(start, self.offset),
                    "`$` must be followed by an ASCII variable name",
                ));
                return;
            }
            self.consume_identifier();
            self.push(TokenKind::Variable, start);
            return;
        }
        if is_identifier_start(byte) {
            self.consume_identifier();
            let text = &self.source[start..self.offset];
            let kind = match text {
                "function" => TokenKind::Function,
                "namespace" => TokenKind::Namespace,
                "class" => TokenKind::Class,
                "interface" => TokenKind::Interface,
                "trait" => TokenKind::Trait,
                "use" => TokenKind::Use,
                "const" => TokenKind::Const,
                "insteadof" => TokenKind::InsteadOf,
                "implements" => TokenKind::Implements,
                "extends" => TokenKind::Extends,
                "abstract" => TokenKind::Abstract,
                "final" => TokenKind::Final,
                "public" => TokenKind::Public,
                "protected" => TokenKind::Protected,
                "private" => TokenKind::Private,
                "static" => TokenKind::Static,
                "new" => TokenKind::New,
                "instanceof" => TokenKind::InstanceOf,
                "throw" => TokenKind::Throw,
                "try" => TokenKind::Try,
                "catch" => TokenKind::Catch,
                "finally" => TokenKind::Finally,
                "using" => TokenKind::Using,
                "return" => TokenKind::Return,
                "if" => TokenKind::If,
                "elseif" => TokenKind::ElseIf,
                "else" => TokenKind::Else,
                "while" => TokenKind::While,
                "for" => TokenKind::For,
                "foreach" => TokenKind::Foreach,
                "as" => TokenKind::As,
                "break" => TokenKind::Break,
                "continue" => TokenKind::Continue,
                "match" => TokenKind::Match,
                "default" => TokenKind::Default,
                "echo" => TokenKind::Echo,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "null" => TokenKind::Null,
                _ => TokenKind::Identifier,
            };
            self.push(kind, start);
            return;
        }
        if byte.is_ascii_digit() {
            self.consume_number();
            let kind = if self.source[start..self.offset].contains('.') {
                TokenKind::Float
            } else {
                TokenKind::Integer
            };
            self.push(kind, start);
            return;
        }
        if matches!(byte, b'\'' | b'"') {
            self.consume_string(byte, start);
            return;
        }

        let (kind, width) = match rest {
            value if value.starts_with("...") => (TokenKind::Ellipsis, 3),
            value if value.starts_with("===") => (TokenKind::StrictEqual, 3),
            value if value.starts_with("!==") => (TokenKind::StrictNotEqual, 3),
            value if value.starts_with("??") => (TokenKind::QuestionQuestion, 2),
            value if value.starts_with("<=") => (TokenKind::LessEqual, 2),
            value if value.starts_with(">=") => (TokenKind::GreaterEqual, 2),
            value if value.starts_with("==") => (TokenKind::EqualEqual, 2),
            value if value.starts_with("!=") => (TokenKind::BangEqual, 2),
            value if value.starts_with("&&") => (TokenKind::AndAnd, 2),
            value if value.starts_with("||") => (TokenKind::OrOr, 2),
            value if value.starts_with("=>") => (TokenKind::FatArrow, 2),
            value if value.starts_with("->") => (TokenKind::Arrow, 2),
            value if value.starts_with("::") => (TokenKind::DoubleColon, 2),
            _ => match byte {
                b'(' => (TokenKind::LParen, 1),
                b')' => (TokenKind::RParen, 1),
                b'{' => (TokenKind::LBrace, 1),
                b'}' => (TokenKind::RBrace, 1),
                b'[' => (TokenKind::LBracket, 1),
                b']' => (TokenKind::RBracket, 1),
                b'<' => (TokenKind::Less, 1),
                b'>' => (TokenKind::Greater, 1),
                b':' => (TokenKind::Colon, 1),
                b';' => (TokenKind::Semicolon, 1),
                b',' => (TokenKind::Comma, 1),
                b'?' => (TokenKind::Question, 1),
                b'+' => (TokenKind::Plus, 1),
                b'-' => (TokenKind::Minus, 1),
                b'*' => (TokenKind::Star, 1),
                b'/' => (TokenKind::Slash, 1),
                b'%' => (TokenKind::Percent, 1),
                b'.' => (TokenKind::Dot, 1),
                b'=' => (TokenKind::Equal, 1),
                b'!' => (TokenKind::Bang, 1),
                b'|' => (TokenKind::Pipe, 1),
                b'\\' => (TokenKind::NamespaceSeparator, 1),
                _ => {
                    let width = self.source[start..]
                        .chars()
                        .next()
                        .map_or(1, char::len_utf8);
                    self.offset += width;
                    self.diagnostics.push(Diagnostic::error(
                        "lexing",
                        "L0001",
                        Span::new(start, self.offset),
                        "unrecognized source character",
                    ));
                    return;
                }
            },
        };
        self.offset += width;
        self.push(kind, start);
    }

    fn consume_identifier(&mut self) {
        while self
            .bytes()
            .get(self.offset)
            .is_some_and(|byte| is_identifier_continue(*byte))
        {
            self.offset += 1;
        }
    }

    fn consume_number(&mut self) {
        while self
            .bytes()
            .get(self.offset)
            .is_some_and(u8::is_ascii_digit)
        {
            self.offset += 1;
        }
        if self.bytes().get(self.offset) == Some(&b'.')
            && self
                .bytes()
                .get(self.offset + 1)
                .is_some_and(u8::is_ascii_digit)
        {
            self.offset += 1;
            while self
                .bytes()
                .get(self.offset)
                .is_some_and(u8::is_ascii_digit)
            {
                self.offset += 1;
            }
        }
    }

    fn consume_string(&mut self, quote: u8, start: usize) {
        self.offset += 1;
        let mut closed = false;
        while let Some(byte) = self.bytes().get(self.offset).copied() {
            if byte == quote {
                self.offset += 1;
                closed = true;
                break;
            }
            if byte == b'\\' {
                self.offset += usize::from(self.offset + 1 < self.source.len()) + 1;
            } else {
                self.offset += 1;
            }
        }
        if closed {
            self.push(TokenKind::String, start);
        } else {
            self.diagnostics.push(Diagnostic::error(
                "lexing",
                "L0004",
                Span::new(start, self.source.len()),
                "unterminated string literal",
            ));
        }
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.offset),
        });
    }
}

const fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

const fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use thp_diagnostics::SourceFile;

    use super::lex;
    use crate::TokenKind;

    #[test]
    fn lexes_core_tokens_and_comments() {
        let source = SourceFile::new(
            "test.thp",
            "<?thp // comment\n$x: int = 12 + 3.5; /* done */ echo \"ok\";",
        );
        let output = lex(&source);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let kinds = output
            .tokens
            .iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                TokenKind::OpenTag,
                TokenKind::Variable,
                TokenKind::Colon,
                TokenKind::Identifier,
                TokenKind::Equal,
                TokenKind::Integer,
                TokenKind::Plus,
                TokenKind::Float,
                TokenKind::Semicolon,
                TokenKind::Echo,
                TokenKind::String,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn diagnoses_unterminated_comment() {
        let source = SourceFile::new("test.thp", "<?thp /*");
        let output = lex(&source);
        assert_eq!(output.diagnostics[0].code, "L0002");
    }
}
