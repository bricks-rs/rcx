use crate::lexer::Token;
use crate::lexer::TokenStream;
use miette::{Diagnostic, SourceSpan};
use std::{
    fmt,
    fmt::{Display, Formatter},
};

pub type Result<'src, T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Diagnostic)]
#[error("error")]
pub struct Error {
    pub kind: ErrorKind,
    #[label("{kind}")]
    pub span: SourceSpan,
    #[source_code]
    raw: String,
}

// impl<'src> std::error::Error for Eror<'src> {}

#[derive(Debug)]
pub enum ErrorKind {
    Syntax(String),
    Eof,
}

impl Display for ErrorKind {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::Syntax(err) => write!(fmt, "Syntax error: {err}"),
            Self::Eof => write!(fmt, "End of file reached"),
        }
    }
}

impl Error {
    pub fn new(
        start: usize,
        length: usize,
        kind: ErrorKind,
        raw: String,
    ) -> Self {
        Self {
            kind,
            span: (start, length).into(),
            raw,
        }
    }

    pub fn from_token(
        token: &Token,
        tokens: &TokenStream,
        err: impl Into<String>,
    ) -> Self {
        Self::new(
            token.span.start,
            token.span.length,
            ErrorKind::Syntax(err.into()),
            tokens.raw().into(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn err_span_display() {
        let src = "some text
with an error
somewhere in it";

        let error = Error {
            kind: ErrorKind::Syntax("an error".to_string()),
            span: (18, 5).into(),
            raw: src.into(),
        };
        let report = miette::Report::from(error);
        assert_snapshot!(format!("{report:?}"));
    }
}
