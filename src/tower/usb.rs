use crate::{opcodes::Opcode, Error, IrTower, Result};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::FileTypeExt,
    path::Path,
    time::{Duration, Instant},
};

const HEADER: [u8; 2] = [0x55, 0xff];
const READ_TIMEOUT: Duration = Duration::from_secs(1);

pub struct UsbTower {
    device: File,
    use_alternate_opcode: bool,
}

impl UsbTower {
    pub fn open(device: impl AsRef<Path>) -> Result<Self> {
        let device = device.as_ref();

        let file_type = device.metadata()?.file_type();
        if !file_type.is_char_device() {
            return Err(Error::NotChardev);
        }

        let device = OpenOptions::new().read(true).write(true).open(device)?;
        Ok(Self {
            device,
            use_alternate_opcode: false,
        })
    }
}

impl IrTower for UsbTower {
    fn send(&mut self, msg: &dyn Opcode) -> Result<()> {
        let mut opcode = msg.request_opcode();
        if self.use_alternate_opcode {
            opcode |= 0x08;
        }
        self.use_alternate_opcode = !self.use_alternate_opcode;

        let mut buf = [0; 50];
        let msg = msg.serialise(&mut buf)?;
        let msg = &buf[..msg];
        let mut buf = Vec::<u8>::new();
        buf.extend_from_slice(&HEADER);
        let mut sum = 0u8;

        buf.push(opcode);
        buf.push(!opcode);
        sum += opcode;

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
        let now = Instant::now();
        while now.elapsed() < READ_TIMEOUT {
            if let Ok(len) = self.device.read_to_end(&mut buf) {
                return Ok(buf);
            }
        }
        Err(Error::Timeout)
    }
}
