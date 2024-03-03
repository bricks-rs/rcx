use crate::Span;
use std::fmt::{self, Display};

pub type Result<'src, T> = std::result::Result<T, Error<'src>>;

#[derive(Debug)]
pub struct Error<'src> {
    pub kind: ErrorKind,
    pub span: Span,
    raw: &'src str,
}

impl<'src> std::error::Error for Error<'src> {}

#[derive(Debug)]
pub enum ErrorKind {
    Syntax(String),
    Eof,
}

impl<'src> Error<'src> {
    pub fn new(
        start: usize,
        length: usize,
        kind: ErrorKind,
        raw: &'src str,
    ) -> Self {
        Self {
            kind,
            span: Span { start, length },
            raw,
        }
    }
}

impl Display for Error<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut line_start = 0;
        let mut line_count = 1;
        for (idx, chr) in self.raw.chars().enumerate() {
            if idx >= self.span.start {
                break;
            }
            if chr == '\n' {
                line_start = idx + 1;
                line_count += 1;
            }
        }
        // find end of current line
        let line_end = self
            .raw
            .chars()
            .enumerate()
            .skip(line_start)
            .find(|(_idx, chr)| *chr == '\n')
            .map(|(idx, _chr)| idx)
            .unwrap_or(self.raw.len());

        // write the error message
        let line_count_str = line_count.to_string();
        writeln!(fmt, "{:?} error on line {}:", self.kind, line_count)?;
        writeln!(fmt, "{}| {}", line_count, &self.raw[line_start..line_end])?;
        // write the span highlight
        let width = line_count_str.len() + 2 + self.span.start - line_start;
        dbg!(width);
        let length = self.span.length;
        // first arg pads with spaces up to the span position
        // second arg repeats the '^' character for the span length
        writeln!(fmt, "{:>width$}{:^<length$}", "", "")?;

        Ok(())
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
        let span = Span {
            start: 18,
            length: 5,
        };
        let error = Error {
            kind: ErrorKind::Syntax("an error".to_string()),
            span,
            raw: src,
        };
        assert_snapshot!(error.to_string());
    }
}
