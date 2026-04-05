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

    pub fn set_watchdog_timeout(&mut self, timeout: registers::Watchdog) -> Result<(), I2C::Error> {
        self.update(registers::REG_TERM_TIMER_CONTROL, |v| {
            let mut reg = registers::ChargeTerminationTimerControl::from(v);
            reg.set_watchdog(timeout);
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
    use embedded_hal::i2c::{Error, ErrorType, Operation};

    #[derive(Debug, PartialEq, Eq)]
    enum DummyError {
        I2c,
    }

    impl Error for DummyError {
        fn kind(&self) -> embedded_hal::i2c::ErrorKind {
            embedded_hal::i2c::ErrorKind::Other
        }
    }

    struct MockBus {
        pub write_data: Vec<Vec<u8>>,
        pub read_data: Vec<u8>,
    }

    impl ErrorType for MockBus {
        type Error = DummyError;
    }

    impl I2c for MockBus {
        fn transaction(
            &mut self,
            _address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            for op in operations {
                match op {
                    Operation::Write(data) => {
                        self.write_data.push(data.to_vec());
                    }
                    Operation::Read(data) => {
                        if self.read_data.is_empty() {
                            return Err(DummyError::I2c);
                        }
                        data[0] = self.read_data.remove(0);
                    }
                }
            }
            Ok(())
        }
    }

    #[test]
    fn ichg_read() {
        // Default value means 2048
        assert_eq!(
            BQ24259::<MockBus>::ichg_to_milliamps(u5::new(0b11000)),
            2048
        );
        assert_eq!(
            BQ24259::<MockBus>::ichg_to_milliamps(u5::new(0b01100)),
            1280
        );
        // Zero value means 512
        assert_eq!(BQ24259::<MockBus>::ichg_to_milliamps(u5::new(0)), 512);
    }

    #[test]
    fn ichg_write() {
        // Default value means 2048
        assert_eq!(BQ24259::<MockBus>::milliamps_to_ichg(2048).value(), 0b11000);
        assert_eq!(BQ24259::<MockBus>::milliamps_to_ichg(896).value(), 0b00110);
        // Zero value means 512
        assert_eq!(BQ24259::<MockBus>::milliamps_to_ichg(512).value(), 0);
        // Value clamps to 2048 from the top
        assert_eq!(
            BQ24259::<MockBus>::milliamps_to_ichg(10000).value(),
            0b11000
        );
        // Value clamps to 512 from the bottom
        assert_eq!(BQ24259::<MockBus>::milliamps_to_ichg(5).value(), 0);
    }

    #[test]
    fn set_watchdog_timeout() {
        let mut bus = MockBus {
            write_data: Vec::new(),
            read_data: vec![0x02],
        };
        let mut bq = BQ24259::new(&mut bus);

        bq.set_watchdog_timeout(registers::Watchdog::Sec160)
            .unwrap();

        assert_eq!(bus.write_data.len(), 2);
        assert_eq!(bus.write_data[0], vec![registers::REG_TERM_TIMER_CONTROL]);
        assert_eq!(
            bus.write_data[1],
            vec![registers::REG_TERM_TIMER_CONTROL, 0x32]
        );
    }
}
