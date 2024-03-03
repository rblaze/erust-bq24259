#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

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
        self.update(5, |v| v & 0b11001111)
    }

    pub fn status(&mut self) -> Result<u8, I2C::Error> {
        self.read(8)
    }

    pub fn faults(&mut self) -> Result<u8, I2C::Error> {
        self.read(9)
    }

    // Reads register and returns its value
    fn read(&mut self, register: u8) -> Result<u8, I2C::Error> {
        let mut value = [0u8; 1];
        self.i2c.write_read(ADDR, &[register], &mut value)?;
        Ok(value[0])
    }

    // Reads register, updates it and writes back.
    fn update(&mut self, register: u8, update: fn(u8) -> u8) -> Result<(), I2C::Error> {
        let value = self.read(register)?;
        self.i2c.write(ADDR, &[register, update(value)])
    }
}
