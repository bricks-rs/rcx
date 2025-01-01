use crate::{
    error::{Error, Result},
    lexer::{Keyword, Token, TokenKind, TokenStream},
    Span,
};

fn get_ident<'src>(
    tokens: &mut TokenStream<'src>,
) -> Result<'src, (&'src Token<'src>, &'src str)> {
    let token = tokens.next_token()?;

    match &token.kind {
        TokenKind::Ident(ident) => Ok((token, ident)),
        _ => Err(Error::from_token(
            token,
            tokens,
            format!("invalid token {:?}, expected ident", token.kind),
        )),
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
    VarList(VarList<'src>),
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
                Self::VarList(VarList::parse(tokens)?)
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
pub struct VarList<'src> {
    pub list: Vec<Var<'src>>,
}

impl<'src> VarList<'src> {
    pub fn parse(tokens: &mut TokenStream<'src>) -> Result<'src, Self> {
        let mut list = Vec::new();

        loop {
            // variable declaration - only functions are void functions
            let (ident_token, ident) = get_ident(tokens)?;

            // either 'int var = 5;', 'int var;'
            let eq_or_paren = tokens.next_token()?;
            match &eq_or_paren.kind {
                TokenKind::Semicolon => {
                    list.push(Var {
                        span: eq_or_paren.span,
                        ident,
                        init: None,
                    });
                    break;
                }
                TokenKind::Eq => {
                    list.push(Var {
                        span: eq_or_paren.span,
                        ident,
                        init: Some(Expr::parse(tokens)?),
                    });
                    // check for semicolon or comma after expr
                    let next = tokens.next_token()?;
                    match next.kind {
                        TokenKind::Semicolon => break,
                        TokenKind::Comma => continue,
                        _ => {
                            return Err(Error::from_token(
                                next,
                                tokens,
                                "expected semicolon or comma \
                                    following init expr",
                            ))
                        }
                    }
                }
                TokenKind::Comma => {
                    // int var1, var2, var3;
                    //
                    // Push the current uninitialised var and keep going
                    list.push(Var {
                        span: eq_or_paren.span,
                        ident,
                        init: None,
                    });
                }
                TokenKind::LeftParen => {
                    return Err(Error::from_token(
                        ident_token,
                        tokens,
                        "only void functions are supported",
                    ));
                }
                other => {
                    return Err(Error::from_token(
                        eq_or_paren,
                        tokens,
                        format!(
                            "invalid token {:?}, expected var or fn",
                            other,
                        ),
                    ));
                }
            }
        }

        Ok(Self { list })
    }
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
            TokenKind::Kw(Keyword::Int) => Err(Error::from_token(
                token,
                tokens,
                "variable declaration is not permitted in this context",
            )),
            TokenKind::Ident(_ident) => {
                // var lvalue or func call
                todo!()
            }
            other => panic!("Unhandled token '{:?}'", other),
        }
    }
}

#[derive(Debug)]
pub struct Expr<'src> {
    pub span: Span,
    pub kind: ExprKind<'src>,
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
            (
                TokenKind::LiteralInt(val),
                Some(TokenKind::Semicolon | TokenKind::Comma),
            ) => {
                // don't consume the peeked semicolon
                // tokens.consume(TokenKind::Semicolon)?;
                Ok(Self {
                    span: first.span,
                    kind: ExprKind::LiteralInt(val),
                })
            }
            (other, _) => Err(Error::from_token(
                first,
                tokens,
                format!("Invalid token {:?}, expected var or fn", other,),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Tokens;
    use insta::{assert_debug_snapshot, assert_snapshot, glob};

    #[test]
    fn snapshot_tests() {
        glob!("../../tests/good", "*.nqc", |path| {
            let src = std::fs::read_to_string(path).unwrap();
            let tokens = Tokens::new(&src).unwrap();
            let stream = tokens.iter();
            let ast = Ast::parse(stream).map_err(miette::Report::from).unwrap();
            assert_debug_snapshot!(ast);
        });
    }

    #[test]
    fn error_snapshot_tests() {
        std::env::set_var("NO_COLOR", "true");
        glob!("../../tests/bad", "*.nqc", |path| {
            let src = std::fs::read_to_string(path).unwrap();
            let tokens = Tokens::new(&src).unwrap();
            let stream = tokens.iter();
            let err = miette::Report::from(Ast::parse(stream).unwrap_err());
            assert_snapshot!(format!("{err:?}"));
        });
    }
}
