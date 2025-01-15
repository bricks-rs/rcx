use crate::asm::ast::LabelOrOpcode;
use regex::Regex;
use std::sync::LazyLock;

static LABEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*:$").unwrap());
static JMP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*jmp\s+[a-zA-Z_][a-zA-Z0-9_]*$").unwrap());
static OPCODE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*[a-zA-Z_][a-zA-Z0-9_]*(\s+((0x[0-9a-fA-F]+)|([0-9]+)))*$")
        .unwrap()
});

fn parse_number(inp: &str) -> u8 {
    if let Some(inp) = inp.strip_prefix("0x") {
        u8::from_str_radix(inp, 16).unwrap()
    } else {
        inp.parse().unwrap()
    }
}

pub fn parse(input: &str) -> Vec<LabelOrOpcode> {
    let mut out = Vec::new();
    let mut errors = Vec::new();

    for (idx, line) in input.lines().enumerate() {
        if LABEL_REGEX.is_match(line) {
            out.push(LabelOrOpcode::Label(line.strip_suffix(':').unwrap()));
        } else if JMP_REGEX.is_match(line) {
            out.push(LabelOrOpcode::Jmp(
                line.split_whitespace().nth(1).unwrap(),
            ));
        } else if OPCODE_REGEX.is_match(line) {
            let mut iter = line.split_whitespace();
            let opcode = iter.next().unwrap();
            let args = iter.map(parse_number).collect();
            out.push(LabelOrOpcode::Opcode(opcode, args));
        } else {
            errors.push((idx + 1, "Invalid line"));
        }
    }

    if !errors.is_empty() {
        panic!("{errors:?}");
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
                "opcode 5 6 0x7",
                vec![LabelOrOpcode::Opcode("opcode", vec![5, 6, 7])],
            ),
            (
                "opcode    0xff",
                vec![LabelOrOpcode::Opcode("opcode", vec![0xff])],
            ),
            ("jmp GAME", vec![LabelOrOpcode::Jmp("GAME")]),
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
