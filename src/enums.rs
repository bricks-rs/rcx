use crate::Error;

/// This section describes the available sources and arguments.
///
/// Sources are like addressing modes. They specify where and how to get certain operand values.
///
/// There are 16 sources available, of which 13 apply to the RCX
#[repr(u8)]
pub enum SourceType {
    /// Returns value of specified variable.
    Variable = 0,
    /// Returns value of specified timer, in 1/100ths of a second.
    Timer = 1,
    /// Returns specified immediate value.
    Immediate = 2,
    /// Returns state of specified motor. See below.
    MotorState = 3,
    /// Returns random value, 0..max.
    Random = 4,
    // 5,6,7 reserved for Cybermaster
    /// Returns current program number.
    CurrentProgram = 8,
    /// Returns value of specified sensor.
    SensorValue = 9,
    /// Returns type of specified sensor.
    SensorType = 10,
    /// Returns mode of specified sensor.
    SensorMode = 11,
    /// Returns raw value of specified sensor, 0..1023.
    RawSensorValue = 12,
    /// Returns boolean value of specified sensor, 0..1.
    BooleanSensorValue = 13,
    /// Returns minutes since power on.
    Clock = 14,
    /// Returns value of message buffer.
    Message = 15,
}

/// Motor state is encoded as a single byte. Bits 0-2 contain the motor
/// power, 0..7. The remaining bits are used as follows:
/// ```text
/// Bit	Description	Notes
/// 0x08	Forward flag	0 if forward, 1 if reverse.
/// 0x40	Off flag	1 if off.
/// 0x80	On flag	1 if on.
///```
/// If both bit 0x40 and bit 0x80 are 0, the specified motor is set to
/// float.
pub struct MotorState {
    pub power: u8,
    pub direction: MotorDirection,
    pub state: MotorPowerState,
}

pub enum MotorDirection {
    Forward,
    Reverse,
}

pub enum MotorPowerState {
    On,
    Off,
    Float,
}

impl TryFrom<u8> for MotorState {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        const DIRECTION_BIT: u8 = 0x08;
        const OFF_FLAG: u8 = 0x40;
        const ON_FLAG: u8 = 0x80;

        let power = value & 0b0111;

        let direction = if value & DIRECTION_BIT == 0 {
            MotorDirection::Forward
        } else {
            MotorDirection::Reverse
        };

        let state = match (value & OFF_FLAG, value & ON_FLAG) {
            (0, 0) => MotorPowerState::Float,
            (_, 0) => MotorPowerState::Off,
            (0, _) => MotorPowerState::On,
            _ => panic!("Invalid motor power state"),
        };

        Ok(Self {
            power,
            direction,
            state,
        })
    }
}
