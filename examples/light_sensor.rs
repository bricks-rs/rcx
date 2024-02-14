use rcx::{
    opcodes::PlaySound,
    tower::{usb::UsbTower, IrTower},
    MotorSelection, Rcx, SensorType, Sound, SourceType,
};

const DEVICE: &str = "/dev/usb/legousbtower0";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut rcx = UsbTower::open(DEVICE)?;

    let mut rcx = Rcx::new(rcx);

    rcx.set_sensor_type(0, SensorType::Light)?;

    loop {
        let data = rcx.get_value(SourceType::SensorValue, 0);
        dbg!(data);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
