use crate::{Error, Result};
use std::{
    fmt::{self, Debug, Display, Formatter},
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
};

fn is_header(byte: u8) -> bool {
    [0x00, 0x55, 0xff].contains(&byte)
}

trait WriteParam {
    fn write_param(&self, buf: impl Write) -> io::Result<()>;
}

macro_rules! writeparamimpl {
    ($ty:ty) => {
        impl WriteParam for $ty {
            fn write_param(&self, mut buf: impl Write) -> io::Result<()> {
                buf.write_all(&self.to_le_bytes())
            }
        }
    };
}

writeparamimpl!(u8);
writeparamimpl!(i8);
writeparamimpl!(u16);
writeparamimpl!(i16);

impl<const N: usize> WriteParam for [u8; N] {
    fn write_param(&self, mut buf: impl Write) -> io::Result<()> {
        buf.write_all(self)
    }
}

impl WriteParam for Vec<u8> {
    fn write_param(&self, mut buf: impl Write) -> io::Result<()> {
        buf.write_all(self)
    }
}

trait ReadParam {
    fn read_param(buf: &mut impl Read) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! readparamimpl {
    ($ty:ty) => {
        impl ReadParam for $ty {
            fn read_param(buf: &mut impl Read) -> Result<Self> {
                const BYTES: usize = (<$ty>::BITS / 8) as usize;

                // read twice the requested number of bytes due to the
                // complement encoding
                let mut double_bytes = [0; BYTES * 2];
                buf.read_exact(&mut double_bytes)?;

                println!("parse: {:02x?}", double_bytes);

                // deduplicate the complements
                let mut bytes = [0; BYTES];
                for idx in 0..BYTES {
                    bytes[idx] = double_bytes[2 * idx];
                    if bytes[idx] != !double_bytes[2 * idx + 1] {
                        println!(
                            "{:02x} != {:02x}",
                            bytes[idx],
                            !double_bytes[2 * idx + 1]
                        );
                        return Err(Error::Checksum);
                    }
                }

                Ok(Self::from_le_bytes(bytes))
            }
        }
    };
}

readparamimpl!(u8);
readparamimpl!(i8);
readparamimpl!(u16);
readparamimpl!(i16);

fn read_byte(bin: &[u8], pc: &mut usize) -> Result<u8> {
    if let Some(byte) = bin.get(*pc) {
        *pc += 1;
        Ok(*byte)
    } else {
        Err(Error::InsufficientData)
    }
}

trait DisasmParam {
    fn disasm_param(bin: &[u8], pc: &mut usize) -> Result<Self>
    where
        Self: Sized;
}

impl DisasmParam for u8 {
    fn disasm_param(bin: &[u8], pc: &mut usize) -> Result<Self>
    where
        Self: Sized,
    {
        read_byte(bin, pc)
    }
}

impl DisasmParam for i8 {
    fn disasm_param(bin: &[u8], pc: &mut usize) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(i8::from_le_bytes([read_byte(bin, pc)?]))
    }
}

impl DisasmParam for u16 {
    fn disasm_param(bin: &[u8], pc: &mut usize) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(u16::from_le_bytes([
            read_byte(bin, pc)?,
            read_byte(bin, pc)?,
        ]))
    }
}

impl DisasmParam for i16 {
    fn disasm_param(bin: &[u8], pc: &mut usize) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(i16::from_le_bytes([
            read_byte(bin, pc)?,
            read_byte(bin, pc)?,
        ]))
    }
}

impl<const N: usize> DisasmParam for [u8; N] {
    fn disasm_param(bin: &[u8], pc: &mut usize) -> Result<Self>
    where
        Self: Sized,
    {
        let mut out = [0; N];
        for byte in out.iter_mut() {
            *byte = read_byte(bin, pc)?
        }
        Ok(out)
    }
}

trait AddToChecksum {
    fn add_to_checksum(&self, checksum: &mut u8);
}

macro_rules! add_to_checksum_impl {
    ($ty:ty) => {
        impl AddToChecksum for $ty {
            fn add_to_checksum(&self, checksum: &mut u8) {
                for byte in self.to_be_bytes() {
                    *checksum = checksum.wrapping_add(byte);
                }
            }
        }
    };
}

add_to_checksum_impl!(u8);
add_to_checksum_impl!(u16);
add_to_checksum_impl!(i16);

impl<const N: usize, T: AddToChecksum> AddToChecksum for [T; N] {
    fn add_to_checksum(&self, checksum: &mut u8) {
        for ele in self {
            ele.add_to_checksum(checksum);
        }
    }
}

impl<const N: usize, T: ReadParam + Default + Copy> ReadParam for [T; N] {
    fn read_param(buf: &mut impl Read) -> Result<Self>
    where
        Self: Sized,
    {
        let mut ret = [T::default(); N];
        for item in ret.iter_mut() {
            *item = T::read_param(buf)?;
        }

        Ok(ret)
    }
}

pub trait Opcode: Debug + Display {
    fn request_opcode(&self) -> u8;
    fn response_opcode(&self) -> Option<u8>;
    fn serialise(&self, buf: &mut [u8]) -> Result<usize>;
    fn disasm(bin: &[u8], pc: &mut usize) -> Result<Self>
    where
        Self: Sized;
    fn branch_target(&self) -> Option<u8> {
        None
    }
    fn is_unconditional_branch(&self) -> bool {
        false
    }
}

include!(concat!(env!("OUT_DIR"), "/opcodes.rs"));

#[cfg(test)]
mod test {
    use super::*;

    // NB serialisation does not include the opcode as that is handled
    // by separate logic which includes the alternate representation
    // selection

    #[test]
    fn play_sound_ser() {
        let op = PlaySound { sound: 2 };
        let mut buf = [0; 100];
        let len = op.serialise(&mut buf).unwrap();
        assert_eq!(&buf[..len], &[2]);
    }

    #[test]
    fn battery_response() {
        let buf = &[
            0x55, 0xff, 0x00, 0xcf, 0x30, 0x43, 0xbc, 0x1e, 0xe1, 0x30, 0xcf,
        ];
        let resp = GetBatteryPowerResponse::deserialise(buf).unwrap();
        assert_eq!(resp, GetBatteryPowerResponse { millivolts: 7747 });
    }
}
