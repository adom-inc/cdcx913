#![no_std]

pub use arbitrary_int::{u2, u3, u4, u7, u10};
use embedded_hal_async::i2c::I2c;

use crate::{
    i2c::{ADDRESS, CommandCode, OpCode},
    registers::{
        OutputStateDefinition, OutputStateSelection,
        generic_configuration::{
            DeviceIdentification, EepromProgrammingStatus, GenericConfigurationRegister0,
            GenericConfigurationRegister1, GenericConfigurationRegister2,
            GenericConfigurationRegister3, GenericConfigurationRegister4,
            GenericConfigurationRegister5, GenericConfigurationRegister6, InputClockSelection,
            SerialInterfacePinMode, Y1ClockSource,
        },
        pll1_configuration::{
            Fs1Selection, OutputY2Multiplexer, OutputY3Multiplexer, Pll1ConfigurationRegister0,
            Pll1ConfigurationRegister1, Pll1ConfigurationRegister2, Pll1ConfigurationRegister3,
            Pll1ConfigurationRegister4, Pll1ConfigurationRegister5, Pll1ConfigurationRegister6,
            Pll1ConfigurationRegister7, Pll1Multiplexer, PllSettings, SscDownCenterSelection,
            SscModulationAmountCenter, SscModulationAmountDown,
        },
    },
};

pub mod i2c;
pub mod registers;

pub struct CDCx913<I2C>
where
    I2C: I2c,
{
    i2c: I2C,
}

#[repr(u8)]
enum Register {
    // Available offsets are [0x0, 0x6]
    GenericConfiguration = 0x00,
    // Available offsets are [0x0, 0xF]
    Pll1Configuration = 0x10,
}

macro_rules! read {
    ($self:expr, $register:ident, $offset:expr, $fn:expr) => {
        paste::paste! {
            $self.with::<[<$register Register $offset>], _>(
                Register::$register as u8 + $offset,
                $fn
            )
            .await
        }
    };
}

macro_rules! modify {
    ($self:expr, $register:ident, $offset:expr, $fn:expr) => {
        paste::paste! {
            $self.modify_byte_unchecked::<[<$register Register $offset>], _>(
                Register::$register as u8 + $offset,
                $fn
            )
            .await
        }
    };
}

impl<I2C: I2c> CDCx913<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    // Writes the byte at the given offset without checking that the address is
    // valid. This is safe as far as the code is concerned but may cause
    // unexpected or undefined behavior in the PLL if the target offset is not
    // in the valid range. According to the datasheet, writing beyond 0x20 "may
    // affect device function", so proceed at your own risk.
    pub async fn write_byte_unchecked(&mut self, offset: u8, value: u8) -> Result<(), I2C::Error> {
        self.i2c
            .write(
                ADDRESS,
                &[CommandCode::new(OpCode::Byte, offset as u8).into(), value],
            )
            .await
    }

    // Reads the byte at the given offset without checking that the address is
    // valid. This is generally safe but the result may not be deterministic if
    // the offset isn't in the allowed range.
    pub async fn read_byte_unchecked(&mut self, offset: u8) -> Result<u8, I2C::Error> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(
                ADDRESS,
                &[CommandCode::new(OpCode::Byte, offset).into()],
                &mut buf,
            )
            .await?;

        Ok(buf[0])
    }

    async fn with<T: From<u8>, R>(
        &mut self,
        offset: u8,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, I2C::Error> {
        let reg: T = self.read_byte_unchecked(offset).await?.into();

        let r = f(&reg);

        Ok(r)
    }

    pub async fn modify_byte_unchecked<T: From<u8> + Into<u8>, R>(
        &mut self,
        offset: u8,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, I2C::Error> {
        let mut reg: T = self.read_byte_unchecked(offset).await?.into();

        let r = f(&mut reg);

        self.write_byte_unchecked(offset, reg.into()).await?;

        Ok(r)
    }

    #[doc(alias = "e_el")]
    pub async fn device_identification(&mut self) -> Result<DeviceIdentification, I2C::Error> {
        read!(self, GenericConfiguration, 0, |reg| reg
            .device_identification())
    }

    #[doc(alias = "rid")]
    pub async fn revision_number(&mut self) -> Result<u3, I2C::Error> {
        read!(self, GenericConfiguration, 0, |reg| u3::new(reg.rid()))
    }

    #[doc(alias = "vid")]
    pub async fn vendor_identification(&mut self) -> Result<u4, I2C::Error> {
        read!(self, GenericConfiguration, 0, |reg| u4::new(reg.vid()))
    }

    #[doc(alias = "eepip")]
    pub async fn eeprom_programming_status(
        &mut self,
    ) -> Result<EepromProgrammingStatus, I2C::Error> {
        read!(self, GenericConfiguration, 1, |reg| reg
            .eeprom_programming_status())
    }

    #[doc(alias = "eelock")]
    pub async fn eeprom_permanently_locked(&mut self) -> Result<bool, I2C::Error> {
        read!(self, GenericConfiguration, 1, |reg| reg.eelock())
    }

    /// Must be written to the EEPROM by calling [`Self::initiate_eeprom_write`]
    /// to take effect. Once flashed, forces the EEPROM into a locked, read-only
    /// state. On the fly configuration is still allowed but EEPROM is no longer
    /// writeable.
    #[doc(alias = "set_eelock")]
    pub async fn set_eeprom_permanently_locked(&mut self, locked: bool) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 1, |reg| reg.set_eelock(locked))
    }

    #[doc(alias = "pwdn")]
    pub async fn power_down(&mut self) -> Result<bool, I2C::Error> {
        read!(self, GenericConfiguration, 1, |reg| reg.pwdn())
    }

    #[doc(alias = "set_pwdn")]
    pub async fn set_power_down(&mut self, value: bool) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 1, |reg| reg.set_pwdn(value))
    }

    #[doc(alias = "inclk")]
    pub async fn input_clock(&mut self) -> Result<InputClockSelection, I2C::Error> {
        read!(self, GenericConfiguration, 1, |reg| reg
            .input_clock_selection())
    }

    #[doc(alias = "set_inclk")]
    pub async fn set_input_clock(&mut self, value: InputClockSelection) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 1, |reg| reg
            .set_input_clock_selection(value))
    }

    #[doc(alias = "target_adr")]
    pub async fn target_address(&mut self) -> Result<u2, I2C::Error> {
        read!(self, GenericConfiguration, 1, |reg| u2::new(
            reg.target_adr()
        ))
    }

    #[doc(alias = "set_target_adr")]
    pub async fn set_target_address(&mut self, value: u2) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 1, |reg| reg
            .set_target_adr(value.value()))
    }

    #[doc(alias = "m1")]
    pub async fn y1_clock_source(&mut self) -> Result<Y1ClockSource, I2C::Error> {
        read!(self, GenericConfiguration, 2, |reg| reg.y1_clock_source())
    }

    #[doc(alias = "set_m1")]
    pub async fn set_y1_clock_source(&mut self, value: Y1ClockSource) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 2, |reg| reg
            .set_y1_clock_source(value))
    }

    #[doc(alias = "spicon")]
    pub async fn serial_pins_operating_mode(
        &mut self,
    ) -> Result<SerialInterfacePinMode, I2C::Error> {
        read!(self, GenericConfiguration, 2, |reg| reg
            .serial_interface_pin_mode())
    }

    #[doc(alias = "set_spicon")]
    pub async fn set_serial_pins_operating_mode(
        &mut self,
        value: SerialInterfacePinMode,
    ) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 2, |reg| reg
            .set_serial_interface_pin_mode(value))
    }

    #[doc(alias = "y1_st1")]
    pub async fn y1_state_1(&mut self) -> Result<OutputStateDefinition, I2C::Error> {
        read!(self, GenericConfiguration, 2, |reg| reg
            .y1_state1_definition())
    }

    #[doc(alias = "set_y1_st1")]
    pub async fn set_y1_state_1(&mut self, value: OutputStateDefinition) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 2, |reg| reg
            .set_y1_state1_definition(value))
    }

    #[doc(alias = "y1_st0")]
    pub async fn y1_state_0(&mut self) -> Result<OutputStateDefinition, I2C::Error> {
        read!(self, GenericConfiguration, 2, |reg| reg
            .y1_state0_definition())
    }

    #[doc(alias = "set_y1_st0")]
    pub async fn set_y1_state_0(&mut self, value: OutputStateDefinition) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 2, |reg| reg
            .set_y1_state0_definition(value))
    }

    #[doc(alias = "pdiv1")]
    pub async fn y1_output_divider(&mut self) -> Result<u10, I2C::Error> {
        let reg2 = read!(self, GenericConfiguration, 2, |reg| *reg)?;
        let reg3 = read!(self, GenericConfiguration, 3, |reg| *reg)?;

        Ok(u10::new(reg3.pdiv1_full_value(&reg2)))
    }

    #[doc(alias = "set_pdiv1")]
    pub async fn set_y1_output_divider(&mut self, value: u10) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 2, |reg| reg
            .set_pdiv1_9_8((value.value() >> 8) as _))?;
        modify!(self, GenericConfiguration, 3, |reg| reg
            .set_pdiv1_7_0((value.value() & 0xFF) as _))
    }

    #[doc(alias = "y1_x")]
    pub async fn y1_state_selection(
        &mut self,
        control_input: u3,
    ) -> Result<OutputStateSelection, I2C::Error> {
        read!(self, GenericConfiguration, 4, |reg| reg
            .y1_state_selection(control_input))
    }

    #[doc(alias = "set_y1_x")]
    pub async fn set_y1_state_selection(
        &mut self,
        control_input: u3,
        value: OutputStateSelection,
    ) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 4, |reg| reg
            .set_y1_state_selection(control_input, value))
    }

    /// Returns the capacitance in pF, not the raw value of the register field
    #[doc(alias = "xcsel")]
    pub async fn crystal_load_capacitance_pf(&mut self) -> Result<u8, I2C::Error> {
        read!(self, GenericConfiguration, 5, |reg| reg
            .crystal_load_capacitance_pf())
    }

    #[doc(alias = "set_xcsel")]
    pub async fn set_crystal_load_capacitor(&mut self, value: u8) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 5, |reg| reg
            .set_crystal_load_capacitance_pf(value))
    }

    #[doc(alias = "bcount")]
    pub async fn block_byte_count(&mut self) -> Result<u7, I2C::Error> {
        read!(self, GenericConfiguration, 6, |reg| u7::new(reg.bcount()))
    }

    #[doc(alias = "set_bcount")]
    pub async fn set_block_byte_count(&mut self, value: u7) -> Result<(), I2C::Error> {
        modify!(self, GenericConfiguration, 6, |reg| reg
            .set_bcount(value.value()))
    }

    #[doc(alias = "eewrite")]
    pub async fn initiate_eeprom_write(&mut self) -> Result<bool, I2C::Error> {
        todo!("set EEWRITE high and then wait for EEPIP to be complete")
    }

    /* ==== PLL Config ==== */

    #[doc(alias = "ssc1_x")]
    pub async fn spread_spectrum_clocking_selection_raw(
        &mut self,
        control_input: u3,
    ) -> Result<u3, I2C::Error> {
        Ok(u3::new(match control_input.value() {
            7 => read!(self, Pll1Configuration, 0, |reg| reg.ssc1_7())?,
            6 => read!(self, Pll1Configuration, 0, |reg| reg.ssc1_6())?,
            5 => {
                let hi = read!(self, Pll1Configuration, 0, |reg| reg.ssc1_5())?;
                let lo = read!(self, Pll1Configuration, 1, |reg| reg.ssc1_5() as u8)?;

                (hi << 1) | lo
            }
            4 => read!(self, Pll1Configuration, 1, |reg| reg.ssc1_4())?,
            3 => read!(self, Pll1Configuration, 1, |reg| reg.ssc1_3())?,
            2 => {
                let hi = read!(self, Pll1Configuration, 1, |reg| reg.ssc1_2() as u8)?;
                let lo = read!(self, Pll1Configuration, 2, |reg| reg.ssc1_2())?;

                (hi << 2) | lo
            }
            1 => read!(self, Pll1Configuration, 2, |reg| reg.ssc1_1())?,
            0 => read!(self, Pll1Configuration, 2, |reg| reg.ssc1_0())?,
            _ => unreachable!(),
        }))
    }

    #[doc(alias = "ssc1_x_down")]
    pub async fn spread_spectrum_clocking_selection_as_down(
        &mut self,
        control_input: u3,
    ) -> Result<SscModulationAmountDown, I2C::Error> {
        let raw_value = self
            .spread_spectrum_clocking_selection_raw(control_input)
            .await?;

        Ok(SscModulationAmountDown::from(raw_value))
    }

    #[doc(alias = "ssc1_x_center")]
    pub async fn spread_spectrum_clocking_selection_as_center(
        &mut self,
        control_input: u3,
    ) -> Result<SscModulationAmountCenter, I2C::Error> {
        let raw_value = self
            .spread_spectrum_clocking_selection_raw(control_input)
            .await?;

        Ok(SscModulationAmountCenter::from(raw_value))
    }

    #[doc(alias = "set_ssc1_x")]
    pub async fn set_spread_spectrum_clocking_selection_raw(
        &mut self,
        control_input: u3,
        value: u3,
    ) -> Result<(), I2C::Error> {
        let value = value.value();

        Ok(match control_input.value() {
            7 => modify!(self, Pll1Configuration, 0, |reg| reg.set_ssc1_7(value))?,
            6 => modify!(self, Pll1Configuration, 0, |reg| reg.set_ssc1_6(value))?,
            5 => {
                let hi = value >> 1;
                let lo = value & 0b001;

                modify!(self, Pll1Configuration, 0, |reg| reg.set_ssc1_5(hi))?;
                modify!(self, Pll1Configuration, 1, |reg| reg.set_ssc1_5(lo != 0))?;
            }
            4 => modify!(self, Pll1Configuration, 1, |reg| reg.set_ssc1_4(value))?,
            3 => modify!(self, Pll1Configuration, 1, |reg| reg.set_ssc1_3(value))?,
            2 => {
                let hi = value >> 2;
                let lo = value & 0b011;

                modify!(self, Pll1Configuration, 1, |reg| reg.set_ssc1_2(hi != 0))?;
                modify!(self, Pll1Configuration, 2, |reg| reg.set_ssc1_2(lo))?;
            }
            1 => modify!(self, Pll1Configuration, 2, |reg| reg.set_ssc1_1(value))?,
            0 => modify!(self, Pll1Configuration, 2, |reg| reg.set_ssc1_0(value))?,
            _ => unreachable!(),
        })
    }

    #[doc(alias = "set_ssc1_x_down")]
    pub async fn set_spread_spectrum_clocking_selection_as_down(
        &mut self,
        control_input: u3,
        value: SscModulationAmountDown,
    ) -> Result<(), I2C::Error> {
        self.set_spread_spectrum_clocking_selection_raw(control_input, u3::new(value as u8))
            .await
    }

    #[doc(alias = "set_ssc1_x_center")]
    pub async fn set_spread_spectrum_clocking_selection_as_center(
        &mut self,
        control_input: u3,
        value: SscModulationAmountCenter,
    ) -> Result<(), I2C::Error> {
        self.set_spread_spectrum_clocking_selection_raw(control_input, u3::new(value as u8))
            .await
    }

    #[doc(alias = "fs1_x")]
    pub async fn pll1_frequency_selection(
        &mut self,
        control_input: u3,
    ) -> Result<Fs1Selection, I2C::Error> {
        read!(self, Pll1Configuration, 3, |reg| reg
            .fs1_selection(control_input))
    }

    #[doc(alias = "set_fs1_x")]
    pub async fn set_pll1_frequency_selection(
        &mut self,
        control_input: u3,
        value: Fs1Selection,
    ) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 3, |reg| reg
            .set_fs1_selection(control_input, value))
    }

    #[doc(alias = "mux1")]
    pub async fn pll1_multiplexer(&mut self) -> Result<Pll1Multiplexer, I2C::Error> {
        read!(self, Pll1Configuration, 4, |reg| reg.pll1_multiplexer())
    }

    #[doc(alias = "set_mux1")]
    pub async fn set_pll1_multiplexer(&mut self, value: Pll1Multiplexer) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 4, |reg| reg
            .set_pll1_multiplexer(value))
    }

    #[doc(alias = "m2")]
    pub async fn y2_multiplexer(&mut self) -> Result<OutputY2Multiplexer, I2C::Error> {
        read!(self, Pll1Configuration, 4, |reg| reg
            .output_y2_multiplexer())
    }

    #[doc(alias = "set_m2")]
    pub async fn set_y2_multiplexer(
        &mut self,
        value: OutputY2Multiplexer,
    ) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 4, |reg| reg
            .set_output_y2_multiplexer(value))
    }

    #[doc(alias = "m3")]
    pub async fn y3_multiplexer(&mut self) -> Result<OutputY3Multiplexer, I2C::Error> {
        read!(self, Pll1Configuration, 4, |reg| reg
            .output_y3_multiplexer())
    }

    #[doc(alias = "set_m3")]
    pub async fn set_y3_multiplexer(
        &mut self,
        value: OutputY3Multiplexer,
    ) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 4, |reg| reg
            .set_output_y3_multiplexer(value))
    }

    #[doc(alias = "y2y3_st1")]
    pub async fn y2y3_state1_definition(&mut self) -> Result<OutputStateDefinition, I2C::Error> {
        read!(self, Pll1Configuration, 4, |reg| reg
            .y2y3_state1_definition())
    }

    #[doc(alias = "set_y2y3_st1")]
    pub async fn set_y2y3_state1_definition(
        &mut self,
        value: OutputStateDefinition,
    ) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 4, |reg| reg
            .set_y2y3_state1_definition(value))
    }

    #[doc(alias = "y2y3_st0")]
    pub async fn y2y3_state0_definition(&mut self) -> Result<OutputStateDefinition, I2C::Error> {
        read!(self, Pll1Configuration, 4, |reg| reg
            .y2y3_state0_definition())
    }

    #[doc(alias = "set_y2y3_st0")]
    pub async fn set_y2y3_state0_definition(
        &mut self,
        value: OutputStateDefinition,
    ) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 4, |reg| reg
            .set_y2y3_state0_definition(value))
    }

    #[doc(alias = "y2y3_x")]
    pub async fn y2y3_state_selection(
        &mut self,
        control_input: u3,
    ) -> Result<OutputStateSelection, I2C::Error> {
        read!(self, Pll1Configuration, 5, |reg| reg
            .y2y3_state_selection(control_input))
    }

    #[doc(alias = "set_y2y3_x")]
    pub async fn set_y2y3_state_selection(
        &mut self,
        control_input: u3,
        value: OutputStateSelection,
    ) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 5, |reg| reg
            .set_y2y3_state_selection(control_input, value))
    }

    #[doc(alias = "ssc1dc")]
    pub async fn pll1_ssc_down_center_selection(
        &mut self,
    ) -> Result<SscDownCenterSelection, I2C::Error> {
        read!(self, Pll1Configuration, 6, |reg| reg
            .pll1_ssc_down_center_selection())
    }

    #[doc(alias = "set_ssc1dc")]
    pub async fn set_pll1_ssc_down_center_selection(
        &mut self,
        value: SscDownCenterSelection,
    ) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 6, |reg| reg
            .set_pll1_ssc_down_center_selection(value))
    }

    #[doc(alias = "pdiv2")]
    pub async fn y2_output_divider(&mut self) -> Result<u7, I2C::Error> {
        read!(self, Pll1Configuration, 6, |reg| u7::new(reg.pdiv2()))
    }

    #[doc(alias = "set_pdiv2")]
    pub async fn set_y2_output_divider(&mut self, value: u7) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 6, |reg| reg
            .set_pdiv2(value.value()))
    }

    #[doc(alias = "pdiv3")]
    pub async fn y3_output_divider(&mut self) -> Result<u7, I2C::Error> {
        read!(self, Pll1Configuration, 7, |reg| u7::new(reg.pdiv3()))
    }

    #[doc(alias = "set_pdiv3")]
    pub async fn set_y3_output_divider(&mut self, value: u7) -> Result<(), I2C::Error> {
        modify!(self, Pll1Configuration, 7, |reg| reg
            .set_pdiv3(value.value()))
    }

    #[doc(alias = "pll1_0")]
    pub async fn pll1_0_settings(&mut self) -> Result<PllSettings, I2C::Error> {
        let bytes = [
            self.read_byte_unchecked(Register::Pll1Configuration as u8 + 0x8)
                .await?,
            self.read_byte_unchecked(Register::Pll1Configuration as u8 + 0x9)
                .await?,
            self.read_byte_unchecked(Register::Pll1Configuration as u8 + 0xA)
                .await?,
            self.read_byte_unchecked(Register::Pll1Configuration as u8 + 0xB)
                .await?,
        ];

        Ok(PllSettings(u32::from_be_bytes(bytes)))
    }

    #[doc(alias = "set_pll1_0")]
    pub async fn set_pll1_0_settings(&mut self, value: PllSettings) -> Result<(), I2C::Error> {
        let bytes = value.0.to_be_bytes();

        self.write_byte_unchecked(Register::Pll1Configuration as u8 + 0x8, bytes[0])
            .await?;
        self.write_byte_unchecked(Register::Pll1Configuration as u8 + 0x9, bytes[1])
            .await?;
        self.write_byte_unchecked(Register::Pll1Configuration as u8 + 0xA, bytes[2])
            .await?;
        self.write_byte_unchecked(Register::Pll1Configuration as u8 + 0xB, bytes[3])
            .await?;

        Ok(())
    }

    #[doc(alias = "pll1_1")]
    pub async fn pll1_1_settings(&mut self) -> Result<PllSettings, I2C::Error> {
        let bytes = [
            self.read_byte_unchecked(Register::Pll1Configuration as u8 + 0xC)
                .await?,
            self.read_byte_unchecked(Register::Pll1Configuration as u8 + 0xD)
                .await?,
            self.read_byte_unchecked(Register::Pll1Configuration as u8 + 0xE)
                .await?,
            self.read_byte_unchecked(Register::Pll1Configuration as u8 + 0xF)
                .await?,
        ];

        Ok(PllSettings(u32::from_be_bytes(bytes)))
    }

    #[doc(alias = "set_pll1_1")]
    pub async fn set_pll1_1_settings(&mut self, value: PllSettings) -> Result<(), I2C::Error> {
        let bytes = value.0.to_be_bytes();

        self.write_byte_unchecked(Register::Pll1Configuration as u8 + 0xC, bytes[0])
            .await?;
        self.write_byte_unchecked(Register::Pll1Configuration as u8 + 0xD, bytes[1])
            .await?;
        self.write_byte_unchecked(Register::Pll1Configuration as u8 + 0xE, bytes[2])
            .await?;
        self.write_byte_unchecked(Register::Pll1Configuration as u8 + 0xF, bytes[3])
            .await?;

        Ok(())
    }
}
