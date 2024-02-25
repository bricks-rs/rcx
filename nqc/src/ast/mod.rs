use crate::{
    error::{Error, ErrorKind, Result},
    lexer::{Keyword, Token, TokenKind, TokenStream},
    Span,
};

type Tokens<'src> = std::iter::Peekable<TokenStream<'src>>;

pub struct Ast<'src> {
    nodes: Vec<TopLevelNode<'src>>,
}

impl<'src> Ast<'src> {
    pub fn parse(tokens: TokenStream<'src>) -> Result<'src, Self> {
        let mut tokens = tokens.peekable();
        let mut nodes = Vec::new();
        while let Some(node) = TopLevelNode::parse(&mut tokens)? {
            nodes.push(node);
        }
        Ok(Self { nodes })
    }
}

pub enum TopLevelNode<'src> {
    Var(Var<'src>),
    Task(Task<'src>),
    Sub(Sub<'src>),
}

impl<'src> TopLevelNode<'src> {
    pub fn parse(tokens: &mut Tokens<'src>) -> Result<'src, Option<Self>> {
        Ok(match tokens.next() {
            None => None,
            Some(token) => {
                let token = token?;
                Some(match token.kind {
                    TokenKind::Kw(Keyword::Task) => {
                        Self::Task(Task::parse(tokens)?)
                    }
                    TokenKind::Kw(Keyword::Sub) => {
                        Self::Sub(Sub::parse(tokens)?)
                    }
                    _ => todo!(),
                })
            }
            _ => todo!(),
        })
    }
}

pub struct Var<'src> {
    span: Span,
    ident: &'src str,
}

pub struct Task<'src> {
    span: Span,
    ident: &'src str,
    nodes: Vec<Expr>,
}

impl<'src> Task<'src> {
    pub fn parse(tokens: &mut Tokens<'src>) -> Result<'src, Self> {
        todo!()
    }
}

pub struct Sub<'src> {
    span: Span,
    ident: &'src str,
    nodes: Vec<Expr>,
}

impl<'src> Sub<'src> {
    pub fn parse(tokens: &mut Tokens<'src>) -> Result<'src, Self> {
        todo!()
    }
}

pub struct Expr {
    span: Span,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_top_levels() {
        let src = "task main() {} sub s() {}";
        let stream = TokenStream::new(src);
        let ast = Ast::parse(stream);
    }
}
