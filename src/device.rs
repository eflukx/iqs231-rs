use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use num_enum::TryFromPrimitive;

use crate::{
    registers::{
        self, ChannelMultiplier, Commands, DebugEvents, EventFlags, MainEvents, OtpBank1, OtpBank2,
        OtpBank3, ProximityThreshold, QuickRelease, RegValue, Register, SoftwareVersion,
        SystemFlags, UiFlags,
    },
    Error,
};

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive)]
pub enum I2cAddress {
    #[default]
    /// Default I2C address
    Default = 0x44,

    /// not used in normal operation
    Test = 0x45,
    
    Alt1 = 0x46,
    Alt2 = 0x47,
}

pub struct Iqs231<I> {
    bus: I,
    address: I2cAddress,
}

impl<I> Iqs231<I> {
    pub fn new(bus: I) -> Self {
        Self {
            bus,
            address: I2cAddress::default(),
        }
    }

    pub fn with_address(self, address: I2cAddress) -> Self {
        Self { address, ..self }
    }

    pub fn destroy(self) -> I {
        self.bus
    }
}

impl<E, I> Iqs231<I>
where
    I: Read<Error = E>,
{
    pub fn read_main_events(&mut self) -> Result<MainEvents, Error<E>> {
        let mut rd_buffer = [0u8; 1];
        self.bus
            .read(self.address as u8, &mut rd_buffer)
            .map_err(|e| Error::IoError(e))?;

        Ok(MainEvents::from_bits_retain(rd_buffer[0]))
    }
}

impl<E, I> Iqs231<I>
where
    I: Write<Error = E> + WriteRead<Error = E>,
{
    pub fn get_prod_nr(&mut self) -> Result<u8, Error<E>> {
        let prod_nr = self.read_reg(Register::ProductNumber)?.value;
        if prod_nr == registers::PRODUCT_NUMBER {
            Ok(prod_nr)
        } else {
            Err(Error::IncorrectProductNumber(prod_nr))
        }
    }

    pub fn get_software_version(&mut self) -> Result<SoftwareVersion, Error<E>> {
        let ver = self.read_reg(Register::ProductNumber)?.value;
        SoftwareVersion::try_from_primitive(ver).map_err(|_| Error::UnknownSoftwareVersion(ver))
    }

    pub fn set_otp_bank1(&mut self, value: OtpBank1) -> Result<(), Error<E>> {
        self.write_reg(Register::OtpBank1, value.into_bytes()[0])
    }

    pub fn get_otp_bank1(&mut self) -> Result<RegValue<OtpBank1>, Error<E>> {
        let regval = self
            .read_reg(Register::OtpBank1)?
            .map(|v| OtpBank1::from_bytes([v]));
        Ok(regval)
    }

    pub fn set_otp_bank2(&mut self, value: OtpBank2) -> Result<(), Error<E>> {
        self.write_reg(Register::OtpBank2, value.into_bytes()[0])
    }

    pub fn get_otp_bank2(&mut self) -> Result<RegValue<OtpBank2>, Error<E>> {
        let regval = self
            .read_reg(Register::OtpBank2)?
            .map(|v| OtpBank2::from_bytes([v]));
        Ok(regval)
    }

    pub fn set_otp_bank3(&mut self, value: OtpBank3) -> Result<(), Error<E>> {
        self.write_reg(Register::OtpBank3, value.into_bytes()[0])
    }

    pub fn get_otp_bank3(&mut self) -> Result<RegValue<OtpBank3>, Error<E>> {
        let regval = self
            .read_reg(Register::OtpBank3)?
            .map(|v| OtpBank3::from_bytes([v]));
        Ok(regval)
    }

    pub fn set_touch_threshold(&mut self, threshold: u16) -> Result<(), Error<E>> {
        if threshold < 4 || threshold > 1024 {
            Err(Error::TouchThresholdOutOfRange)
        } else {
            let value = (threshold - 4) >> 2;
            self.write_reg(Register::TouchThreshold, value as u8)
        }
    }

    pub fn get_touch_threshold(&mut self) -> Result<RegValue<u16>, Error<E>> {
        Ok(self
            .read_reg(Register::TouchThreshold)?
            .map(|v| ((v as u16) << 2) + 4))
    }

    pub fn set_proximity_threshold(
        &mut self,
        threshold: ProximityThreshold,
    ) -> Result<(), Error<E>> {
        self.write_reg(Register::ProximityThreshold, threshold.into())
    }

    pub fn get_proximity_threshold(&mut self) -> Result<RegValue<ProximityThreshold>, Error<E>> {
        self.read_reg_t(Register::ProximityThreshold)
    }

    /// Default 3. Low values are recommended for intended effect.
    /// Use a higher value when using the feature in a noisy environment.
    pub fn set_temp_interference_threshold(&mut self, threshold: u8) -> Result<(), Error<E>> {
        self.write_reg(Register::TempInterferenceThreshold, threshold)
    }

    pub fn set_ch0_multipliers(&mut self, mult: ChannelMultiplier) -> Result<(), Error<E>> {
        self.write_reg(Register::CH0_Multipliers, mult.into_bytes()[0])
    }

    pub fn get_ch0_multipliers(&mut self) -> Result<RegValue<ChannelMultiplier>, Error<E>> {
        let regval = self
            .read_reg(Register::CH0_Multipliers)?
            .map(|val| ChannelMultiplier::from_bytes([val]));

        Ok(regval)
    }

    pub fn set_ch0_compensation(&mut self, comp: u8) -> Result<(), Error<E>> {
        self.write_reg(Register::CH0_Compensation, comp)
    }

    pub fn get_ch0_compensation(&mut self) -> Result<RegValue<u8>, Error<E>> {
        self.read_reg(Register::CH0_Compensation)
    }

    pub fn set_ch1_multipliers(&mut self, mult: ChannelMultiplier) -> Result<(), Error<E>> {
        self.write_reg(Register::CH1_Multipliers, mult.into_bytes()[0])
    }

    pub fn get_ch1_multipliers(&mut self) -> Result<RegValue<ChannelMultiplier>, Error<E>> {
        let regval = self
            .read_reg(Register::CH1_Multipliers)?
            .map(|val| ChannelMultiplier::from_bytes([val]));

        Ok(regval)
    }

    pub fn set_ch1_compensation(&mut self, comp: u8) -> Result<(), Error<E>> {
        self.write_reg(Register::CH1_Compensation, comp)
    }

    pub fn get_ch1_compensation(&mut self) -> Result<RegValue<u8>, Error<E>> {
        self.read_reg(Register::CH1_Compensation)
    }

    pub fn get_debug_events(&mut self) -> Result<DebugEvents, Error<E>> {
        let value = self.read_reg(Register::ProductNumber)?.value;
        Ok(DebugEvents::from_bits_retain(value))
    }

    pub fn get_system_flags(&mut self) -> Result<SystemFlags, Error<E>> {
        let value = self.read_reg(Register::System_Flags)?.value;
        Ok(SystemFlags::from_bits_retain(value))
    }

    pub fn get_ui_flags(&mut self) -> Result<UiFlags, Error<E>> {
        let value = self.read_reg(Register::UI_Flags)?.value;
        Ok(UiFlags::from_bits_retain(value))
    }

    pub fn get_event_flags(&mut self) -> Result<EventFlags, Error<E>> {
        let value = self.read_reg(Register::EventFlags)?.value;
        Ok(EventFlags::from_bits_retain(value))
    }

    pub fn get_otp_bank_1(&mut self) -> Result<RegValue<OtpBank1>, Error<E>> {
        let rv = self.read_reg(Register::OtpBank1)?;
        Ok(rv.map(|v| OtpBank1::from_bytes([v])))
    }

    pub fn get_otp_bank_2(&mut self) -> Result<RegValue<OtpBank2>, Error<E>> {
        let rv = self.read_reg(Register::OtpBank2)?;
        Ok(rv.map(|v| OtpBank2::from_bytes([v])))
    }

    pub fn get_otp_bank_3(&mut self) -> Result<RegValue<OtpBank3>, Error<E>> {
        let rv = self.read_reg(Register::OtpBank3)?;
        Ok(rv.map(|v| OtpBank3::from_bytes([v])))
    }

    pub fn set_quick_release(&mut self, quick_rel: QuickRelease) -> Result<(), Error<E>> {
        self.write_reg(Register::QuickRelease, quick_rel.into_bytes()[0])
    }

    pub fn get_quick_release(&mut self) -> Result<RegValue<QuickRelease>, Error<E>> {
        let rv = self.read_reg(Register::QuickRelease)?;
        Ok(rv.map(|v| QuickRelease::from_bytes([v])))
    }

    /// Proximity channel: Filtered count value
    /// (0-2000)
    pub fn get_prox_filtered_count(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH0_ACF_H)
    }

    /// Proximity channel: Reference count value (Long term average)
    /// (0-2000)
    pub fn get_prox_reference_count(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH0_LTA_H)
    }

    /// Proximity channel: Quick release detect reference value
    /// (0-2000)
    pub fn get_prox_quick_release_detect_reference(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH0_QRD_H)
    }

    /// Movement channel: Filtered count value
    /// (0-2000)
    pub fn get_move_filtered_count(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH1_ACF_H)
    }

    /// Movement channel: Upper reference count value
    /// (0-2000)
    pub fn get_move_upper_reference_count(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH1_UMOV_H)
    }

    /// Movement channel: Lower reference count value
    /// (0-2000)
    pub fn get_move_lower_reference_count(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH1_LMOV_L)
    }

    /// Temperature channel: Unfiltered count value (if temperature feature enabled)
    /// (0-2000)
    pub fn get_move_unfiltered_count(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH1_RAW_H)
    }

    /// Movement channel temperature reference (a previous value of temperature channel)
    /// (0-2000)
    pub fn get_temp_reference(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::Temperature_H)
    }
    /// Countdown timer to give active feedback on the time-out. Movement events will reset this timer
    /// (0 – 255) × 100ms | Timer range: 0 – 90min
    pub fn get_lta_halt_timer(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::LtaHaltTimer_H)
    }

    // FILTER_HALT_TIMER R n/a Countdown timer to give active feedback on the fixed 5sec time-out when in filter halt mode (before entering Proximity detect)
    // 0 – 50 x 100ms | Timer range: 0 – 5 seconds
    pub fn get_filter_halt_timer(&mut self) -> Result<RegValue<u8>, Error<E>> {
        self.read_reg(Register::FilterHaltTimer)
    }

    // TIMER_READ_INPUT R n/a Countdown timer to signal when a read operation is done on IO2
    // (0 – 10) x 100ms | Timer range: 0 – 1 seconds
    pub fn get_timer_read_input(&mut self) -> Result<RegValue<u8>, Error<E>> {
        self.read_reg(Register::TimerReadInput)
    }

    // TIMER_REDO_ATI R n/a
    // Countdown timer to give active feedback on the time until re-calibration is attempted after ATI-error
    // (0 – 255) × 100ms | Timer range: 0 – 25s
    pub fn get_timer_redo_ati(&mut self) -> Result<RegValue<u8>, Error<E>> {
        self.read_reg(Register::TimerRedoAti)
    }

    /// Use this function (taking ownership of device) to put device in standalone mode
    /// returns the the I²C bus
    pub fn into_standalone(mut self) -> Result<I, Error<E>> {
        self.write_reg(Register::Commands, Commands::STANDALONE.bits())?;
        Ok(self.destroy())
    }

    /// Send command(s)
    /// Sending command "STANDALONE" ("WARM_BOOT") NOT allowed, as this disables i2c on the device.
    /// use `into_standalone()` to issue this the `STANDALONE` command, set the device in standalone modde and render the I²C bus
    pub fn send_commands(&mut self, commands: Commands) -> Result<(), Error<E>> {
        if commands.contains(Commands::STANDALONE) {
            Err(Error::ShutdownCommandNotAllowed)
        } else {
            self.write_reg(Register::Commands, commands.bits())
        }
    }

    fn read_reg16(&mut self, register: impl Into<Register>) -> Result<RegValue<u16>, Error<E>> {
        let reg: Register = register.into();
        let hi = self.read_reg(reg)?;
        let lo = self.read_reg(reg.next()?)?;

        Ok(RegValue {
            main_events: hi.main_events | lo.main_events,
            value: (hi.value as u16) << 8 | lo.value as u16,
        })
    }

    /// Read register converted into the specified type (using `From<u8>`)
    fn read_reg_t<T: From<u8>>(
        &mut self,
        register: impl Into<Register>,
    ) -> Result<RegValue<T>, Error<E>> {
        self.read_reg(register).map(|rv| rv.map(T::from))
    }

    fn read_reg(&mut self, register: impl Into<Register>) -> Result<RegValue<u8>, Error<E>> {
        let reg: Register = register.into();
        let mut rd_buffer = [0u8; 2];

        self.bus
            .write_read(self.address as u8, &[reg as u8], &mut rd_buffer)
            .map_err(|e| Error::IoError(e))?;

        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Read reg [{}] -> {:#x}",
            defmt::Debug2Format(&reg),
            rd_buffer
        );

        Ok(rd_buffer.into())
    }

    fn write_reg(&mut self, register: impl Into<Register>, value: u8) -> Result<(), Error<E>> {
        let reg: Register = register.into();

        #[cfg(feature = "defmt")]
        defmt::trace!("Write reg [{}] <- {:#x}", defmt::Debug2Format(&reg), value);

        if reg.is_writable() {
            self.bus
                .write(self.address as u8, &[reg as u8, value])
                .map_err(|e| Error::IoError(e))
        } else {
            Err(Error::RegisterNotWritable.into())
        }
    }
}
