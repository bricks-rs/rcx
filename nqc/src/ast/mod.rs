use crate::{
    error::{Error, ErrorKind, Result},
    lexer::{Keyword, Token, TokenKind, TokenStream, Tokens},
    Span,
};

// type Tokens<'src> = std::iter::Peekable<TokenStream<'src>>;

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
            other => panic!("Unhandled token '{:?}'", other),
        })
    }
}

#[derive(Debug)]
pub struct Var<'src> {
    pub span: Span,
    pub ident: &'src str,
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
pub struct Expr {
    span: Span,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn empty_top_levels() {
        let src = "task main() {} sub s() {}";
        let tokens = Tokens::new(src).unwrap();
        let stream = tokens.iter();
        let ast = Ast::parse(stream).unwrap();
        assert_debug_snapshot!(ast);
    }
}
