use crate::{Error, ErrorKind, Span};
use std::str::FromStr;

type Src<'src> =
    std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'src>>>;

#[derive(Debug)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq)]
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
    Space,
    LiteralInt(&'src str),
    Comment(&'src str),
    Ident(&'src str),
    Kw(Keyword),
}

impl<'src> Token<'src> {
    pub fn parse(
        src: &mut Src<'src>,
        line: &mut usize,
        raw: &'src str,
    ) -> Result<Self, Error<'src>> {
        // This typically should succeed because we check for eof in the
        // TokenStream iterator
        let (start, chr) =
            src.next().ok_or(Error::new(0, 0, ErrorKind::Eof, raw))?;

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
                    .ok_or(Error::new(start, 0, ErrorKind::Eof, raw))?
                    .1
                    == '/'
                {
                    let mut last_idx = start;
                    let newline_idx = loop {
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
                    span_length = newline_idx - start;
                    TokenKind::Comment(&raw[start..start + span_length])
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
                TokenKind::Space
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
            _ => Err(Error::new(start, 1, ErrorKind::Syntax, raw))?,
        };
        Ok(Token {
            kind,
            span: Span {
                start,
                length: span_length,
            },
        })
    }
}

#[derive(Debug, PartialEq, Eq, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    While,
    Task,
    Sub,
    For,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Int,
}

pub struct TokenStream<'src> {
    src: Src<'src>,
    raw: &'src str,
    line: usize,
}

impl<'src> TokenStream<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            src: src.chars().enumerate().peekable(),
            raw: src,
            line: 0,
        }
    }
}

impl<'src> Iterator for TokenStream<'src> {
    type Item = Result<Token<'src>, Error<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.src.peek()?;
        Some(Token::parse(&mut self.src, &mut self.line, self.raw))
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
        let stream = TokenStream::new(src)
            .inspect(|maybe_token| println!("{maybe_token:?}"))
            .map(|token| token.unwrap().kind)
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
            Space,
            Comment("//this is a comment"),
            Space,
            Comment("//another comment"),
        ];
        assert_eq!(stream, expected);
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
        let stream = TokenStream::new(src)
            .inspect(|maybe_token| println!("{maybe_token:?}"))
            .map(|token| token.unwrap().kind)
            .collect::<Vec<_>>();
        let expected = &[
            Kw(Keyword::Task),
            Space,
            Ident("main"),
            LeftParen,
            RightParen,
            Space,
            LeftBrace,
            Space,
            Ident("SetPower"),
            LeftParen,
            Ident("OUT_A"),
            Comma,
            Space,
            LiteralInt("50"),
            RightParen,
            Semicolon,
            Space,
            Ident("OnFwd"),
            LeftParen,
            Ident("OUT_A"),
            RightParen,
            Semicolon,
            Space,
            Ident("Wait"),
            LeftParen,
            LiteralInt("100"),
            RightParen,
            Semicolon,
            Space,
            Ident("Off"),
            LeftParen,
            Ident("OUT_A"),
            RightParen,
            Semicolon,
            Space,
            RightBrace,
        ];
        assert_eq!(stream, expected);
    }
}
