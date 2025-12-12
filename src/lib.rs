#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

pub mod registers;

use bilge::arbitrary_int::u5;
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x6b;

pub struct BQ24259<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> BQ24259<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn disable_watchdog(&mut self) -> Result<(), I2C::Error> {
        self.update(registers::REG_TERM_TIMER_CONTROL, |v| {
            let mut reg = registers::ChargeTerminationTimerControl::from(v);
            reg.set_en_timer(false);
            reg.into()
        })
    }

    pub fn reset_watchdog(&mut self) -> Result<(), I2C::Error> {
        self.update(registers::REG_POWER_ON_CONFIGURATION, |v| {
            let mut reg = registers::PowerOnConfiguration::from(v);
            reg.set_watchdog_reset(true);
            reg.into()
        })
    }

    pub fn status(&mut self) -> Result<registers::SystemStatus, I2C::Error> {
        let value = self.read(registers::REG_SYSTEM_STATUS)?;
        Ok(registers::SystemStatus::from(value))
    }

    pub fn new_fault(&mut self) -> Result<registers::NewFault, I2C::Error> {
        let value = self.read(registers::REG_NEW_FAULT)?;
        Ok(registers::NewFault::from(value))
    }

    fn ichg_to_milliamps(ichg: u5) -> u16 {
        u16::from(ichg) * 64 + 512
    }

    fn milliamps_to_ichg(milliamps: u16) -> u5 {
        let value = (milliamps.clamp(512, 2048) - 512) / 64;
        u5::new(value as u8)
    }

    /// Gets charge current limit ICHG
    pub fn charge_current_limit(&mut self) -> Result<u16, I2C::Error> {
        let value = self.read(registers::REG_CHARGE_CURRENT_CONTROL)?;
        let reg = registers::ChargeCurrentControl::from(value);
        Ok(Self::ichg_to_milliamps(reg.ichg()))
    }

    /// Sets charge current limit ICHG, clamped to [512..2048] mA
    pub fn set_charge_current_limit(&mut self, milliamps: u16) -> Result<(), I2C::Error> {
        let ichg = Self::milliamps_to_ichg(milliamps);
        self.update(registers::REG_CHARGE_CURRENT_CONTROL, |v| {
            let mut reg = registers::ChargeCurrentControl::from(v);
            reg.set_ichg(ichg);
            reg.into()
        })
    }

    // Reads register and returns its value.
    pub fn read(&mut self, register: u8) -> Result<u8, I2C::Error> {
        let mut value = [0u8; 1];
        self.i2c.write_read(ADDR, &[register], &mut value)?;
        Ok(value[0])
    }

    // Reads register, updates it and writes back.
    pub fn update(&mut self, register: u8, update: impl Fn(u8) -> u8) -> Result<(), I2C::Error> {
        let value = self.read(register)?;
        self.i2c.write(ADDR, &[register, update(value)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal::i2c::{Error, ErrorType};

    #[derive(Debug)]
    enum DummyError {}

    impl Error for DummyError {
        fn kind(&self) -> embedded_hal::i2c::ErrorKind {
            unimplemented!()
        }
    }

    struct DummyBus {}

    impl ErrorType for DummyBus {
        type Error = DummyError;
    }

    impl I2c for DummyBus {
        fn transaction(
            &mut self,
            _address: u8,
            _operations: &mut [embedded_hal::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn ichg_read() {
        // Default value means 2048
        assert_eq!(
            BQ24259::<DummyBus>::ichg_to_milliamps(u5::new(0b11000)),
            2048
        );
        assert_eq!(
            BQ24259::<DummyBus>::ichg_to_milliamps(u5::new(0b01100)),
            1280
        );
        // Zero value means 512
        assert_eq!(BQ24259::<DummyBus>::ichg_to_milliamps(u5::new(0)), 512);
    }

    #[test]
    fn ichg_write() {
        // Default value means 2048
        assert_eq!(
            BQ24259::<DummyBus>::milliamps_to_ichg(2048).value(),
            0b11000
        );
        assert_eq!(BQ24259::<DummyBus>::milliamps_to_ichg(896).value(), 0b00110);
        // Zero value means 512
        assert_eq!(BQ24259::<DummyBus>::milliamps_to_ichg(512).value(), 0);
        // Value clamps to 2048 from the top
        assert_eq!(
            BQ24259::<DummyBus>::milliamps_to_ichg(10000).value(),
            0b11000
        );
        // Value clamps to 512 from the bottom
        assert_eq!(BQ24259::<DummyBus>::milliamps_to_ichg(5).value(), 0);
    }
}
