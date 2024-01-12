pub mod opcodes;

pub mod tower;

mod errors;
pub use errors::{Error, Result};
mod enums;

use opcodes::{GetBatteryPower, GetBatteryPowerResponse, PlaySound};
use tower::IrTower;

pub struct Rcx {
    tower: Box<dyn IrTower>,
}

impl Rcx {
    pub fn new(tower: impl IrTower + 'static) -> Self {
        Self {
            tower: Box::new(tower),
        }
    }

    pub fn alive(&mut self) -> Result<()> {
        self.tower.send_recv(&opcodes::Alive {})?;
        Ok(())
    }

    pub fn get_battery_power(
        &mut self,
    ) -> Result<opcodes::GetBatteryPowerResponse> {
        let resp = self.tower.send_recv(&opcodes::GetBatteryPower {})?;
        opcodes::GetBatteryPowerResponse::deserialise(&resp)
    }
}
