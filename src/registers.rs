// For the i2c register reference, see datasheet: https://www.azoteq.com/images/stories/pdf/iqs231a_datasheet.pdf (pg. 14 and pg. 30 onwards)
use core::ops::Deref;
use modular_bitfield::prelude::*;
use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};

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

    Reserved = 0x03, // R/W n/a RESERVED
    Commands = 0x04, // R/W 0x00 ATI_CH0 DISABLE
    OtpBank1 = 0x05, // R/W 0x00 Standalone / I2C address Proximity threshold
    OtpBank2 = 0x06, // R/W 0x00 Increase
    OtpBank3 = 0x07, // R/W 0x00 Charge transfer frequency Temperature

    QuickRelease = 0x08, // R/W 0x00 Quick release threshold LUT Quick release beta
    Movement = 0x09,     // R/W 0x34
    TouchThreshold = 0x0A, // R/W 0x07
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
    pub struct Commands: u8 {
        const ATI_CH0 = 0x80;
        const DISABLE_SENSING = 0x40;
        const ENABLE_SENSING = 0x20;
        const TOGGLE_AC_FILTER = 0x10;
        const _RESERVED1 = 0x08;
        const TOGGLE_ULP_MODE = 0x04;
        const _RESERVED2 = 0x02;
        const STANDALONE = 0x01; // A.k.a. WARM_BOOT
    }
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
        const _RESERVED1 = 0x80;
        const ATI_ERROR = 0x40;
        const CH0_ATI = 0x20;
        const _RESERVED2 = 0x10;
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

#[bitfield(bits = 8)]
pub struct OtpBank1 {
    pub touch_thresh: B2,
    pub ac_filter: B2,
    pub prox_thresh: ProximityThreshold,
    pub i2c_addr: B2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BitfieldSpecifier, IntoPrimitive)]
#[repr(u8)]
#[bits = 2]
pub enum ProximityThreshold {
    Counts4,  //0x0
    Counts6,  //0x1
    Counts8,  //0x2
    Counts10, //0x3
}

impl From<u8> for ProximityThreshold {
    fn from(value: u8) -> Self {
        match value & 0x03 {
            0b00 => Self::Counts4,
            0b01 => Self::Counts6,
            0b10 => Self::Counts8,
            0b11 => Self::Counts10,
            _ => unreachable!(),
        }
    }
}

#[bitfield(bits = 8)]
pub struct OtpBank2 {
    pub ui_select: UiSelect,
    pub quick_release: B1,
    pub failsafe_pulses_on_io1: bool,
    pub base_value: BaseValue,
    pub target: B1,
    pub increase_debounce: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BitfieldSpecifier)]
#[bits = 2]
pub enum UiSelect {
    ProxNoMov,             //0x0
    ProxWithMov,           //0x1
    ProxWithMovTouchNoMov, //0x2
    ProxWithMovTouchOnIo2, //0x3
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BitfieldSpecifier)]
#[bits = 2]
pub enum BaseValue {
    Counts100, //0x0
    Counts75,  //0x1
    Counts150, //0x2
    Counts200, //0x3
}

#[bitfield(bits = 8)]
pub struct OtpBank3 {
    pub sample_rate: SampleRate,
    pub ati_events_on_io1: B1,

    pub io2_function: Io2Function,
    pub temp_n_interference_compensation: bool,
    pub charge_transfer_freq: ChargeTransferFrequency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BitfieldSpecifier)]
#[bits = 2]
pub enum SampleRate {
    _30Hz,  // 0x0 (57ms)
    _100Hz, // 0x1 (34ms)
    _8Hz,   // 0x2 (154ms)
    _4Hz,   // 0x3 (280ms)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BitfieldSpecifier)]
#[bits = 2]
pub enum Io2Function {
    Sensitivity, // 0x00 – Sensitivity input    (proximity threshold adjust)
    Synchronize, // 0x01 – Synchronize input
    Movement,    // 0x10 – Movement output
    Ignore,      // 0x11 – Ignore input, no output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BitfieldSpecifier)]
#[bits = 2]
pub enum ChargeTransferFrequency {
    _500kHz, // 0x00 – 500kHz
    _125kHz, // 0x01 – 125 kHz
    _64kHz,  // 0x10 – 64 kHz
    _16kHz,  // 0x11 – 16.5kHz
}

#[bitfield(bits = 8)]
pub struct QuickRelease {
    pub base: B4,
    pub threshold: QuickReleaseThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BitfieldSpecifier)]
#[bits = 4]
pub enum QuickReleaseThreshold {
    Qrt100,  //0x0
    Qrt150,  //0x1
    Qrt50,   //0x2
    Qrt250,  //0x3
    Qrt10,   //0x4
    Qrt20,   //0x5
    Qrt25,   //0x6
    Qrt30,   //0x7
    Qrt75,   //0x8
    Qrt200,  //0x9
    Qrt300,  //0xA
    Qrt400,  //0xB
    Qrt500,  //0xC
    Qrt750,  //0xD
    Qrt850,  //0xE
    Qrt1000, //0xF
}

impl QuickReleaseThreshold {
    pub fn counts(&self) -> u16 {
        match self {
            Self::Qrt100 => 100,
            Self::Qrt150 => 150,
            Self::Qrt50 => 50,
            Self::Qrt250 => 250,
            Self::Qrt10 => 10,
            Self::Qrt20 => 20,
            Self::Qrt25 => 25,
            Self::Qrt30 => 30,
            Self::Qrt75 => 75,
            Self::Qrt200 => 200,
            Self::Qrt300 => 300,
            Self::Qrt400 => 400,
            Self::Qrt500 => 500,
            Self::Qrt750 => 750,
            Self::Qrt850 => 850,
            Self::Qrt1000 => 1000,
        }
    }
}

#[bitfield(bits = 8)]
pub struct ChannelMultiplier {
    pub compensation_multiplier: B4,
    pub sensitivity_multiplier: B2,
    reserved: B2,
}

#[test]
fn otpbank3_bitfield_does_its_thing() {
    let otp = OtpBank3::new()
        .with_charge_transfer_freq(ChargeTransferFrequency::_64kHz)
        .with_sample_rate(SampleRate::_8Hz);

    assert_eq!(otp.into_bytes()[0], 0x82);

    let otp = OtpBank3::new()
        .with_charge_transfer_freq(ChargeTransferFrequency::_125kHz)
        .with_sample_rate(SampleRate::_30Hz)
        .with_ati_events_on_io1(1)
        .with_temp_n_interference_compensation(true);

    assert_eq!(otp.into_bytes()[0], 0x64)
}

#[test]
fn quickrelease_bitfield_does_its_thing() {
    let qr = QuickRelease::from_bytes([0xb4]);
    assert_eq!(qr.base(), 4);
    assert_eq!(qr.threshold(), QuickReleaseThreshold::Qrt400);
    assert_eq!(qr.threshold().counts(), 400);

    let qrr = QuickRelease::new()
        .with_base(5)
        .with_threshold(QuickReleaseThreshold::Qrt200);
    assert_eq!(qrr.bytes, [0x95]);

    let qr2 = QuickRelease::from_bytes([0x4a]);
    assert_eq!(qr2.base(), 0xa);
    assert_eq!(qr2.threshold(), QuickReleaseThreshold::Qrt10);
    assert_eq!(qr2.threshold().counts(), 10);
}
