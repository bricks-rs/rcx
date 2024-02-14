use rcx::{tower::usb::UsbTower, Rcx, SensorType, SourceType};

const DEVICE: &str = "/dev/usb/legousbtower0";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let rcx = UsbTower::open(DEVICE)?;

    let mut rcx = Rcx::new(rcx);

    rcx.set_sensor_type(0, SensorType::Light)?;

    loop {
        let data = rcx.get_value(SourceType::SensorValue, 0);
        let _ = dbg!(data);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
