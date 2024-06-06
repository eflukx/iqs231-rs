use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use num_enum::TryFromPrimitive;

use crate::{
    registers::{
        self, Commands, DebugEvents, EventFlags, MainEvents, RegValue, Register, SoftwareVersion,
        SystemFlags, UiFlags,
    },
    Error,
};

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive)]
pub enum I2cAddress {
    #[default]
    Default = 0x44,
    Test = 0x45, // not used in normal operation
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
            Err(Error::IncorrectProductNumber(prod_nr).into())
        }
    }

    pub fn get_software_version(&mut self) -> Result<SoftwareVersion, Error<E>> {
        let ver = self.read_reg(Register::ProductNumber)?.value;
        SoftwareVersion::try_from_primitive(ver)
            .map_err(|_| Error::UnknownSoftwareVersion(ver).into())
    }

    /// Use this function to put device in standalone mode, rendering the I²C bus
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

    pub fn get_debug_events(&mut self) -> Result<DebugEvents, Error<E>> {
        let value = self.read_reg(Register::ProductNumber)?.value;
        Ok(DebugEvents::from_bits_retain(value))
    }

    pub fn get_system_flags(&mut self) -> Result<SystemFlags, Error<E>> {
        let value = self.read_reg(Register::System_Flags)?.value;
        Ok(SystemFlags::from_bits_retain(value))
    }

    pub fn get_event_flags(&mut self) -> Result<EventFlags, Error<E>> {
        let value = self.read_reg(Register::EventFlags)?.value;
        Ok(EventFlags::from_bits_retain(value))
    }

    pub fn get_ui_flags(&mut self) -> Result<UiFlags, Error<E>> {
        let value = self.read_reg(Register::UI_Flags)?.value;
        Ok(UiFlags::from_bits_retain(value))
    }

    /// Proximity channel: Filtered count value
    /// (0-2000)
    pub fn get_prox_filtered_count_value(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH0_ACF_H)
    }

    /// Proximity channel: Reference count value (Long term average)
    /// (0-2000)
    pub fn get_prox_reference_count_value(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH0_LTA_H)
    }

    /// Proximity channel: Quick release detect reference value
    /// (0-2000)
    pub fn get_prox_quick_relese_detect_reference_value(
        &mut self,
    ) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH0_QRD_H)
    }

    /// Movement channel: Filtered count value
    /// (0-2000)
    pub fn get_move_filtered_count_value(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH1_ACF_H)
    }

    /// Movement channel: Upper reference count value
    /// (0-2000)
    pub fn get_move_upper_reference_count_value(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH1_UMOV_H)
    }

    /// Movement channel: Lower reference count value
    /// (0-2000)
    pub fn get_move_lower_reference_count_value(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH1_LMOV_L)
    }

    /// Temperature channel: Unfiltered count value (if temperature feature enabled)
    /// (0-2000)
    pub fn get_temp_unfiltered_count_value(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::CH1_RAW_H)
    }

    /// Movement channel temperature reference (a previous value of temperature channel)
    /// (0-2000)
    pub fn get_move_temp_reference(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::Temperature_H)
    }
    /// Countdown timer to give active feedback on the time-out. Movement events will reset this timer
    /// (0 – 255) × 100ms | Timer range: 0 – 90min
    pub fn get_lta_halt_timer(&mut self) -> Result<RegValue<u16>, Error<E>> {
        self.read_reg16(Register::LtaHaltTimer_H)
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

    fn read_reg(&mut self, register: impl Into<Register>) -> Result<RegValue<u8>, Error<E>> {
        let mut rd_buffer = [0u8; 2];
        self.bus
            .write_read(self.address as u8, &[register.into() as u8], &mut rd_buffer)
            .map_err(|e| Error::IoError(e))?;

        Ok(rd_buffer.into())
    }

    fn write_reg(&mut self, register: impl Into<Register>, value: u8) -> Result<(), Error<E>> {
        let reg: Register = register.into();

        // defmt::debug!("Write reg [{}] -> {:#x}", defmt::Debug2Format(&reg), value);

        if reg.is_writable() {
            self.bus
                .write(self.address as u8, &[reg as u8, value])
                .map_err(|e| Error::IoError(e))
        } else {
            Err(Error::RegisterNotWritable.into())
        }
    }
}
