use crate::{Error, Result};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

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

impl<const N: usize, T: ReadParam + Default + Copy> ReadParam for [T; N] {
    fn read_param(buf: &mut impl Read) -> Result<Self>
    where
        Self: Sized,
    {
        let mut ret = [T::default(); N];
        for idx in 0..N {
            ret[idx] = T::read_param(buf)?;
        }

        Ok(ret)
    }
}

pub trait Opcode {
    fn request_opcode(&self) -> u8;
    fn response_opcode(&self) -> Option<u8>;
    fn serialise(&self, buf: &mut [u8]) -> Result<usize>;
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
