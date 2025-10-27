#![no_std]
#![no_main]

use cdcx913::registers::OutputStateSelection;
use cdcx913::registers::generic_configuration::InputClockSelection;
use cdcx913::registers::pll1_configuration::Pll1Multiplexer;
use cdcx913::{u10, u3};
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Config, I2c};
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("Hello world!");

    let p = embassy_stm32::init(Default::default());

    let i2c = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, p.DMA1_CH6, p.DMA1_CH0, {
        let mut cfg = Config::default();
        cfg.scl_pullup = true;
        cfg.sda_pullup = true;
        cfg
    });

    defmt::info!("Initializing");

    let mut cdcx913 = cdcx913::CDCx913::new(i2c);

    defmt::debug!(
        "Device Identification = {}",
        cdcx913.device_identification().await.unwrap()
    );
    defmt::debug!(
        "Revision Number = {}",
        cdcx913.revision_number().await.unwrap()
    );
    defmt::debug!(
        "Vendor Identification = {}",
        cdcx913.vendor_identification().await.unwrap()
    );

    defmt::debug!(
        "PLL1_0 Settings = {}",
        defmt::Debug2Format(&cdcx913.pll1_0_settings().await.unwrap())
    );
    defmt::debug!(
        "PLL1_1 Settings = {}",
        defmt::Debug2Format(&cdcx913.pll1_0_settings().await.unwrap())
    );

    cdcx913
        .set_input_clock(InputClockSelection::LvCmos)
        .await
        .unwrap();

    cdcx913
        .set_pll1_multiplexer(Pll1Multiplexer::Pll1)
        .await
        .unwrap();

    cdcx913
        .set_y1_state_selection(u3::new(0), OutputStateSelection::State1)
        .await
        .unwrap();
    
    cdcx913
        .set_y1_output_divider(u10::new(2))
        .await
        .unwrap();

    loop {
        Timer::after(Duration::from_micros(1_000)).await;
    }
}
