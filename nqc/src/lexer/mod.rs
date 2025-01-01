use crate::{Error, ErrorKind, Span};
use std::{iter::Peekable, str::FromStr};

type Src<'src> =
    std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'src>>>;

#[derive(Debug)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind<'src> {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Plus,
    Minus,
    Asterisk,
    Dot,
    Comma,
    Divide,
    Eq,
    EqEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    LShift,
    RShift,
    Semicolon,
    Hash,
    LiteralInt(&'src str),
    Ident(&'src str),
    Kw(Keyword),
}

impl<'src> Token<'src> {
    pub fn parse(
        src: &mut Src<'src>,
        line: &mut usize,
        raw: &'src str,
    ) -> Result<Option<Self>, Error> {
        // This typically should succeed because we check for eof in the
        // TokenStream iterator
        let (start, chr) =
            src.next()
                .ok_or(Error::new(0, 0, ErrorKind::Eof, raw.into()))?;

        let mut span_length = 1;
        let kind = match chr {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Asterisk,
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            '/' => {
                // either a comment or a divide, depending on the next
                // char
                if src
                    .peek()
                    .ok_or(Error::new(start, 0, ErrorKind::Eof, raw.into()))?
                    .1
                    == '/'
                {
                    let mut last_idx = start;
                    let _newline_idx = loop {
                        if let Some((idx, chr)) = src.next() {
                            if chr == '\n' {
                                break idx;
                            }
                            last_idx = idx;
                        } else {
                            break last_idx + 1;
                        }
                    };
                    *line += 1;
                    return Ok(None);
                } else {
                    TokenKind::Divide
                }
            }
            '=' => {
                // Either an = or an ==
                if let Some((_, '=')) = src.peek() {
                    src.next();
                    span_length = 2;
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '<' => {
                // Can be < or << or <=
                match src.peek() {
                    Some((_, '<')) => {
                        src.next();
                        span_length = 2;
                        TokenKind::LShift
                    }
                    Some((_, '=')) => {
                        src.next();
                        span_length = 2;
                        TokenKind::LtEq
                    }
                    _ => TokenKind::Lt,
                }
            }
            '>' => {
                // Can be > or >> or >=
                match src.peek() {
                    Some((_, '>')) => {
                        src.next();
                        span_length = 2;
                        TokenKind::RShift
                    }
                    Some((_, '=')) => {
                        src.next();
                        span_length = 2;
                        TokenKind::GtEq
                    }
                    _ => TokenKind::Gt,
                }
            }
            ';' => TokenKind::Semicolon,
            '#' => TokenKind::Hash,
            s if s.is_ascii_whitespace() => {
                // consume whitespace until eof or non whitespace
                loop {
                    match src.peek() {
                        Some((_, chr)) if chr.is_ascii_whitespace() => {
                            span_length += 1;
                        }
                        _ => break,
                    }
                    src.next();
                }
                return Ok(None);
            }
            s if s.is_ascii_alphabetic() || s == '_' => {
                // consume ident
                loop {
                    match src.peek() {
                        Some((_, chr))
                            if chr.is_ascii_alphanumeric() || *chr == '_' =>
                        {
                            src.next();
                            span_length += 1;
                        }
                        _ => break,
                    }
                }
                let ident = &raw[start..start + span_length];
                if let Ok(kw) = Keyword::from_str(ident) {
                    TokenKind::Kw(kw)
                } else {
                    TokenKind::Ident(ident)
                }
            }
            s if s.is_ascii_digit() => {
                // consume digits
                loop {
                    match src.peek() {
                        Some((_, chr)) if chr.is_ascii_digit() => {
                            src.next();
                            span_length += 1;
                        }
                        _ => break,
                    }
                }
                TokenKind::LiteralInt(&raw[start..start + span_length])
            }
            other => Err(Error::new(
                start,
                1,
                ErrorKind::Syntax(format!("Unexpected character `{other}`")),
                raw.into(),
            ))?,
        };
        Ok(Some(Token {
            kind,
            span: Span {
                start,
                length: span_length,
            },
        }))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    #[strum(serialize = "_event_src")]
    EventSrc,
    #[strum(serialize = "__nolist")]
    Nolist,
    #[strum(serialize = "__res")]
    Res,
    #[strum(serialize = "__sensor")]
    Sensor,
    #[strum(serialize = "__taskid")]
    Taskid,
    #[strum(serialize = "__type")]
    Type,
    Abs,
    Acquire,
    Asm,
    Break,
    Case,
    Catch,
    Const,
    Continue,
    Default,
    Do,
    Else,
    False,
    For,
    Goto,
    If,
    Inline,
    Int,
    Monitor,
    Repeat,
    Return,
    Sign,
    Start,
    Stop,
    Sub,
    Switch,
    Task,
    True,
    Void,
    While,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Int,
}

pub struct Tokens<'src> {
    src: Src<'src>,
    raw: &'src str,
    tokens: Vec<Token<'src>>,
}

impl<'src> Tokens<'src> {
    pub fn new(src: &'src str) -> Result<Self, Vec<Error>> {
        let raw = src;
        let mut src = src.chars().enumerate().peekable();
        let mut line = 0;
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        loop {
            if src.peek().is_none() {
                break;
            }
            let token = Token::parse(&mut src, &mut line, raw);
            match token {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {}
                Err(error) => errors.push(error),
            }
        }
        if errors.is_empty() {
            Ok(Self { src, raw, tokens })
        } else {
            Err(errors)
        }
    }

    pub fn iter<'tokens: 'src>(&'tokens self) -> TokenStream<'src> {
        TokenStream {
            raw: self.raw,
            iter: self.tokens.iter().peekable(),
        }
    }
}

#[derive(Debug)]
pub struct TokenStream<'src> {
    raw: &'src str,
    iter: Peekable<std::slice::Iter<'src, Token<'src>>>,
}

impl<'src> TokenStream<'src> {
    pub fn peek(&mut self) -> Option<&'src Token<'src>> {
        self.iter.peek().copied()
    }

    #[track_caller]
    pub fn next_token(&mut self) -> Result<&'src Token<'src>, Error> {
        let caller = std::panic::Location::caller();
        self.iter.next().ok_or_else(|| {
            println!("Line {caller:?}");
            Error::new(0, 0, ErrorKind::Eof, self.raw.into())
        })
    }

    pub fn consume(&mut self, expected: TokenKind) -> Result<(), Error> {
        let tok = self.next_token()?;
        if tok.kind == expected {
            Ok(())
        } else {
            let span = tok.span;
            Err(Error::new(
                span.start,
                span.length,
                ErrorKind::Syntax(format!(
                    "Expected `{:?}`, got `{:?}`",
                    expected, tok.kind
                )),
                self.raw.into(),
            ))
        }
    }

    pub fn eof(&mut self) -> bool {
        self.peek().is_none()
    }

    pub fn raw(&self) -> &'src str {
        self.raw
    }
}

impl<'src> Iterator for TokenStream<'src> {
    type Item = &'src Token<'src>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn basic_tokens() {
        use TokenKind::*;

        let src = "()[]{}+-*/===;.<<>><=>=while<>#_some_var
        //this is a comment
        //another comment";
        let tokens = Tokens::new(src).unwrap();
        let stream = tokens
            .iter()
            .map(|token| token.kind.clone())
            .collect::<Vec<_>>();
        let expected = &[
            LeftParen,
            RightParen,
            LeftBracket,
            RightBracket,
            LeftBrace,
            RightBrace,
            Plus,
            Minus,
            Asterisk,
            Divide,
            EqEq,
            Eq,
            Semicolon,
            Dot,
            LShift,
            RShift,
            LtEq,
            GtEq,
            Kw(Keyword::While),
            Lt,
            Gt,
            Hash,
            Ident("_some_var"),
        ];
        assert_eq!(expected, stream.as_slice());
    }

    #[test]
    fn simple_prog() {
        use TokenKind::*;

        let src = "task main() {
            SetPower(OUT_A, 50);
            OnFwd(OUT_A);
            Wait(100);
            Off(OUT_A);
        }";
        let tokens = Tokens::new(src).unwrap();
        let stream = tokens
            .iter()
            .map(|token| token.kind.clone())
            .collect::<Vec<_>>();
        let expected = &[
            Kw(Keyword::Task),
            Ident("main"),
            LeftParen,
            RightParen,
            LeftBrace,
            Ident("SetPower"),
            LeftParen,
            Ident("OUT_A"),
            Comma,
            LiteralInt("50"),
            RightParen,
            Semicolon,
            Ident("OnFwd"),
            LeftParen,
            Ident("OUT_A"),
            RightParen,
            Semicolon,
            Ident("Wait"),
            LeftParen,
            LiteralInt("100"),
            RightParen,
            Semicolon,
            Ident("Off"),
            LeftParen,
            Ident("OUT_A"),
            RightParen,
            Semicolon,
            RightBrace,
        ];
        assert_eq!(expected, stream.as_slice());
    }
}
