use std::ops::BitOr;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MotorDirection {
    Forward,
    Reverse,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

/**
There are six avaiable sound types:
```text
    Index	Description
    0	Blip
    1	Beep beep
    2	Downward tones
    3	Upward tones
    4	Low buzz
    5	Fast upward tones
```
*/
#[repr(u8)]
pub enum Sound {
    Blip = 0,
    BeepBeep = 1,
    DownwardTones = 2,
    UpwardTones = 3,
    LowBuzz = 4,
    FastUpwardTones = 5,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MotorSelection {
    pub(crate) bitfield: u8,
}

impl MotorSelection {
    pub const A: Self = Self { bitfield: 0x01 };
    pub const B: Self = Self { bitfield: 0x02 };
    pub const C: Self = Self { bitfield: 0x04 };
}

impl BitOr for MotorSelection {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bitfield: self.bitfield | rhs.bitfield,
        }
    }
}

/**
    Set the slope and mode of sensor number sensor to the value specified by mode, and clear that sensor's value. The bits of mode are split into two portions. Bits 0-4 contain a slope value in 0..31, while bits 5-7 contain the mode, 0..7. The eight modes, which control the value returned by the sensor, are:
    ```text
        Mode	Name	Description
        0	Raw	Value in 0..1023.
        1	Boolean	Either 0 or 1.
        2	Edge count	Number of boolean transitions.
        3	Pulse count	Number of boolean transitions divided by two.
        4	Percentage	Raw value scaled to 0..100.
        5	Temperature in °C	1/10ths of a degree, -19.8..69.5.
        6	Temperature in °F	1/10ths of a degree, -3.6..157.1.
        7	Angle	1/16ths of a rotation, represented as a signed short.
    ```

    The slope value controls 0/1 detection for the three boolean modes. A slope of 0 causes raw sensor values greater than 562 to cause a transition to 0 and raw sensor values less than 460 to cause a transition to 1. The hysteresis prevents bouncing between 0 and 1 near the transition point. A slope value in 1..31, inclusive, causes a transition to 0 or to 1 whenever the difference between consecutive raw sensor values exceeds the slope. Increases larger than the slope result in 0 transitions, while decreases larger than the slope result in 1 transitions. Note the inversions: high raw values correspond to a boolean 0, while low raw values correspond to a boolean 1.
*/
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SensorMode {
    Raw = 0,
    Boolean,
    EdgeCount,
    PulseCount,
    Percentage,
    TemperatureC,
    TemperatureF,
    Angle,
}
