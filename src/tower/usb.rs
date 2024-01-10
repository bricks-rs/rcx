use crate::{opcodes::Opcode, IrTower, Result};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

const HEADER: [u8; 2] = [0x55, 0xff];

pub struct UsbTower {
    device: File,
}

impl UsbTower {
    pub fn open(device: impl AsRef<Path>) -> Result<Self> {
        let device = OpenOptions::new().read(true).write(true).open(device)?;
        Ok(Self { device })
    }
}

impl IrTower for UsbTower {
    fn send(&mut self, msg: &dyn Opcode) -> Result<()> {
        let mut buf = [0; 50];
        let msg = msg.serialise(&mut buf)?;
        let msg = &buf[..msg];
        let mut buf = Vec::<u8>::new();
        buf.extend_from_slice(&HEADER);
        let mut sum = 0u8;

        for &byte in msg {
            buf.push(byte);
            buf.push(!byte);
            sum = sum.wrapping_add(byte);
        }
        buf.push(sum);
        buf.push(!sum);

        println!("{buf:02x?}");

        self.device.write_all(&buf)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.device.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
