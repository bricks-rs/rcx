use rcx::{tower::usb::UsbTower, MotorSelection, Rcx};

const DEVICE: &str = "/dev/usb/legousbtower0";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let rcx = UsbTower::open(DEVICE)?;

    let mut rcx = Rcx::new(rcx);

    // dbg!(rcx.alive());
    // dbg!(rcx.get_battery_power());
    // dbg!(rcx.get_value(SourceType::SensorValue, 1));
    // println!("{}", rcx.get_versions()?);
    // rcx.play_sound(Sound::FastUpwardTones)?;
    // rcx.play_tone(440, 50)?;
    // rcx.power_off();

    rcx.set_motor_direction(MotorSelection::A, rcx::MotorDirection::Forward)?;
    rcx.set_motor_power(MotorSelection::A, 5)?;
    rcx.set_motor_on_off(MotorSelection::A, rcx::MotorPowerState::On)?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    rcx.set_motor_on_off(MotorSelection::A, rcx::MotorPowerState::Float)?;

    Ok(())
}
