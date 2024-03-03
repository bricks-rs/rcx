use crate::{
    error::{Error, ErrorKind, Result},
    lexer::{Keyword, Token, TokenKind, TokenStream},
    Span,
};

fn get_ident<'src>(
    tokens: &mut TokenStream<'src>,
) -> Result<'src, (&'src Token<'src>, &'src str)> {
    match tokens.next() {
        None => panic!(),
        Some(token) => match &token.kind {
            TokenKind::Ident(ident) => Ok((token, ident)),
            other => panic!("Unhandled token '{:?}'", other),
        },
    }
}

#[derive(Debug)]
pub struct Ast<'src> {
    pub nodes: Vec<TopLevelNode<'src>>,
}

impl<'src> Ast<'src> {
    pub fn parse(mut tokens: TokenStream<'src>) -> Result<'src, Self> {
        let mut nodes = Vec::new();
        while !tokens.eof() {
            let node = TopLevelNode::parse(&mut tokens)?;
            nodes.push(node);
        }
        Ok(Self { nodes })
    }
}

#[derive(Debug)]
pub enum TopLevelNode<'src> {
    Var(Var<'src>),
    Task(Task<'src>),
    Sub(Sub<'src>),
}

impl<'src> TopLevelNode<'src> {
    pub fn parse(tokens: &mut TokenStream<'src>) -> Result<'src, Self> {
        let token = tokens.next_token()?;
        Ok(match &token.kind {
            TokenKind::Kw(Keyword::Task) => Self::Task(Task::parse(tokens)?),
            TokenKind::Kw(Keyword::Sub) => Self::Sub(Sub::parse(tokens)?),
            TokenKind::Kw(Keyword::Int) => {
                // could be var decl or function decl
                let ident_token = tokens.next_token()?;
                let TokenKind::Ident(ident) = ident_token.kind else {
                    return Err(Error::new(
                        ident_token.span.start,
                        ident_token.span.length,
                        ErrorKind::Syntax(format!(
                            "Invalid token {:?}, expected ident",
                            ident_token.kind
                        )),
                        tokens.raw(),
                    ));
                };

                // either 'int var = 5;', 'int var;', or 'int fn(){}'
                let eq_or_paren = tokens.next_token()?;
                match &eq_or_paren.kind {
                    TokenKind::Semicolon => Self::Var(Var {
                        span: eq_or_paren.span,
                        ident,
                        init: None,
                    }),
                    TokenKind::Eq => Self::Var(Var {
                        span: eq_or_paren.span,
                        ident,
                        init: Some(Expr::parse(tokens)?),
                    }),
                    TokenKind::LeftParen => todo!("fn decl"),
                    other => {
                        return Err(Error::new(
                            ident_token.span.start,
                            ident_token.span.length,
                            ErrorKind::Syntax(format!(
                                "Invalid token {:?}, expected var or fn",
                                other,
                            )),
                            tokens.raw(),
                        ));
                    }
                }
            }
            other => panic!("Unhandled token '{:?}'", other),
        })
    }
}

#[derive(Debug)]
pub struct Var<'src> {
    pub span: Span,
    pub ident: &'src str,
    pub init: Option<Expr<'src>>,
}

#[derive(Debug)]
pub struct Task<'src> {
    pub span: Span,
    pub ident: &'src str,
    pub block: Block<'src>,
}

impl<'src> Task<'src> {
    pub fn parse(tokens: &mut TokenStream<'src>) -> Result<'src, Self> {
        let (token, ident) = get_ident(tokens)?;
        // pull out parens around arg str (but no args are permitted)
        tokens.consume(TokenKind::LeftParen)?;
        tokens.consume(TokenKind::RightParen)?;
        let block = Block::parse(token.span, tokens)?;
        Ok(Self {
            span: token.span,
            ident,
            block,
        })
    }
}

#[derive(Debug)]
pub struct Sub<'src> {
    pub span: Span,
    pub ident: &'src str,
    pub block: Block<'src>,
}

impl<'src> Sub<'src> {
    pub fn parse(tokens: &mut TokenStream<'src>) -> Result<'src, Self> {
        let (token, ident) = get_ident(tokens)?;
        tokens.consume(TokenKind::LeftParen)?;
        tokens.consume(TokenKind::RightParen)?;
        let block = Block::parse(token.span, tokens)?;
        Ok(Self {
            span: token.span,
            ident,
            block,
        })
    }
}

#[derive(Debug)]
pub struct Block<'src> {
    pub span: Span,
    pub nodes: Vec<Stmt<'src>>,
}

impl<'src> Block<'src> {
    pub fn parse(
        span: Span,
        tokens: &mut TokenStream<'src>,
    ) -> Result<'src, Self> {
        let mut nodes = Vec::new();

        tokens.consume(TokenKind::LeftBrace)?;

        // parse statements until closing brace is reached
        loop {
            if tokens.peek().map(|tok| &tok.kind)
                == Some(&TokenKind::RightBrace)
            {
                break;
            }

            nodes.push(Stmt::parse(tokens)?);
        }

        tokens.consume(TokenKind::RightBrace)?;

        Ok(Self { span, nodes })
    }
}

#[derive(Debug)]
pub struct Stmt<'src> {
    pub span: Span,
    pub kind: StmtKind<'src>,
}

#[derive(Debug)]
pub enum StmtKind<'src> {
    VarDecl { ident: &'src str },
}

impl<'src> Stmt<'src> {
    pub fn parse<'tok>(
        tokens: &'tok mut TokenStream<'src>,
    ) -> Result<'src, Self> {
        let token = tokens.next_token()?;
        match &token.kind {
            TokenKind::Kw(Keyword::Int) => panic!(),
            other => panic!("Unhandled token '{:?}'", other),
        }
    }
}

#[derive(Debug)]
pub struct Expr<'src> {
    span: Span,
    kind: ExprKind<'src>,
}

#[derive(Debug)]
pub enum ExprKind<'src> {
    LiteralInt(&'src str),
}

impl<'src> Expr<'src> {
    pub fn parse(tokens: &mut TokenStream<'src>) -> Result<'src, Self> {
        let first = tokens.next_token()?;
        let next = tokens.peek();
        let next_kind = next.as_ref().map(|tok| &tok.kind);
        match (&first.kind, next_kind) {
            (TokenKind::LiteralInt(val), Some(TokenKind::Semicolon)) => {
                // consume the peeked semicolon
                tokens.consume(TokenKind::Semicolon)?;
                Ok(Self {
                    span: first.span,
                    kind: ExprKind::LiteralInt(val),
                })
            }
            (other, _) => Err(Error::new(
                first.span.start,
                first.span.length,
                ErrorKind::Syntax(format!(
                    "Invalid token {:?}, expected var or fn",
                    other,
                )),
                tokens.raw(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Tokens;
    use insta::{assert_debug_snapshot, glob};

    #[test]
    fn snapshot_tests() {
        glob!("../../tests", "*.nqc", |path| {
            let src = std::fs::read_to_string(path).unwrap();
            let tokens = Tokens::new(&src).unwrap();
            let stream = tokens.iter();
            let ast = Ast::parse(stream).unwrap();
            assert_debug_snapshot!(ast);
        });
    }
}
