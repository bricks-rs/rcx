use crate::{
    binfmt::{RcxBin, Section, SectionType, SymbolType},
    opcodes::{Opcode, Opcodes},
};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter, Write},
    path::Path,
};

#[derive(Debug)]
struct Instruction {
    offset: usize,
    opcode: Opcodes,
    branch_target: Option<BranchType>,
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

    let mut section_disasm = disasm_code_section(&section.data);
    section_disasm.sort_unstable_by_key(|instr| instr.offset);
    for instr in section_disasm {
        let target = instr
            .branch_target
            .map(|tgt| format!(" => {tgt}"))
            .unwrap_or_default();
        let mut buf = [0u8; 10];
        let len = instr.opcode.serialise(&mut buf).unwrap();
        let hex_source = hex::encode(&buf[..len]);
        let _ = writeln!(
            out,
            "{:02x}: {:02x} {}{}    {:02x}{}",
            instr.offset,
            instr.opcode.request_opcode(),
            instr.opcode,
            target,
            instr.opcode.request_opcode(),
            hex_source,
        );
    }
}

fn disasm_code_section(section: &[u8]) -> Vec<Instruction> {
    let mut out = BTreeMap::new();
    let mut pc = 0;
    let mut branch_instructions_to_go_back_to = Vec::new();
    while pc < section.len() {
        if out.contains_key(&pc) {
            println!("Seen {:02x}@{:02x} previously", section[pc], pc);

            if let Some(next) = branch_instructions_to_go_back_to.pop() {
                pc = next;
                continue;
            } else {
                break;
            }
        }
        let start = pc;

        let opcode = match crate::opcodes::parse_opcode(section, &mut pc) {
            Ok(opcode) => opcode,
            Err(e) => {
                eprintln!("{e}");
                eprintln!("[{:02x}] {:02x?}", pc, &section[pc..]);
                if branch_instructions_to_go_back_to.is_empty() {
                    break;
                } else {
                    // just checked is_empty, so this unwrap will never
                    // go off
                    pc = branch_instructions_to_go_back_to.pop().unwrap();
                    continue;
                }
            }
        };

        let branch_target = is_branch(&opcode, pc);
        out.insert(
            start,
            Instruction {
                offset: start,
                opcode,
                branch_target,
            },
        );

        match branch_target {
            Some(BranchType::Unconditional(target)) => {
                pc = target;
            }
            Some(BranchType::Conditional(target)) => {
                branch_instructions_to_go_back_to.push(pc);
                pc = target;
            }
            None => {}
        }
    }
    out.into_values().collect()
}

#[derive(Copy, Clone, Debug)]
enum BranchType {
    Conditional(usize),
    Unconditional(usize),
}

impl Display for BranchType {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        let target = match self {
            Self::Conditional(target) => target,
            Self::Unconditional(target) => target,
        };
        write!(fmt, "{target:02x}")
    }
}

/// If the opcode is a branch then returns its offset
fn is_branch(opcode: &Opcodes, pc: usize) -> Option<BranchType> {
    Some(match opcode {
        Opcodes::BranchAlwaysFar(opcode) => {
            // pc has already been advanced to the address of extension,
            // so the offset is one byte earlier
            let address_of_offset = pc - 1;
            let target = if opcode.offset & 0x80 == 0 {
                address_of_offset
                    + usize::from(opcode.offset)
                    + 128 * usize::from(opcode.extension)
            } else {
                address_of_offset + 128
                    - usize::from(opcode.offset)
                    - 128 * usize::from(opcode.extension)
            };
            BranchType::Unconditional(target)
        }
        Opcodes::BranchAlwaysNear(opcode) => {
            let address_of_offset = pc;
            let target = if opcode.offset & 0x80 == 0 {
                address_of_offset + usize::from(opcode.offset)
            } else {
                address_of_offset + 128 - usize::from(opcode.offset)
            };
            BranchType::Unconditional(target)
        }
        Opcodes::DecrementLoopCounterFar(_opcode) => {
            todo!()
        }
        Opcodes::DecrementLoopCounterNear(_opcode) => {
            todo!()
        }
        Opcodes::TestAndBranchFar(_opcode) => {
            todo!()
        }
        Opcodes::TestAndBranchNear(opcode) => {
            let address_of_offset = pc - 1;
            let target = address_of_offset + usize::from(opcode.offset);
            BranchType::Conditional(target)
        }
        _ => None?,
    })
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
