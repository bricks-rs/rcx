use rcx::{
    opcodes::PlaySound,
    tower::{usb::UsbTower, IrTower},
    Rcx,
};

const DEVICE: &str = "/dev/usb/legousbtower0";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rcx = UsbTower::open(DEVICE)?;

    let msg = PlaySound { sound: 1 };
    dbg!(rcx.send(&msg)?);

    let resp = rcx.recv();
    println!("resp: {resp:02x?}");

    let mut rcx = Rcx::new(rcx);

    dbg!(rcx.alive());

    Ok(())
}
