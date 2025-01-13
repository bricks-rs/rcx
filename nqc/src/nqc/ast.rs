use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Stmt<'input> {
    Expr(Box<Expr<'input>>),
    FuncDecl(&'input str, Vec<Box<Expr<'input>>>),
}

impl Display for Stmt<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Expr(expr) => Display::fmt(expr, fmt),
            Self::FuncDecl(ident, expr) => {
                write!(fmt, "int {ident} {{\n    {expr:?}\n}}")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'input> {
    Literal(i32),
    Ident(&'input str),
    BinaryOp(Box<Expr<'input>>, BinaryOp, Box<Expr<'input>>),
}

impl Display for Expr<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Literal(val) => write!(fmt, "{val}"),
            Self::Ident(ident) => fmt.write_str(ident),
            Self::BinaryOp(left, op, right) => {
                write!(fmt, "{left} {op} {right}")
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl Display for BinaryOp {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
        })
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::lexer::Tokens;
//     use insta::{assert_debug_snapshot, assert_snapshot, glob};

//     #[test]
//     fn snapshot_tests() {
//         glob!("../../tests/good", "*.nqc", |path| {
//             let src = std::fs::read_to_string(path).unwrap();
//             let tokens = Tokens::new(&src).unwrap();
//             let stream = tokens.iter();
//             let ast = Ast::parse(stream).map_err(miette::Report::from).unwrap();
//             assert_debug_snapshot!(ast);
//         });
//     }

//     #[test]
//     fn error_snapshot_tests() {
//         std::env::set_var("NO_COLOR", "true");
//         glob!("../../tests/bad", "*.nqc", |path| {
//             let src = std::fs::read_to_string(path).unwrap();
//             let tokens = Tokens::new(&src).unwrap();
//             let stream = tokens.iter();
//             let err = miette::Report::from(Ast::parse(stream).unwrap_err());
//             assert_snapshot!(format!("{err:?}"));
//         });
//     }
// }
