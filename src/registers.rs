// For the i2c register reference, see datasheet: https://www.azoteq.com/images/stories/pdf/iqs231a_datasheet.pdf (pg. 14 and pg. 30 onwards)
use core::ops::Deref;
use num_enum::TryFromPrimitive;

use crate::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Struct for the values returned byt the sensor, wrapping the always returned `main_events` and the value itself
pub struct RegValue<T> {
    pub main_events: MainEvents,
    pub value: T,
}

impl<T> RegValue<T> {
    pub fn split(self) -> (MainEvents, T) {
        (self.main_events, self.value)
    }

    pub(crate) fn map<B, F>(self, f: F) -> RegValue<B>
    where
        F: FnOnce(T) -> B,
    {
        RegValue {
            value: f(self.value),
            main_events: self.main_events,
        }
    }
}

impl RegValue<u8> {
    pub fn into<T: From<u8>>(self) -> T {
        T::from(self.value)
    }
}

impl<T> Deref for RegValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> From<RegValue<T>> for MainEvents {
    fn from(rv: RegValue<T>) -> Self {
        rv.main_events
    }
}

impl From<RegValue<u8>> for u8 {
    fn from(value: RegValue<u8>) -> Self {
        value.value
    }
}

impl From<[u8; 2]> for RegValue<u8> {
    fn from(bytes: [u8; 2]) -> Self {
        Self {
            main_events: MainEvents::from_bits_retain(bytes[0]),
            value: bytes[1],
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive)]
#[allow(dead_code, non_camel_case_types)]
pub enum Register {
    ProductNumber = 0x00,   // R 0x40 0x40
    SoftwareVersion = 0x01, // R 0x06 0x06 (IQS231A), 0x07 (IQS231B – Identical to 0x06 software)
    DebugEvents = 0x02,     // R n/a RESERVED ATI_ERROR CH0_ATI RESERVED QUICK
    Reserved = 0x03,        // R/W n/a RESERVED
    Commands = 0x04,        // R/W 0x00 ATI_CH0 DISABLE
    OtpBank1 = 0x05,        // R/W 0x00 Standalone / I2C address Proximity threshold
    OtpBank2 = 0x06,        // R/W 0x00 Increase
    OtpBank3 = 0x07,        // R/W 0x00 Charge transfer frequency Temperature
    QuickRelease = 0x08,    // R/W 0x00 Quick release threshold LUT Quick release beta
    Movement = 0x09,        // R/W 0x34
    TouchThreshold = 0x0A,  // R/W 0x07
    ProximityThreshold = 0x0B,
    TempInterferenceThreshold = 0x0C,
    CH0_Multipliers = 0x0D, // R/W n/a Reserved Reserved CH0 Sensitivity Multiplier CH0 Compensation multiplier
    CH0_Compensation = 0x0E, // R/W n/a 0 – 255
    CH1_Multipliers = 0x0F, // R/W n/a Reserved Reserved CH1 Sensitivity Multiplier CH1 Compensation multiplier
    CH1_Compensation = 0x10, // R/W n/a 0 – 255
    System_Flags = 0x11, // R n/a I2C TEMP CH1_ACTIVE CURRENT_CH NO SYNC CH0_LTA_HALTED ATI_MODE ZOOM MODE
    UI_Flags = 0x12,     // R n/a TEMP
    ATI_Flags = 0x13,    // R n/a Reserved
    EventFlags = 0x14,   // R n/a CH1_ATI
    CH0_ACF_H = 0x15,    // R n/a Proximity channel: Filtered count value
    CH0_ACF_L = 0x16,    // R n/a
    CH0_LTA_H = 0x17,    // R n/a Proximity channel: Reference count value (Long term average)
    CH0_LTA_L = 0x18,    // R n/a
    CH0_QRD_H = 0x19,    // R n/a Proximity channel: Quick release detect reference value
    CH0_QRD_L = 0x1A,    // R n/a 0 – 2000
    CH1_ACF_H = 0x1B,    // R n/a Movement channel: Filtered count value
    CH1_ACF_L = 0x1C,    // R n/a 0 – 2000
    CH1_UMOV_H = 0x1D,   // R n/a Movement channel: Upper reference count value
    CH1_UMOV_L = 0x1E,   // R n/a 0 – 2000
    CH1_LMOV_H = 0x1F,   // R n/a Movement channel: Lower reference count value
    CH1_LMOV_L = 0x20,   // R n/a 0 – 2000
    CH1_RAW_H = 0x21, // R n/a Temperature channel: Unfiltered count value (if temperature feature enabled)
    CH1_RAW_L = 0x22, // R n/a 0 – 2000
    Temperature_H = 0x23, // R n/a Movement channel temperature reference (a previous value of temperature channel)
    Temperature_L = 0x24, // R n/a 0 – 2000
    LtaHaltTimer_H = 0x25, // R n/a Countdown timer to give active feedback on the time-out. Movement events will reset this timer
    LtaHaltTimer_L = 0x26, // R n/a (0 – 255) × 100ms | Timer range: 0 – 90min
    FilterHaltTimer = 0x27, // R n/a Countdown timer to give active feedback on the fixed 5sec time-out when in filter halt mode (before entering Proximity detect)
    TimerReadInput = 0x28,  // R n/a Countdown timer to signal when a read operation is done on IO2
    TimerRedoAti = 0x29,    // R n/a
}

impl Register {
    pub fn is_writable(&self) -> bool {
        matches!(
            self,
            Register::Reserved
                | Register::Commands
                | Register::OtpBank1
                | Register::OtpBank2
                | Register::OtpBank3
                | Register::QuickRelease
                | Register::Movement
                | Register::TouchThreshold
                | Register::ProximityThreshold
                | Register::TempInterferenceThreshold
                | Register::CH0_Multipliers
                | Register::CH0_Compensation
                | Register::CH1_Multipliers
                | Register::CH1_Compensation
        )
    }

    pub(crate) fn next<T>(self) -> Result<Self, Error<T>> {
        Self::from_u8(self as u8 + 1)
    }

    pub(crate) fn from_u8<T>(reg_nr: u8) -> Result<Self, Error<T>> {
        Self::try_from_primitive(reg_nr).map_err(|_| Error::InvalidRegister)
    }
}

impl From<Register> for u8 {
    fn from(value: Register) -> Self {
        value as u8
    }
}

pub const PRODUCT_NUMBER: u8 = 0x40;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive)]
pub enum SoftwareVersion {
    IQS231A = 0x06,
    IQS231B = 0x07,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MainEvents: u8 {
        const SENSING_DISABLED = 0x20;
        const WARM_BOOT = 0x10;
        const COLD_BOOT = 0x08;
        const RELEASE = 0x04;
        const TOUCH = 0x02;
        const PROX = 0x01;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DebugEvents: u8 {
        const RESERVED1 = 0x80;
        const ATI_ERROR = 0x40;
        const CH0_ATI = 0x20;
        const RESERVED2 = 0x10;

        const QUICK_RELEASE = 0x08;
        const EXIT_MOV_DETECT = 0x04;
        const ENTER_MOV_DETECT = 0x02;
        const MOVEMENT = 0x01;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SystemFlags: u8 {
        const I2C  = 0x80;
        const TEMP = 0x40;
        const CH0_ACTIVE = 0x20;
        const CURRENT_CH = 0x10;
        const NO_SYNC = 0x08;
        const CH0_LTA_HALTED = 0x04;
        const ATI_MODE = 0x02;
        const ZOOM_MODE = 0x01;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UiFlags: u8 {
        const TEMP_CHANNEL_ATI = 0x80;
        const TEMPERATURE_RESEED = 0x40;
        const _RESERVED2 = 0x20;
        const UI_AUTO_ATI_OFF = 0x10;
        const UI_SENSING_DISABLD = 0x08;
        const QUICK_RELEASE = 0x04;
        const _RESERVED1 = 0x02;
        const OUTPUT_ACTIVE = 0x01;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EventFlags: u8 {
        const CH1_ATI_ERROR = 0x80;
        const _RESERVED2 = 0x40;
        const _RESERVED1 = 0x20;
        const CH1_MOVEMENT = 0x10;
        const CH0_ATI_ERROR = 0x08;
        const CH0_UNDEBOUNCED = 0x04;
        const CH0_TOUCH = 0x02;
        const CH0_PROX = 0x01;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Commands: u8 {
        const ATI_CH0 = 0x80;
        const DISABLE_SENSING = 0x40;
        const ENABLE_SENSING = 0x20;
        const TOGGLE_AC_FILTER = 0x10;
        const RESERVED1 = 0x08;
        const TOGGLE_ULP_MODE = 0x04;
        const RESERVED2 = 0x02;
        const STANDALONE = 0x01; // A.k.a. WARM_BOOT
    }
}

pub enum QuickReleaseThreshold {}
