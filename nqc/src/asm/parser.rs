use crate::asm::ast::LabelOrOpcode;

//TODO move into lexer module
pub fn parse(input: &str) -> Vec<LabelOrOpcode> {
    let mut out = Vec::new();

    // TODO regexes etc.
    for line in input.lines() {
        if let Some(label) = line.strip_suffix(':') {
            out.push(LabelOrOpcode::Label(label));
        } else {
            let mut iter = line.split_whitespace();
            let opcode = iter.next().unwrap();
            let args = iter.map(|arg| arg.parse().unwrap()).collect();
            out.push(LabelOrOpcode::Opcode(opcode, args));
        }
    }

    out
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::asm::ast::*;

    #[test]
    fn label_or_opcode() {
        for (case, expected) in [
            ("label:", vec![LabelOrOpcode::Label("label")]),
            ("opcode 5", vec![LabelOrOpcode::Opcode("opcode", vec![5])]),
            (
                "opcode 5 6 7",
                vec![LabelOrOpcode::Opcode("opcode", vec![5, 6, 7])],
            ),
            (
                "opcode 255",
                vec![LabelOrOpcode::Opcode("opcode", vec![0xff])],
            ),
        ] {
            dbg!(case);
            assert_eq!(parse(case), expected,);
        }
    }

    // #[test]
    // fn opcode() {
    //     assert_eq!(
    //         asm::OpcodeParser::new().parse("game 3").unwrap(),
    //         LabelOrOpcode::Opcode("game", vec![5])
    //     );
    // }
}
