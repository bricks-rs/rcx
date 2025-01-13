use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
	asm
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::asm::ast::*;

    #[test]
    fn label_or_opcode() {
        for (case, expected) in [
            ("label:", vec![LabelOrOpcode::Label("label")]),
            ("opcode 5\n", vec![LabelOrOpcode::Opcode("opcode", vec![])]),
        ] {
            assert_eq!(
                asm::LabelOrOpcodesParser::new().parse(case).unwrap(),
                expected
            );
        }
    }
}
