//! I2C control and status registers

use bitfield::bitfield;

/// Input Source Control Register REG00
pub const REG_INPUT_SOURCE_CONTROL: u8 = 0x00;

/// Power-On Configuration Register REG01
pub const REG_POWER_ON_CONFIGURATION: u8 = 0x01;

/// Charge Current Control Register REG02
pub const REG_CHARGE_CURRENT_CONTROL: u8 = 0x02;

/// Pre-Charge/Termination Current Control Register REG03
pub const REG_PCT_CURRENT_CONTROL: u8 = 0x03;

/// Charge Voltage Control Register REG04
pub const REG_CHARGE_VOLTAGE_CONTROL: u8 = 0x04;

/// Charge Termination/Timer Control Register REG05
pub const REG_TERM_TIMER_CONTROL: u8 = 0x05;

/// Boost Voltage/Thermal Regulation Control Register REG06
pub const REG_BOOST_TEMP_CONTROL: u8 = 0x06;

/// Misc Operation Control Register REG07
pub const REG_MISC_OPERATION_CONTROL: u8 = 0x07;

/// System Status Register REG08
pub const REG_SYSTEM_STATUS: u8 = 0x08;

bitfield! {
    /// System Status Register description
    pub struct SystemStatus(u8);
    impl Debug;
    pub vbus_stat, _: 7,6;
    pub chrg_stat, _: 5,4;
    pub dpm_stat, _: 3;
    pub pg_stat, _: 2;
    pub therm_stat, _: 1;
    pub vsys_stat, _: 0;
}

/// New Fault Register REG09
pub const REG_NEW_FAULT: u8 = 0x09;

bitfield! {
    /// System Status Register description
    pub struct NewFault(u8);
    impl Debug;
    pub watchdog_fault, _: 7;
    pub otg_fault, _: 6;
    pub chrg_fault, _: 5, 4;
    pub bat_fault, _: 3;
    pub ntc_fault_cold, _: 1;
    pub ntc_fault_hot, _: 0;
}

/// Vender / Part / Revision Status Register REG0A
pub const REG_VENDOR: u8 = 0x0A;
