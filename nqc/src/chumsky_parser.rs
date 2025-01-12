// use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::prelude::*;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Null,
    // Bool(bool),
    Int(i16),
    // Str(&'src str),
    Op(&'src str),
    Ctrl(char),
    Ident(&'src str),
    Kw(Keyword),
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Null => write!(f, "null"),
            // Token::Bool(x) => write!(f, "{x}"),
            Token::Int(n) => write!(f, "{n}"),
            // Token::Str(s) => write!(f, "{}", s),
            Token::Op(s) => write!(f, "{}", s),
            Token::Ctrl(c) => write!(f, "{}", c),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Kw(kw) => write!(f, "{kw}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, strum::EnumString, strum::Display)]
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

pub fn lexer<'src>() -> impl Parser<
    'src,
    &'src str,
    Vec<Spanned<Token<'src>>>,
    extra::Err<Rich<'src, char, Span>>,
> {
    // A parser for numbers
    let int = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Int);

    // A parser for operators
    let op = one_of("+*-/!=")
        .repeated()
        .at_least(1)
        .to_slice()
        .map(Token::Op);

    // A parser for control characters (delimiters, semicolons, etc.)
    let ctrl = one_of("()[]{};,").map(Token::Ctrl);

    // A parser for identifiers and keywords
    let ident = text::ident().map(|ident: &str| {
        if let Ok(kw) = ident.parse() {
            Token::Kw(kw)
        } else {
            Token::Ident(ident)
        }
    });

    // A single token can be one of the above
    let token = int.or(op).or(ctrl).or(ident);

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        // If we encounter an error, skip and attempt to lex the next
        // character as a token instead
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn basic_tokens() {
        use Token::*;

        let src = "()[]{}+-*/===while-() some_var
        //this is a comment
        //another comment
        a()";

        let (tokens, errs) = lexer().parse(src).into_output_errors();
        assert_eq!(errs, []);
        let tokens = tokens
            .unwrap()
            .into_iter()
            .map(|(token, _span)| token)
            .collect::<Vec<_>>();

        let expected = &[
            Ctrl('('),
            Ctrl(')'),
            Ctrl('['),
            Ctrl(']'),
            Ctrl('{'),
            Ctrl('}'),
            Op("+-*/==="),
            Kw(Keyword::While),
            Op("-"),
            Ctrl('('),
            Ctrl(')'),
            Ident("some_var"),
            Ident("a"),
            Ctrl('('),
            Ctrl(')'),
        ];
        assert_eq!(expected, tokens.as_slice());
    }

    #[test]
    fn simple_prog() {
        use Token::*;

        let src = "task main() {
            SetPower(OUT_A, 50);
            OnFwd(OUT_A);
            Wait(100);
            Off(OUT_A);
        }";

        let (tokens, errs) = lexer().parse(src).into_output_errors();
        assert_eq!(errs, []);
        let tokens = tokens
            .unwrap()
            .into_iter()
            .map(|(token, _span)| token)
            .collect::<Vec<_>>();

        let expected = &[
            Kw(Keyword::Task),
            Ident("main"),
            Ctrl('('),
            Ctrl(')'),
            Ctrl('{'),
            Ident("SetPower"),
            Ctrl('('),
            Ident("OUT_A"),
            Ctrl(','),
            Int(50),
            Ctrl(')'),
            Ctrl(';'),
            Ident("OnFwd"),
            Ctrl('('),
            Ident("OUT_A"),
            Ctrl(')'),
            Ctrl(';'),
            Ident("Wait"),
            Ctrl('('),
            Int(100),
            Ctrl(')'),
            Ctrl(';'),
            Ident("Off"),
            Ctrl('('),
            Ident("OUT_A"),
            Ctrl(')'),
            Ctrl(';'),
            Ctrl('}'),
        ];
        assert_eq!(expected, tokens.as_slice());
    }
}
