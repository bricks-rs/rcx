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
const TX_SEPARATION: Duration = Duration::from_millis(300);

pub struct UsbTower {
    device: File,
    use_alternate_opcode: bool,
    last_tx: Instant,
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
            last_tx: Instant::now(),
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

        let mut buf = [0; 1024];
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

        println!("send: {buf:02x?}");

        // Enforce a minimum time separation between transmissions to
        // avoid confusing the RCX
        while self.last_tx.elapsed() < TX_SEPARATION {
            std::thread::sleep(Duration::from_millis(5));
        }

        self.device.write_all(&buf)?;
        self.device.flush()?;
        self.last_tx = Instant::now();
        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![0; 256];
        let now = Instant::now();
        while now.elapsed() < READ_TIMEOUT {
            if let Ok(len) = self.device.read(&mut buf) {
                buf.truncate(len);
                return Ok(buf);
            }
        }
        Err(Error::Timeout)
    }
}
