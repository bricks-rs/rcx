pub mod opcodes;
pub mod tower;

mod display_impls;
mod enums;
mod errors;

pub use enums::*;
pub use errors::{Error, Result};

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

    pub fn get_memory_map(&mut self) -> Result<opcodes::GetMemoryMapResponse> {
        let resp = self.tower.send_recv(&opcodes::GetMemoryMap {})?;
        opcodes::GetMemoryMapResponse::deserialise(&resp)
    }

    pub fn get_value(
        &mut self,
        source: SourceType,
        argument: u8,
    ) -> Result<opcodes::GetValueResponse> {
        let resp = self.tower.send_recv(&opcodes::GetValue {
            source: source as u8,
            argument,
        })?;
        opcodes::GetValueResponse::deserialise(&resp)
    }

    pub fn get_versions(&mut self) -> Result<opcodes::GetVersionsResponse> {
        const KEY: [u8; 5] = [1, 3, 5, 7, 11];
        let resp = self.tower.send_recv(&opcodes::GetVersions { key: KEY })?;
        opcodes::GetVersionsResponse::deserialise(&resp)
    }

    pub fn play_sound(&mut self, sound: Sound) -> Result<()> {
        self.tower
            .send_recv(&opcodes::PlaySound { sound: sound as u8 })?;
        Ok(())
    }

    pub fn play_tone(
        &mut self,
        frequency_hz: i16,
        duration_cs: i8,
    ) -> Result<()> {
        self.tower.send_recv(&opcodes::PlayTone {
            frequency: frequency_hz,
            duration: duration_cs,
        })?;
        Ok(())
    }

    pub fn power_off(&mut self) -> Result<()> {
        self.tower.send_recv(&opcodes::PowerOff {})?;
        Ok(())
    }

    pub fn set_display(
        &mut self,
        source: SourceType,
        argument: u8,
    ) -> Result<()> {
        self.tower.send_recv(&opcodes::SetDisplay {
            source: source as u8,
            argument,
        })?;
        Ok(())
    }

    pub fn set_message(&mut self, message: u8) -> Result<()> {
        self.tower.send(&opcodes::SetMessage { message })
    }

    pub fn set_motor_direction(
        &mut self,
        motor: MotorSelection,
        direction: MotorDirection,
    ) -> Result<()> {
        let mut bitfield = motor.bitfield;
        if direction == MotorDirection::Forward {
            bitfield |= 0x80;
        }
        self.tower
            .send_recv(&opcodes::SetMotorDirection { code: bitfield })?;
        Ok(())
    }

    pub fn set_motor_on_off(
        &mut self,
        motor: MotorSelection,
        state: MotorPowerState,
    ) -> Result<()> {
        let mut code = motor.bitfield;
        match state {
            MotorPowerState::On => code |= 0x80,
            MotorPowerState::Off => code |= 0x40,
            MotorPowerState::Float => {}
        }
        self.tower.send_recv(&opcodes::SetMotorOnOff { code })?;
        Ok(())
    }

    pub fn set_motor_power(
        &mut self,
        motor: MotorSelection,
        power: u8,
    ) -> Result<()> {
        if power > 7 {
            return Err(Error::InvalidData("Motor power must be 0-7"));
        }
        self.tower.send_recv(&opcodes::SetMotorPower {
            motors: motor.bitfield,
            source: SourceType::Immediate as u8,
            argument: power,
        })?;
        Ok(())
    }

    pub fn set_power_down_delay(&mut self, minutes: u8) -> Result<()> {
        self.tower
            .send_recv(&opcodes::SetPowerDownDelay { minutes })?;
        Ok(())
    }

    pub fn set_program_number(&mut self, program: u8) -> Result<()> {
        if program > 4 {
            return Err(Error::InvalidData("Program number must be 0-4"));
        }
        self.tower
            .send_recv(&opcodes::SetProgramNumber { program })?;
        Ok(())
    }

    pub fn set_sensor_mode(
        &mut self,
        sensor: u8,
        mode: SensorMode,
    ) -> Result<()> {
        if sensor > 2 {
            return Err(Error::InvalidData("Sensor index must be 0-2"));
        }
        self.tower.send_recv(&opcodes::SetSensorMode {
            sensor,
            code: mode as u8,
        })?;
        Ok(())
    }
}
