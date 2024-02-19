use super::binfmt::RcxBin;
use std::{fmt::Write, path::Path};

pub fn print(file: &Path, bin: &RcxBin) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "Disassembly of `{}`", file.display());
    print_header(bin, &mut out);
    print_symbol_table(bin, &mut out);
    out
}

fn print_header(bin: &RcxBin, mut out: impl Write) {
    let _ = write!(
        out,
        "{} version {:x} targeting {:x}",
        String::from_utf8_lossy(&bin.signature),
        bin.version,
        bin.target_type,
    );
}

fn print_symbol_table(bin: &RcxBin, mut out: impl Write) {
    let _ = writeln!(out, "SYMBOLS:");
    for symbol in &bin.symbols {
        let _ = writeln!(
            out,
            "  {} {} {:?}",
            symbol.ty, symbol.index, symbol.name,
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;

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
        let expected = "RCXI version 102 targeting 0";
        assert_eq!(printed, expected);
    }
}
