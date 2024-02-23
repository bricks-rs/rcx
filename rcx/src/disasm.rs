use crate::{
    binfmt::{RcxBin, Section, SectionType, SymbolType},
    opcodes::{Opcode, Opcodes},
};
use std::{collections::HashSet, fmt::Write, path::Path};

#[derive(Debug)]
struct Instruction {
    offset: usize,
    opcode: Opcodes,
}

#[must_use = "This function returns the disassembly as a string"]
pub fn print(file: &Path, bin: &RcxBin) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "Disassembly of `{}`", file.display());
    print_header(bin, &mut out);
    print_symbol_table(bin, &mut out);
    print_sections(bin, &mut out);
    out
}

fn print_header(bin: &RcxBin, mut out: impl Write) {
    let _ = writeln!(
        out,
        "{} version {:x} targeting {}",
        String::from_utf8_lossy(&bin.signature),
        bin.version,
        bin.target_type,
    );
}

fn print_symbol_table(bin: &RcxBin, mut out: impl Write) {
    let _ = writeln!(out, ".SYMBOLS:");
    for symbol in &bin.symbols {
        let _ = writeln!(
            out,
            "  {} {} {:?}",
            symbol.ty, symbol.index, symbol.name,
        );
    }
}

fn print_sections(bin: &RcxBin, mut out: impl Write) {
    for section in &bin.sections {
        print_section(section, bin, &mut out);
    }
}

fn print_section(section: &Section, bin: &RcxBin, out: &mut impl Write) {
    let sect_name = match section.ty {
        SectionType::Task => bin.symbols.iter().find(|sym| {
            sym.ty == SymbolType::Task && sym.index == section.number
        }),
        SectionType::Subroutine => bin.symbols.iter().find(|sym| {
            sym.ty == SymbolType::Sub && sym.index == section.number
        }),
        _ => None,
    };
    let _ = writeln!(
        out,
        "\n.SECTION {}",
        sect_name
            .map(|sym| format!("{:?}", sym.name))
            .unwrap_or_default()
    );

    println!("{:02x?}", section.data);

    let section_disasm = disasm_code_section(&section.data);
    for instr in section_disasm {
        let _ = writeln!(out, "{:02x}: {}", instr.offset, instr.opcode);
    }
}

fn disasm_code_section(section: &[u8]) -> Vec<Instruction> {
    let mut out = Vec::new();
    let mut pc = 0;
    let mut seen_offsets = HashSet::new();
    while pc < section.len() {
        let opcode;
        opcode = match crate::opcodes::parse_opcode(section, &mut pc) {
            Ok(opcode) => opcode,
            Err(e) => {
                eprintln!("{e}");
                eprintln!("[{:02x}] {:02x?}", pc, &section[pc..]);
                break;
            }
        };

        seen_offsets.insert(pc);
        out.push(Instruction { offset: pc, opcode });
    }
    out
}

/// If the opcode is a branch then returns its offset
fn is_branch(opcode: &dyn Opcode) -> Option<usize> {
    None
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;
    use pretty_assertions::assert_eq;

    const PROG: &[u8] = &hex!(
        "52435849020103000500000001000400e181218100000e0013070207e187
    130102321700710100000001330014000232001401020500130100002400
    00010085420059000008140102feff270d8502000b000006140102020043
    02640027a800010008007365745f66776400000005006d61696e0000010a
    006c6f6f705f7461736b0002000600706f776572000201060064656c7461
    00"
    );

    #[test]
    fn test_header() {
        let bin = RcxBin::parse(PROG).unwrap();
        let mut printed = String::new();
        print_header(&bin, &mut printed);
        let expected = "RCXI version 102 targeting Rcx\n";
        assert_eq!(printed, expected);
    }

    #[test]
    fn test_symbol_table() {
        let bin = RcxBin::parse(PROG).unwrap();
        let mut printed = String::new();
        print_symbol_table(&bin, &mut printed);
        let expected = r#".SYMBOLS:
  Sub 0 "set_fwd"
  Task 0 "main"
  Task 1 "loop_task"
  Var 0 "power"
  Var 1 "delta"
"#;
        assert_eq!(printed, expected);
    }
}
