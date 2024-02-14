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

    pub fn set_sensor_type(
        &mut self,
        sensor: u8,
        ty: SensorType,
    ) -> Result<()> {
        if sensor > 2 {
            return Err(Error::InvalidData("Sensor index must be 0-2"));
        }
        self.tower.send_recv(&opcodes::SetSensorType {
            sensor,
            type_: ty as u8,
        })?;
        Ok(())
    }

    pub fn set_time(&mut self, hours: u8, minutes: u8) -> Result<()> {
        if hours > 23 || minutes > 59 {
            return Err(Error::InvalidData(
                "Hours must be 0-23 and minutes must be 0-59",
            ));
        }
        self.tower.send_recv(&opcodes::SetTime { hours, minutes })?;
        Ok(())
    }

    pub fn set_transmitter_range(
        &mut self,
        range: TransmitterRange,
    ) -> Result<()> {
        self.tower
            .send_recv(&opcodes::SetTransmitterRange { range: range as u8 })?;
        Ok(())
    }

    pub fn start_firmware_download(
        &mut self,
        address: i16,
        checksum: i16,
    ) -> Result<opcodes::StartFirmwareDownloadResponse> {
        let resp = self.tower.send_recv(&opcodes::StartFirmwareDownload {
            address,
            checksum,
            unknown: 0,
        })?;
        opcodes::StartFirmwareDownloadResponse::deserialise(&resp)
    }

    pub fn start_subroutine_download(
        &mut self,
        subroutine: i16,
        length: i16,
    ) -> Result<opcodes::StartSubroutineDownloadResponse> {
        if subroutine > 7 {
            return Err(Error::InvalidData("Subroutine must be 0-7"));
        }
        let resp = self.tower.send_recv(&opcodes::StartSubroutineDownload {
            unknown: 0,
            subroutine,
            length,
        })?;
        opcodes::StartSubroutineDownloadResponse::deserialise(&resp)
    }

    pub fn start_task(&mut self, task: u8) -> Result<()> {
        if task > 9 {
            return Err(Error::InvalidData("Task must be 0-9"));
        }
        self.tower.send_recv(&opcodes::StartTask { task })?;
        Ok(())
    }

    pub fn start_task_download(&mut self, task: u8, length: i16) -> Result<()> {
        if task > 9 {
            return Err(Error::InvalidData("Task must be 0-9"));
        }
        self.tower.send_recv(&opcodes::StartTaskDownload {
            unknown: 0,
            task: i16::from(task),
            length,
        })?;
        Ok(())
    }

    pub fn stop_all_tasks(&mut self) -> Result<()> {
        self.tower.send_recv(&opcodes::StopAllTasks {})?;
        Ok(())
    }

    pub fn stop_task(&mut self, task: u8) -> Result<()> {
        if task > 9 {
            return Err(Error::InvalidData("Task must be 0-9"));
        }
        self.tower.send_recv(&opcodes::StopTask { task })?;
        Ok(())
    }

    pub fn transfer_data(
        &mut self,
        index: i16,
        length: i16,
        data: [u8; 256],
        checksum: u8,
    ) -> Result<opcodes::TransferDataResponse> {
        let resp = self.tower.send_recv(&opcodes::TransferData {
            index,
            length,
            data,
            checksum,
        })?;
        opcodes::TransferDataResponse::deserialise(&resp)
    }

    pub fn unlock_firmware(&mut self) -> Result<()> {
        let resp = self
            .tower
            .send_recv(&opcodes::UnlockFirmware { key: *b"LEGO\xae" })?;
        let resp = opcodes::UnlockFirmwareResponse::deserialise(&resp)?;
        if &resp.data == b"Just a bit off the block!" {
            Ok(())
        } else {
            Err(Error::RcxError("Unexpected response from brick"))
        }
    }
}
