#![deny(unsafe_code)]

//! I2C control and status registers

use bilge::prelude::*;

/// Input Source Control Register REG00
pub const REG_INPUT_SOURCE_CONTROL: u8 = 0x00;

/// Power-On Configuration Register REG01
pub const REG_POWER_ON_CONFIGURATION: u8 = 0x01;

#[bitsize(1)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoostLim {
    Amp1_0 = 0,
    Amp1_5 = 1,
}

/// Power-On Configuration Register
#[bitsize(8)]
#[derive(FromBits)]
pub struct PowerOnConfiguration {
    pub boost_lim: BoostLim,
    pub sys_min: u3,
    pub en_charge: bool,
    pub en_otg: bool,
    pub watchdog_reset: bool,
    pub register_reset: bool,
}

/// Charge Current Control Register REG02
pub const REG_CHARGE_CURRENT_CONTROL: u8 = 0x02;

#[bitsize(8)]
#[derive(FromBits)]
pub struct ChargeCurrentControl {
    pub force_20pct: bool,
    pub bcold: bool,
    pub ichg: u5,
    pub reserved: u1,
}

/// Pre-Charge/Termination Current Control Register REG03
pub const REG_PCT_CURRENT_CONTROL: u8 = 0x03;

/// Charge Voltage Control Register REG04
pub const REG_CHARGE_VOLTAGE_CONTROL: u8 = 0x04;

/// Charge Termination/Timer Control Register REG05
pub const REG_TERM_TIMER_CONTROL: u8 = 0x05;

#[bitsize(2)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChgTimer {
    FiveHours = 0,
    EightHours = 1,
    TwelveHours = 2,
    TwentyHours = 3,
}

#[bitsize(2)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Watchdog {
    Disabled = 0,
    Sec40 = 1,
    Sec80 = 2,
    Sec160 = 3,
}

/// Charge Termination/Timer Control Register
#[bitsize(8)]
#[derive(FromBits, DebugBits)]
pub struct ChargeTerminationTimerControl {
    reserved: u1,
    pub charge_timer: ChgTimer,
    pub en_timer: bool,
    pub watchdog: Watchdog,
    reserved: u1,
    pub en_term: bool,
}

/// Boost Voltage/Thermal Regulation Control Register REG06
pub const REG_BOOST_TEMP_CONTROL: u8 = 0x06;

/// Misc Operation Control Register REG07
pub const REG_MISC_OPERATION_CONTROL: u8 = 0x07;

/// System Status Register REG08
pub const REG_SYSTEM_STATUS: u8 = 0x08;

#[bitsize(2)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Vbus {
    Unknown = 0,
    UsbHost = 1,
    AdapterPort = 2,
    Otg = 3,
}

#[bitsize(2)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Chrg {
    NotCharging = 0,
    PreCharge = 1,
    FastCharging = 2,
    ChargeTermination = 3,
}

/// System Status Register
#[bitsize(8)]
#[derive(FromBits, DebugBits)]
pub struct SystemStatus {
    pub vsys: bool,
    pub therm: bool,
    pub pg: bool,
    pub dpm: bool,
    pub chrg: Chrg,
    pub vbus: Vbus,
}

/// New Fault Register REG09
pub const REG_NEW_FAULT: u8 = 0x09;

#[bitsize(2)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChrgFault {
    Normal = 0,
    /// OVP or bad source
    InputFault = 1,
    ThermalShutdown = 2,
    ChargeTimerExpiration = 3,
}

/// New Fault Register
#[bitsize(8)]
#[derive(FromBits, DebugBits)]
pub struct NewFault {
    pub ntc_hot: bool,
    pub ntc_cold: bool,
    reserved: u1,
    pub bat_fault: bool,
    pub chrg_fault: ChrgFault,
    pub otg_fault: bool,
    pub watchdog_expired: bool,
}

/// Vender / Part / Revision Status Register REG0A
pub const REG_VENDOR: u8 = 0x0A;

/// Expected value of REG_VENDOR
pub const EXPECTED_VENDOR_VALUE: u8 = 0b00100000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_on_configuration() {
        let reg = PowerOnConfiguration::from(0b00011011);
        assert_eq!(reg.register_reset(), false);
        assert_eq!(reg.watchdog_reset(), false);
        assert_eq!(reg.en_otg(), false);
        assert_eq!(reg.en_charge(), true);
        assert_eq!(u8::from(reg.sys_min()), 0b101);
        assert_eq!(reg.boost_lim(), BoostLim::Amp1_5);
    }

    #[test]
    fn charge_current_control() {
        let reg = ChargeCurrentControl::from(0b01011001);
        assert_eq!(u8::from(reg.ichg()), 0b10110);
        assert_eq!(reg.bcold(), false);
        assert_eq!(reg.force_20pct(), true);
    }

    #[test]
    fn charge_termination_timer_control() {
        let reg = ChargeTerminationTimerControl::from(0b10011100);
        assert_eq!(reg.en_term(), true);
        assert_eq!(reg.watchdog(), Watchdog::Sec40);
        assert_eq!(reg.en_timer(), true);
        assert_eq!(reg.charge_timer(), ChgTimer::TwelveHours);
    }

    #[test]
    fn status() {
        let reg = SystemStatus::from(0b10010100);
        assert_eq!(reg.vbus(), Vbus::AdapterPort);
        assert_eq!(reg.chrg(), Chrg::PreCharge);
        assert_eq!(reg.dpm(), false);
        assert_eq!(reg.pg(), true);
        assert_eq!(reg.therm(), false);
        assert_eq!(reg.vsys(), false);
    }

    #[test]
    fn new_fault() {
        let reg = NewFault::from(0b10100010);
        assert_eq!(reg.watchdog_expired(), true);
        assert_eq!(reg.otg_fault(), false);
        assert_eq!(reg.chrg_fault(), ChrgFault::ThermalShutdown);
        assert_eq!(reg.bat_fault(), false);
        assert_eq!(reg.ntc_cold(), true);
        assert_eq!(reg.ntc_hot(), false);
    }
}
