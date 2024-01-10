use rcx::usbtower::UsbTower;

const DEVICE: &str = "/dev/usb/legousbtower0";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rcx = UsbTower::open(DEVICE)?;

    let msg = [0x51, 0x00];
    rcx.send(&msg)?;

    let resp = rcx.recv()?;
    println!("resp: {resp:02x?}");

    Ok(())
}
