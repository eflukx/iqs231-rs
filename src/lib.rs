#![no_std]

pub mod device;
pub mod registers;

pub use device::Iqs231;

#[derive(Debug)]
pub enum Error<IE> {
    /// All I²C bus and comms errors are wrapped here
    IoError(IE),

    /// Software version is other than known at the time of writing (0x06 or 0x07)
    UnknownSoftwareVersion(u8),
    /// Product number is always expected to be 0x40 (defined as `registers::PRODUCT_NUMBER`)
    IncorrectProductNumber(u8),

    /// Requested register does not exist
    InvalidRegister,

    /// Register is not writable
    RegisterNotWritable,

    /// Use `into_standalone()` to issue this the `STANDALONE` command,
    ShutdownCommandNotAllowed,

    /// touch threshold should be 4..=1024
    TouchThresholdOutOfRange,
}

// Allow for quenching the error in a Result<_,()>
impl<E> From<Error<E>> for ()
{
    fn from(value: Error<E>) -> Self {
        ()
    }
}
