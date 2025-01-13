//! Parser for the .rcx binary format
//!
//! Referenced from: <https://github.com/BrickBot/nqc/blob/master/rcxlib/RCX_Image.cpp>
//!
//! ```text
//! * signature - 4 bytes
//! * version - 2 bytes
//! * chunks_count - 2 bytes
//! * symbol_count - 2 bytes
//! * target_type - 1 byte
//! * reserved - 1 byte
//! * for each chunk:
//!   - type - 1 byte (type <= 2)
//!   - number - 1 byte
//!   - length - 2 bytes
//!   - data - <length> bytes, padded to u32 alignment
//!  * for each symbol:
//!   - type - 1 byte
//!   - index - 1 byte
//!   - length - 1 byte
//!   - name - <length> bytes cstr
//! ```

use crate::{Error, Result};
use nom::{number::Endianness, IResult};
use std::{
    ffi::CString,
    fmt::{self, Debug, Display, Formatter, Write},
};
use tracing::trace;

const RCX_TAG: &str = "RCXI";
const MAX_SECTIONS: usize = 10;
const INDENT: &str = "  ";
const HEXDUMP_WRAP_BYTES: usize = 16;

fn print_hex_with_marker_at(bin: &[u8], pos: usize) -> String {
    let mut out = String::new();

    // header
    write!(&mut out, "     ").unwrap();
    for n in 0..16 {
        write!(&mut out, " {n:2x}").unwrap();
    }
    writeln!(&mut out).unwrap();

    // hexdump
    for (idx, chunk) in bin.chunks(HEXDUMP_WRAP_BYTES).enumerate() {
        write!(&mut out, "0x{:02x}:", idx * HEXDUMP_WRAP_BYTES,).unwrap();
        for byte in chunk {
            write!(&mut out, " {byte:02x}").unwrap();
        }
        writeln!(&mut out).unwrap();
        if (idx * HEXDUMP_WRAP_BYTES..(idx + 1) * HEXDUMP_WRAP_BYTES)
            .contains(&pos)
        {
            // indent the marker appropriately
            out += "     "; // indent past the offset display
            out.extend(std::iter::repeat("   ").take(pos % HEXDUMP_WRAP_BYTES));
            writeln!(&mut out, "^<<").unwrap();
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RcxBin {
    pub signature: [u8; 4],
    pub version: u16,
    pub section_count: u16,
    pub symbol_count: u16,
    pub target_type: TargetType,
    pub reserved: u8,
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
}

impl RcxBin {
    pub fn parse(bin: &[u8]) -> Result<Self> {
        let (_i, bin) = parse(bin).map_err(|err| {
            let input = match &err {
                nom::Err::Error(err) => err.input,
                nom::Err::Failure(err) => err.input,
                nom::Err::Incomplete(needed) => {
                    return Error::Nom(format!(
                        "Incomplete input, needed {needed:?}",
                    ));
                }
            };
            let pos = bin.len() - input.len();
            println!("{}", print_hex_with_marker_at(bin, pos));
            err.into()
        })?;
        bin.verify()?;
        Ok(bin)
    }

    pub fn verify(&self) -> Result<()> {
        fn repeated_idx(sections: &[Section]) -> bool {
            let mut c = sections
                .iter()
                .map(|c| (c.ty as u8, c.number))
                .collect::<Vec<_>>();
            c.sort_unstable();
            c.dedup();
            c.len() != sections.len()
        }

        // check chunk count
        if self.section_count as usize != self.sections.len()
            || self.sections.len() > MAX_SECTIONS
        {
            Err(Error::Parse("Invalid number of chunks"))
        } else if repeated_idx(&self.sections) {
            Err(Error::Parse("Nonunique chunk numbers"))
        } else {
            Ok(())
        }
    }
}

impl Display for RcxBin {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            fmt,
            "Signature: {}",
            String::from_utf8_lossy(&self.signature),
        )?;
        writeln!(fmt, "Version: {:x}", self.version)?;
        writeln!(
            fmt,
            "{} sections, {} symbols",
            self.section_count, self.symbol_count,
        )?;
        writeln!(fmt, "Target: {}", self.target_type)?;
        writeln!(fmt, "Sections:")?;
        for section in &self.sections {
            writeln!(fmt, "{section}")?;
        }
        writeln!(fmt, "Symbols:")?;
        for symbol in &self.symbols {
            writeln!(fmt, "{symbol}")?;
        }
        Ok(())
    }
}

fn parse(bin: &[u8]) -> IResult<&[u8], RcxBin> {
    trace!("Input len: {}", bin.len());
    let read_u16 = nom::number::complete::u16(Endianness::Little);
    let read_u8 = nom::number::complete::u8;

    let (i, signature) = nom::bytes::complete::tag(RCX_TAG)(bin)?;
    let (i, version) = read_u16(i)?;
    let (i, section_count) = read_u16(i)?;
    let (i, symbol_count) = read_u16(i)?;
    let (i, target_type) = TargetType::parse(i)?;
    let (i, reserved) = read_u8(i)?;

    trace!("Parse {section_count} sections");
    let (i, sections) =
        nom::multi::count(parse_section, section_count.into())(i)?;
    trace!("Parse {symbol_count} symbols");
    let (i, symbols) = nom::multi::count(parse_symbol, symbol_count.into())(i)?;

    IResult::Ok((
        i,
        RcxBin {
            signature: signature.try_into().unwrap_or([0; 4]),
            version,
            section_count,
            symbol_count,
            target_type,
            reserved,
            sections,
            symbols,
        },
    ))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TargetType {
    /// Original RCX (i.e., RCX bricks with v.0309 or earlier firmwares)
    Rcx = 0,
    /// CyberMaster
    CyberMaster,
    /// Scout
    Scout,
    /// RCX 2.0 (i.e., RCX bricks with v.0328 or later firmwares)
    Rcx2,
    /// Spybotics
    Spybotics,
    /// Dick Swan's alternate firmware
    Swan,
}

impl TargetType {
    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, ty) = nom::number::complete::u8(i)?;
        let ty = match ty {
            0 => Self::Rcx,
            1 => Self::CyberMaster,
            2 => Self::Scout,
            3 => Self::Rcx2,
            4 => Self::Spybotics,
            5 => Self::Swan,
            _ => {
                return Err(nom::Err::Failure(nom::error::Error {
                    input: i,
                    code: nom::error::ErrorKind::Verify,
                }));
            }
        };
        Ok((i, ty))
    }
}

impl Display for TargetType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, fmt)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub ty: SectionType,
    pub number: u8,
    pub length: u16,
    pub data: Vec<u8>,
}

fn parse_section(i: &[u8]) -> IResult<&[u8], Section> {
    let neg_offset = i.len();
    let read_u16 = nom::number::complete::u16(Endianness::Little);
    let read_u8 = nom::number::complete::u8;

    let (i, ty) = SectionType::parse(i)?;
    trace!("- section type {ty} - offset from back: {neg_offset}");
    let (i, number) = read_u8(i)?;
    trace!("  number {number}");
    trace!("  length raw: {:02x}{:02x}", i[1], i[0]);
    let (i, length) = read_u16(i)?;
    trace!("  length {length}");
    let (i, data) = nom::bytes::complete::take(length)(i)?;
    trace!("  data len {}", data.len());
    trace!("  data: {data:02x?}");

    // read padding bytes
    let (i, _pad) = nom::bytes::complete::take((4 - (length % 4)) & 3)(i)?;

    Ok((
        i,
        Section {
            ty,
            number,
            length,
            data: data.to_vec(),
        },
    ))
}

impl Display for Section {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        writeln!(fmt, "{INDENT}{} - {} bytes", self.ty, self.length)?;
        writeln!(fmt, "{INDENT}{INDENT}{}", hex::encode(&self.data))?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SectionType {
    Task = 0,
    Subroutine,
    Sound,
    Animation,
    Count,
}

impl SectionType {
    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, ty) = nom::number::complete::u8(i)?;
        let ty = match ty {
            0 => Self::Task,
            1 => Self::Subroutine,
            2 => Self::Sound,
            3 => Self::Animation,
            4 => Self::Count,
            _ => {
                return Err(nom::Err::Failure(nom::error::Error {
                    input: i,
                    code: nom::error::ErrorKind::Verify,
                }));
            }
        };
        Ok((i, ty))
    }
}

impl Display for SectionType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, fmt)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub ty: SymbolType,
    pub index: u8,
    pub length: u16,
    pub name: CString,
}

fn parse_symbol(i: &[u8]) -> IResult<&[u8], Symbol> {
    let read_u16 = nom::number::complete::u16(Endianness::Little);
    let read_u8 = nom::number::complete::u8;

    let (i, ty) = SymbolType::parse(i)?;
    trace!("Symbol type {ty}");
    let (i, index) = read_u8(i)?;
    let (i, length) = read_u16(i)?;
    let (i, name) = nom::bytes::complete::take(length)(i)?;

    Ok((
        i,
        Symbol {
            ty,
            index,
            length,
            name: CString::from_vec_with_nul(name.to_vec()).map_err(|_| {
                nom::Err::Failure(nom::error::Error {
                    input: i,
                    code: nom::error::ErrorKind::Alpha,
                })
            })?,
        },
    ))
}

impl Display for Symbol {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        writeln!(
            fmt,
            "{INDENT}{} at {} - {} bytes",
            self.ty, self.index, self.length
        )?;
        writeln!(fmt, "{INDENT}{INDENT}{:?}", self.name)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SymbolType {
    Task = 0,
    Sub,
    Var,
}

impl SymbolType {
    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, ty) = nom::number::complete::u8(i)?;
        let ty = match ty {
            0 => Self::Task,
            1 => Self::Sub,
            2 => Self::Var,
            _ => {
                return Err(nom::Err::Failure(nom::error::Error {
                    input: i,
                    code: nom::error::ErrorKind::Verify,
                }));
            }
        };
        Ok((i, ty))
    }
}

impl Display for SymbolType {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, fmt)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;

    const SAMPLE: &[u8] = &hex!(
        "5243584902010100010000000000 \
        140013070207e18713010232e1812181 \
        430264002141000005006d61696e00"
    );

    const COMPLEX: &[u8] = &hex!(
        "52435849020103000500000001000400e181218100000e0013070207e187
    130102321700710100000001330014000232001401020500130100002400
    00010085420059000008140102feff270d8502000b000006140102020043
    02640027a800010008007365745f66776400000005006d61696e0000010a
    006c6f6f705f7461736b0002000600706f776572000201060064656c7461
    00"
    );

    #[test]
    fn err_msg() {
        let out = print_hex_with_marker_at(COMPLEX, 5);
        let expected = "       0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
0x00: 52 43 58 49 02 01 03 00 05 00 00 00 01 00 04 00
                    ^<<
0x10: e1 81 21 81 00 00 0e 00 13 07 02 07 e1 87 13 01
0x20: 02 32 17 00 71 01 00 00 00 01 33 00 14 00 02 32
0x30: 00 14 01 02 05 00 13 01 00 00 24 00 00 01 00 85
0x40: 42 00 59 00 00 08 14 01 02 fe ff 27 0d 85 02 00
0x50: 0b 00 00 06 14 01 02 02 00 43 02 64 00 27 a8 00
0x60: 01 00 08 00 73 65 74 5f 66 77 64 00 00 00 05 00
0x70: 6d 61 69 6e 00 00 01 0a 00 6c 6f 6f 70 5f 74 61
0x80: 73 6b 00 02 00 06 00 70 6f 77 65 72 00 02 01 06
0x90: 00 64 65 6c 74 61 00
";
        assert_eq!(out, expected);

        let out = print_hex_with_marker_at(COMPLEX, 35);
        let expected = "       0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
0x00: 52 43 58 49 02 01 03 00 05 00 00 00 01 00 04 00
0x10: e1 81 21 81 00 00 0e 00 13 07 02 07 e1 87 13 01
0x20: 02 32 17 00 71 01 00 00 00 01 33 00 14 00 02 32
              ^<<
0x30: 00 14 01 02 05 00 13 01 00 00 24 00 00 01 00 85
0x40: 42 00 59 00 00 08 14 01 02 fe ff 27 0d 85 02 00
0x50: 0b 00 00 06 14 01 02 02 00 43 02 64 00 27 a8 00
0x60: 01 00 08 00 73 65 74 5f 66 77 64 00 00 00 05 00
0x70: 6d 61 69 6e 00 00 01 0a 00 6c 6f 6f 70 5f 74 61
0x80: 73 6b 00 02 00 06 00 70 6f 77 65 72 00 02 01 06
0x90: 00 64 65 6c 74 61 00
";
        assert_eq!(out, expected);
    }

    #[test]
    fn parse_sample() {
        let bin = RcxBin::parse(SAMPLE).unwrap();
        assert_eq!(
            bin,
            RcxBin {
                signature: *b"RCXI",
                version: 0x0102,
                section_count: 1,
                symbol_count: 1,
                target_type: TargetType::Rcx,
                reserved: 0,
                sections: vec![Section {
                    ty: SectionType::Task,
                    number: 0,
                    length: 20,
                    data: vec![
                        0x13, 0x7, 0x2, 0x7, 0xe1, 0x87, 0x13, 0x1, 0x2, 0x32,
                        0xe1, 0x81, 0x21, 0x81, 0x43, 0x2, 0x64, 0x0, 0x21,
                        0x41
                    ]
                }],
                symbols: vec![Symbol {
                    ty: SymbolType::Task,
                    index: 0,
                    length: 5,
                    name: CString::new("main").unwrap(),
                }],
            }
        );
    }

    #[test]
    fn parse_complex() {
        let bin = RcxBin::parse(COMPLEX).unwrap();
        assert_eq!(
            bin,
            RcxBin {
                signature: *b"RCXI",
                version: 0x0102,
                section_count: 3,
                symbol_count: 5,
                target_type: TargetType::Rcx,
                reserved: 0,
                sections: vec![
                    Section {
                        ty: SectionType::Subroutine,
                        number: 0,
                        length: 4,
                        data: vec![225, 129, 33, 129]
                    },
                    Section {
                        ty: SectionType::Task,
                        number: 0,
                        length: 14,
                        data: vec![
                            19, 7, 2, 7, 225, 135, 19, 1, 2, 50, 23, 0, 113, 1
                        ]
                    },
                    Section {
                        ty: SectionType::Task,
                        number: 1,
                        length: 51,
                        data: vec![
                            20, 0, 2, 50, 0, 20, 1, 2, 5, 0, 19, 1, 0, 0, 36,
                            0, 0, 1, 0, 133, 66, 0, 89, 0, 0, 8, 20, 1, 2, 254,
                            255, 39, 13, 133, 2, 0, 11, 0, 0, 6, 20, 1, 2, 2,
                            0, 67, 2, 100, 0, 39, 168
                        ]
                    },
                ],
                symbols: vec![
                    Symbol {
                        ty: SymbolType::Sub,
                        index: 0,
                        length: 8,
                        name: CString::new("set_fwd").unwrap()
                    },
                    Symbol {
                        ty: SymbolType::Task,
                        index: 0,
                        length: 5,
                        name: CString::new("main").unwrap()
                    },
                    Symbol {
                        ty: SymbolType::Task,
                        index: 1,
                        length: 10,
                        name: CString::new("loop_task").unwrap()
                    },
                    Symbol {
                        ty: SymbolType::Var,
                        index: 0,
                        length: 6,
                        name: CString::new("power").unwrap()
                    },
                    Symbol {
                        ty: SymbolType::Var,
                        index: 1,
                        length: 6,
                        name: CString::new("delta").unwrap()
                    }
                ],
            }
        );
    }
}
