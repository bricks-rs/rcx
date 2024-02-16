# rcx
[![Crates.io Version](https://img.shields.io/crates/v/rcx)](https://crates.io/crates/rcx)
[![docs.rs](https://img.shields.io/docsrs/rcx)](https://docs.rs/rcx/)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/bricks-rs/rcx/rust.yml)](https://github.com/bricks-rs/rcx/actions)

Interface library for communicating with the LEGO Mindstorms RCX brick

## Example

```rust,no_run
use rcx::{tower::usb::UsbTower, MotorSelection, Rcx};

const DEVICE: &str = "/dev/usb/legousbtower0";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let rcx = UsbTower::open(DEVICE)?;

    let mut rcx = Rcx::new(rcx);

    rcx.set_motor_direction(MotorSelection::A, rcx::MotorDirection::Forward)?;
    rcx.set_motor_power(MotorSelection::A, 5)?;
    rcx.set_motor_on_off(MotorSelection::A, rcx::MotorPowerState::On)?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    rcx.set_motor_on_off(MotorSelection::A, rcx::MotorPowerState::Float)?;

    Ok(())
}
```