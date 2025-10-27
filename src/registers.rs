use arbitrary_int::u2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
#[repr(u8)]
pub enum OutputStateDefinition {
    DevicePowerDown = 0b00,
    Disabled3State = 0b01,
    DisabledLow = 0b10,
    Enabled = 0b11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
#[repr(u8)]
pub enum OutputStateSelection {
    State0 = 0,
    State1 = 1,
}

impl From<u2> for OutputStateDefinition {
    fn from(value: u2) -> Self {
        match value.value() {
            0b00 => Self::DevicePowerDown,
            0b01 => Self::Disabled3State,
            0b10 => Self::DisabledLow,
            0b11 => Self::Enabled,
            _ => unreachable!(),
        }
    }
}

pub mod generic_configuration {
    use arbitrary_int::{u2, u3};

    use crate::registers::{OutputStateDefinition, OutputStateSelection};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
    #[repr(u8)]
    pub enum DeviceIdentification {
        CDCEL913 = 0,
        CDCE913 = 1,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
    #[repr(u8)]
    pub enum EepromProgrammingStatus {
        Completed = 0,
        InProgress = 1,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
    #[repr(u8)]
    pub enum InputClockSelection {
        Xtal = 0b00,
        Vcxo = 0b01,
        LvCmos = 0b10,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
    #[repr(u8)]
    pub enum Y1ClockSource {
        InputClock = 0,
        Pll1Clock = 1,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
    #[repr(u8)]
    pub enum SerialInterfacePinMode {
        SerialProgrammingInterface = 0,
        ControlS1S2 = 1,
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct GenericConfigurationRegister0(u8);
        impl Debug;
        pub e_el, _: 7;
        pub rid, _: 6, 4;
        pub vid, _: 3, 0;
    }

    impl GenericConfigurationRegister0 {
        pub fn device_identification(&self) -> DeviceIdentification {
            if self.e_el() {
                DeviceIdentification::CDCE913
            } else {
                DeviceIdentification::CDCEL913
            }
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct GenericConfigurationRegister1(u8);
        impl Debug;
        pub _eepip, _: 6;
        pub eelock, set_eelock: 5;
        pub pwdn, set_pwdn: 4;
        pub _inclk, set_inclk: 3, 2;
        pub target_adr, set_target_adr: 1, 0;
    }

    impl GenericConfigurationRegister1 {
        pub fn eeprom_programming_status(&self) -> EepromProgrammingStatus {
            if self._eepip() {
                EepromProgrammingStatus::InProgress
            } else {
                EepromProgrammingStatus::Completed
            }
        }

        pub fn input_clock_selection(&self) -> InputClockSelection {
            match self._inclk() {
                0b00 => InputClockSelection::Xtal,
                0b01 => InputClockSelection::Vcxo,
                0b10 => InputClockSelection::LvCmos,
                _ => InputClockSelection::Xtal, // Reserved maps to Xtal
            }
        }

        pub fn set_input_clock_selection(&mut self, selection: InputClockSelection) {
            self.set_inclk(selection as u8);
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct GenericConfigurationRegister2(u8);
        impl Debug;
        pub _m1, set_m1: 7;
        pub _spicon, set_spicon: 6;
        pub y1_st1, set_y1_st1: 5, 4;
        pub y1_st0, set_y1_st0: 3, 2;
        pub pdiv1_9_8, set_pdiv1_9_8: 1, 0;
    }

    impl GenericConfigurationRegister2 {
        pub fn y1_clock_source(&self) -> Y1ClockSource {
            if !self._m1() {
                Y1ClockSource::InputClock
            } else {
                Y1ClockSource::Pll1Clock
            }
        }

        pub fn set_y1_clock_source(&mut self, source: Y1ClockSource) {
            self.set_m1(source as u8 != 0);
        }

        pub fn serial_interface_pin_mode(&self) -> SerialInterfacePinMode {
            if !self._spicon() {
                SerialInterfacePinMode::SerialProgrammingInterface
            } else {
                SerialInterfacePinMode::ControlS1S2
            }
        }

        pub fn set_serial_interface_pin_mode(&mut self, mode: SerialInterfacePinMode) {
            self.set_spicon(mode as u8 != 0);
        }

        pub fn y1_state1_definition(&self) -> OutputStateDefinition {
            OutputStateDefinition::from(u2::new(self.y1_st1()))
        }

        pub fn set_y1_state1_definition(&mut self, state: OutputStateDefinition) {
            self.set_y1_st1(state as u8);
        }

        pub fn y1_state0_definition(&self) -> OutputStateDefinition {
            OutputStateDefinition::from(u2::new(self.y1_st0()))
        }

        pub fn set_y1_state0_definition(&mut self, state: OutputStateDefinition) {
            self.set_y1_st0(state as u8);
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct GenericConfigurationRegister3(u8);
        impl Debug;
        pub pdiv1_7_0, set_pdiv1_7_0: 7, 0;
    }

    impl GenericConfigurationRegister3 {
        pub fn pdiv1_full_value(&self, reg2: &GenericConfigurationRegister2) -> u16 {
            let upper = reg2.pdiv1_9_8() as u16;
            let lower = self.pdiv1_7_0() as u16;
            (upper << 8) | lower
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct GenericConfigurationRegister4(u8);
        impl Debug;
        pub y1_7, set_y1_7: 7;
        pub y1_6, set_y1_6: 6;
        pub y1_5, set_y1_5: 5;
        pub y1_4, set_y1_4: 4;
        pub y1_3, set_y1_3: 3;
        pub y1_2, set_y1_2: 2;
        pub y1_1, set_y1_1: 1;
        pub y1_0, set_y1_0: 0;
    }

    impl GenericConfigurationRegister4 {
        pub fn y1_state_selection(&self, index: u3) -> OutputStateSelection {
            // Based on Y1_x bits and state definitions in register 2
            // This returns which state (0 or 1) is selected
            // The actual interpretation depends on Y1_ST0 and Y1_ST1 from register 2

            if self.0 & (1 << index.value()) == 0 {
                OutputStateSelection::State0
            } else {
                OutputStateSelection::State1
            }
        }

        pub fn set_y1_state_selection(&mut self, index: u3, state: OutputStateSelection) {
            let bit_value = state as u8;
            self.0 = (self.0 & !(1 << index.value())) | (bit_value << index.value());
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct GenericConfigurationRegister5(u8);
        impl Debug;
        pub xcsel, set_xcsel: 7, 3;
        pub reserved, _: 2, 0;
    }

    impl GenericConfigurationRegister5 {
        pub fn crystal_load_capacitance_pf(&self) -> u8 {
            match self.xcsel() {
                x @ 0x00..0x14 => x,
                _ => 20,
            }
        }

        pub fn set_crystal_load_capacitance_pf(&mut self, pf: u8) {
            let value = match pf {
                x @ 0x00..0x14 => x,
                _ => 20,
            };
            self.set_xcsel(value);
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct GenericConfigurationRegister6(u8);
        impl Debug;
        pub bcount, set_bcount: 7, 1;
        pub eewrite, set_eewrite: 0;
    }
}

pub mod pll1_configuration {
    use arbitrary_int::{u2, u3};

    use crate::registers::{OutputStateDefinition, OutputStateSelection};

    #[repr(u8)]
    pub enum SscModulationAmountDown {
        Off = 0b000,
        Minus025Percent = 0b001,
        Minus05Percent = 0b010,
        Minus075Percent = 0b011,
        Minus1Percent = 0b100,
        Minus125Percent = 0b101,
        Minus15Percent = 0b110,
        Minus2Percent = 0b111,
    }

    impl From<u3> for SscModulationAmountDown {
        fn from(value: u3) -> Self {
            match value.value() {
                0b000 => Self::Off,
                0b001 => Self::Minus025Percent,
                0b010 => Self::Minus05Percent,
                0b011 => Self::Minus075Percent,
                0b100 => Self::Minus1Percent,
                0b101 => Self::Minus125Percent,
                0b110 => Self::Minus15Percent,
                0b111 => Self::Minus2Percent,
                _ => unreachable!(),
            }
        }
    }

    #[repr(u8)]
    pub enum SscModulationAmountCenter {
        Off = 0b000,
        PlusMinus025Percent = 0b001,
        PlusMinus05Percent = 0b010,
        PlusMinus075Percent = 0b011,
        PlusMinus1Percent = 0b100,
        PlusMinus125Percent = 0b101,
        PlusMinus15Percent = 0b110,
        PlusMinus2Percent = 0b111,
    }

    impl From<u3> for SscModulationAmountCenter {
        fn from(value: u3) -> Self {
            match value.value() {
                0b000 => Self::Off,
                0b001 => Self::PlusMinus025Percent,
                0b010 => Self::PlusMinus05Percent,
                0b011 => Self::PlusMinus075Percent,
                0b100 => Self::PlusMinus1Percent,
                0b101 => Self::PlusMinus125Percent,
                0b110 => Self::PlusMinus15Percent,
                0b111 => Self::PlusMinus2Percent,
                _ => unreachable!(),
            }
        }
    }

    #[repr(u8)]
    pub enum Fs1Selection {
        Fvcxo0 = 0,
        Fvcxo1 = 1,
    }

    #[repr(u8)]
    pub enum Pll1Multiplexer {
        Pll1 = 0,
        Pll1Bypass = 1,
    }

    #[repr(u8)]
    pub enum OutputY2Multiplexer {
        Pdiv1 = 0,
        Pdiv2 = 1,
    }

    #[repr(u8)]
    pub enum OutputY3Multiplexer {
        Pdiv1 = 0b00,
        Pdiv2 = 0b01,
        Pdiv3 = 0b10,
        Reserved = 0b11,
    }

    #[repr(u8)]
    pub enum Y2Y3State {
        State0 = 0,
        State1 = 1,
    }

    #[repr(u8)]
    pub enum SscDownCenterSelection {
        Down = 0,
        Center = 1,
    }

    #[repr(u8)]
    pub enum VcoRangeSelection {
        LessThan125MHz = 0b00,
        From125To150MHz = 0b01,
        From150To175MHz = 0b10,
        GreaterOrEqual175MHz = 0b11,
    }

    impl From<u2> for VcoRangeSelection {
        fn from(value: u2) -> Self {
            match value.value() {
                0b00 => Self::LessThan125MHz,
                0b01 => Self::From125To150MHz,
                0b10 => Self::From150To175MHz,
                0b11 => Self::GreaterOrEqual175MHz,
                _ => unreachable!(),
            }
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister0(u8);
        impl Debug;
        pub ssc1_7, set_ssc1_7: 7, 5;
        pub ssc1_6, set_ssc1_6: 4, 2;
        pub ssc1_5, set_ssc1_5: 1, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister1(u8);
        impl Debug;
        pub ssc1_5, set_ssc1_5: 7;
        pub ssc1_4, set_ssc1_4: 6, 4;
        pub ssc1_3, set_ssc1_3: 3, 1;
        pub ssc1_2, set_ssc1_2: 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister2(u8);
        impl Debug;
        pub ssc1_2, set_ssc1_2: 7, 6;
        pub ssc1_1, set_ssc1_1: 6, 3;
        pub ssc1_0, set_ssc1_0: 2, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister3(u8);
        impl Debug;
        pub fs1_7, set_fs1_7: 7;
        pub fs1_6, set_fs1_6: 6;
        pub fs1_5, set_fs1_5: 5;
        pub fs1_4, set_fs1_4: 4;
        pub fs1_3, set_fs1_3: 3;
        pub fs1_2, set_fs1_2: 2;
        pub fs1_1, set_fs1_1: 1;
        pub fs1_0, set_fs1_0: 0;
    }

    impl Pll1ConfigurationRegister3 {
        pub fn fs1_selection(&self, index: u3) -> Fs1Selection {
            // Based on Y1_x bits and state definitions in register 2
            // This returns which state (0 or 1) is selected
            // The actual interpretation depends on Y1_ST0 and Y1_ST1 from register 2

            if self.0 & (1 << index.value()) == 0 {
                Fs1Selection::Fvcxo0
            } else {
                Fs1Selection::Fvcxo1
            }
        }

        pub fn set_fs1_selection(&mut self, index: u3, state: Fs1Selection) {
            let bit_value = state as u8;
            self.0 = (self.0 & !(1 << index.value())) | (bit_value << index.value());
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister4(u8);
        impl Debug;
        pub _mux1, set_mux1: 7;
        pub _m2, set_m2: 6;
        pub _m3, set_m3: 5, 4;
        pub y2y3_st1, set_y2y3_st1: 3, 2;
        pub y2y3_st0, set_y2y3_st0: 1, 0;
    }

    impl Pll1ConfigurationRegister4 {
        pub fn pll1_multiplexer(&self) -> Pll1Multiplexer {
            if !self._mux1() {
                Pll1Multiplexer::Pll1
            } else {
                Pll1Multiplexer::Pll1Bypass
            }
        }

        pub fn set_pll1_multiplexer(&mut self, mux: Pll1Multiplexer) {
            self.set_mux1(mux as u8 != 0);
        }

        pub fn output_y2_multiplexer(&self) -> OutputY2Multiplexer {
            if !self._m2() {
                OutputY2Multiplexer::Pdiv1
            } else {
                OutputY2Multiplexer::Pdiv2
            }
        }

        pub fn set_output_y2_multiplexer(&mut self, mux: OutputY2Multiplexer) {
            self.set_m2(mux as u8 != 0);
        }

        pub fn output_y3_multiplexer(&self) -> OutputY3Multiplexer {
            match self._m3() {
                0b00 => OutputY3Multiplexer::Pdiv1,
                0b01 => OutputY3Multiplexer::Pdiv2,
                0b10 => OutputY3Multiplexer::Pdiv3,
                0b11 => OutputY3Multiplexer::Reserved,
                _ => unreachable!(),
            }
        }

        pub fn set_output_y3_multiplexer(&mut self, mux: OutputY3Multiplexer) {
            self.set_m3(mux as u8);
        }

        pub fn y2y3_state1_definition(&self) -> OutputStateDefinition {
            OutputStateDefinition::from(u2::new(self.y2y3_st1()))
        }

        pub fn set_y2y3_state1_definition(&mut self, state: OutputStateDefinition) {
            self.set_y2y3_st1(state as u8);
        }

        pub fn y2y3_state0_definition(&self) -> OutputStateDefinition {
            OutputStateDefinition::from(u2::new(self.y2y3_st0()))
        }

        pub fn set_y2y3_state0_definition(&mut self, state: OutputStateDefinition) {
            self.set_y2y3_st0(state as u8);
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister5(u8);
        impl Debug;
        pub y2y3_7, set_y2y3_7: 7;
        pub y2y3_6, set_y2y3_6: 6;
        pub y2y3_5, set_y2y3_5: 5;
        pub y2y3_4, set_y2y3_4: 4;
        pub y2y3_3, set_y2y3_3: 3;
        pub y2y3_2, set_y2y3_2: 2;
        pub y2y3_1, set_y2y3_1: 1;
        pub y2y3_0, set_y2y3_0: 0;
    }

    impl Pll1ConfigurationRegister5 {
        pub fn y2y3_state_selection(&self, index: u3) -> OutputStateSelection {
            // Based on Y2Y3_x bits and state definitions in register 4
            // This returns which state (0 or 1) is selected
            // The actual interpretation depends on Y2Y3_ST0 and Y2Y3_ST1 from register 4

            if self.0 & (1 << index.value()) == 0 {
                OutputStateSelection::State0
            } else {
                OutputStateSelection::State1
            }
        }

        pub fn set_y2y3_state_selection(&mut self, index: u3, state: OutputStateSelection) {
            let bit_value = state as u8;
            self.0 = (self.0 & !(1 << index.value())) | (bit_value << index.value());
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister6(u8);
        impl Debug;
        pub _ssc1dc, set_ssc1dc: 7;
        pub pdiv2, set_pdiv2: 6, 0;
    }

    impl Pll1ConfigurationRegister6 {
        pub fn pll1_ssc_down_center_selection(&self) -> SscDownCenterSelection {
            if !self._ssc1dc() {
                SscDownCenterSelection::Down
            } else {
                SscDownCenterSelection::Center
            }
        }

        pub fn set_pll1_ssc_down_center_selection(&mut self, selection: SscDownCenterSelection) {
            self.set_ssc1dc(selection as u8 != 0);
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister7(u8);
        impl Debug;
        pub reserved, _: 7;
        pub pdiv3, set_pdiv3: 6, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister8(u8);
        impl Debug;
        pub pll1_0n_11_4, set_pll1_0n_11_4: 7, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegister9(u8);
        impl Debug;
        pub pll1_0n_3_0, set_pll1_0n_3_0: 7, 4;
        pub pll1_0r_8_5, set_pll1_0r_8_5: 3, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegisterA(u8);
        impl Debug;
        pub pll1_0r_4_0, set_pll1_0r_4_0: 7, 3;
        pub pll1_0q_5_3, set_pll1_0q_5_3: 2, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegisterB(u8);
        impl Debug;
        pub pll1_0q_2_0, set_pll1_0q_2_0: 7, 5;
        pub pll1_0p_2_0, set_pll1_0p_2_0: 4, 2;
      pub _vco1_0_range, set_vco1_0_range: 1, 0;
    }

    impl Pll1ConfigurationRegisterB {
        pub fn vco1_0_range_selection(&self) -> VcoRangeSelection {
            u2::new(self._vco1_0_range()).into()
        }

        pub fn set_vco1_0_range_selection(&mut self, range: VcoRangeSelection) {
            self.set_vco1_0_range(range as u8);
        }
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegisterC(u8);
        impl Debug;
        pub pll1_1n_11_4, set_pll1_1n_11_4: 7, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegisterD(u8);
        impl Debug;
        pub pll1_1n_3_0, set_pll1_1n_3_0: 7, 4;
        pub pll1_1r_8_5, set_pll1_1r_8_5: 3, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegisterE(u8);
        impl Debug;
        pub pll1_1r_4_0, set_pll1_1r_4_0: 7, 3;
        pub pll1_1q_5_3, set_pll1_1q_5_3: 2, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct Pll1ConfigurationRegisterF(u8);
        impl Debug;
        pub pll1_1q_2_0, set_pll1_1q_2_0: 7, 5;
        pub pll1_1p_2_0, set_pll1_1p_2_0: 4, 2;
        pub _vco1_1_range, set_vco1_1_range: 1, 0;
    }

    bitfield::bitfield! {
        #[derive(Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
        pub struct PllSettings(u32);
        impl Debug;
        u16;
        pub pllx_yn, set_pllx_yn: 31, 20;
        pub pllx_yr, set_pllx_yr: 19, 11;
        u8;
        pub pllx_yq, set_pllx_yq: 10, 5;
        pub pllx_yp, set_pllx_yp: 4, 2;
        pub _vco1x_y_range, set_vcox_y_range: 1, 0;
    }

    impl PllSettings {
        pub fn vco_range_selection(&self) -> VcoRangeSelection {
            u2::new(self._vco1x_y_range()).into()
        }

        pub fn set_vco_range_selection(&mut self, range: VcoRangeSelection) {
            self.set_vcox_y_range(range as u8);
        }
    }
}
