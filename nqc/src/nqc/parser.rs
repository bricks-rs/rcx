use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
	nqc
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::nqc::ast::*;

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
            (
                "game-3 * 6",
                Expr::BinaryOp(
                    Expr::Ident("game").into(),
                    BinaryOp::Sub,
                    Expr::BinaryOp(
                        Expr::Literal(3).into(),
                        BinaryOp::Mul,
                        Expr::Literal(6).into(),
                    )
                    .into(),
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
    fn stmt() {
        for (case, expected) in [
            ("22;", Stmt::Expr(Expr::Literal(22).into())),
            ("(22);", Stmt::Expr(Expr::Literal(22).into())),
            (
                "(((33))+3);",
                Stmt::Expr(
                    Expr::BinaryOp(
                        Expr::Literal(33).into(),
                        BinaryOp::Add,
                        Expr::Literal(3).into(),
                    )
                    .into(),
                ),
            ),
            (
                "int game() {43;}",
                Stmt::FuncDecl("game", vec![Expr::Literal(43).into()]),
            ),
        ] {
            dbg!(case);
            assert_eq!(
                *nqc::StmtParser::new().parse(case).unwrap(),
                expected,
                "{case}"
            );
        }
    }

    #[test]
    fn block() {
        let cases: &[(&str, &[Box<Expr>])] = &[
            ("22;", &[Expr::Literal(22).into()]),
            ("(22);", &[Expr::Literal(22).into()]),
            (
                "(((33))+3);",
                &[Expr::BinaryOp(
                    Expr::Literal(33).into(),
                    BinaryOp::Add,
                    Expr::Literal(3).into(),
                )
                .into()],
            ),
            (
                "22;33; 44;",
                &[
                    Expr::Literal(22).into(),
                    Expr::Literal(33).into(),
                    Expr::Literal(44).into(),
                ],
            ),
        ];

        for (case, expected) in cases {
            dbg!(case);
            assert_eq!(
                *nqc::BlockParser::new().parse(case).unwrap(),
                **expected,
                "{case}"
            );
        }
    }

    #[test]
    fn func() {
        let case = "int game() {
            22;
            33;
        }";
        let expected = Stmt::FuncDecl(
            "game",
            vec![Expr::Literal(22).into(), Expr::Literal(33).into()],
        );
        assert_eq!(
            *nqc::StmtParser::new().parse(case).unwrap(),
            expected,
            "{case}"
        );
    }
}
