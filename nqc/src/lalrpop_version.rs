use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
	nqc
);

pub mod ast {
    use std::fmt::{Display, Formatter};

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
}

#[cfg(test)]
mod test {
    use super::ast::*;
    use super::*;

    #[test]
    fn term() {
        for (case, expected) in
            [("22", Expr::Literal(22)), ("0", Expr::Literal(0))]
        {
            assert_eq!(*nqc::TermParser::new().parse(case).unwrap(), expected);
        }
    }

    #[test]
    fn expr() {
        for (case, expected) in [
            ("22", Expr::Literal(22)),
            ("(22)", Expr::Literal(22)),
            ("(((33)))", Expr::Literal(33)),
            ("game", Expr::Ident("game")),
            ("_game", Expr::Ident("_game")),
            ("_game", Expr::Ident("_game")),
            ("_g4me", Expr::Ident("_g4me")),
            (
                "2*3",
                Expr::BinaryOp(
                    Expr::Literal(2).into(),
                    BinaryOp::Mul,
                    Expr::Literal(3).into(),
                ),
            ),
            (
                "2/3",
                Expr::BinaryOp(
                    Expr::Literal(2).into(),
                    BinaryOp::Div,
                    Expr::Literal(3).into(),
                ),
            ),
            (
                "2*3",
                Expr::BinaryOp(
                    Expr::Literal(2).into(),
                    BinaryOp::Mul,
                    Expr::Literal(3).into(),
                ),
            ),
            (
                "2-3",
                Expr::BinaryOp(
                    Expr::Literal(2).into(),
                    BinaryOp::Sub,
                    Expr::Literal(3).into(),
                ),
            ),
            (
                "game-3",
                Expr::BinaryOp(
                    Expr::Ident("game").into(),
                    BinaryOp::Sub,
                    Expr::Literal(3).into(),
                ),
            ),
        ] {
            dbg!(case);
            assert_eq!(
                *nqc::ExprParser::new().parse(case).unwrap(),
                expected,
                "{case}"
            );
        }
    }

    #[test]
    fn game() {
        assert_eq!(
            nqc::AddSubOpParser::new().parse("+").unwrap(),
            BinaryOp::Add
        );
    }
}
