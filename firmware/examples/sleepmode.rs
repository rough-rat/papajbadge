#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

#![no_std]
#![no_main]


use {ch58x_hal as hal};
use hal::delay::CycleDelay;
use hal::gpio::{ Input, Level, Output, OutputDrive, Pull};

use ch58x_hal::pac::Interrupt;
use ch58x_rs_kit::helpers::enable_sleep;

use embedded_hal_local::delay::DelayNs;
use hal::rtc::{Rtc};

use ch58x_rs_kit::{get_configured_rtc, helpers};
use ch58x_rs_kit::log;
use helpers::{blinky, enter_sleep};

use ch58x_rs_kit::logger::init as init_logger;
use embassy_executor::Spawner;

use hal::uart::UartTx;

use abc_note_stream::AbcIter;
use embassy_time::{Delay, Duration, Timer};
use papajbadge::rtc_loop;

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(spawner: Spawner) -> ! {
    let mut config = hal::Config::default(); 
    config.low_power = true; //800uA->150uA
    let p = hal::init(config);

    let mut ena = Output::new(p.PA4, Level::Low, OutputDrive::_5mA);
    ena.set_low();

    let _ = Output::new(p.PA9, Level::Low, OutputDrive::_20mA);

    let but = Input::new(p.PB22, Pull::None);

    let serial = UartTx::new(p.UART0, p.PB7, Default::default()).unwrap();
    init_logger(serial);
    log!( "\n\n\nHello World!");

    let mut rtc = get_configured_rtc();

    if but.is_low() {
        // without that, the board is impossible to reprogram without power cycling
        log!("Button pressed, loopin' time\n");
        let led = Output::new(p.PA8, Level::Low, OutputDrive::_5mA);
        blinky(led);
    } else {
        // let mut led = Output::new(p.PA8, Level::Low, OutputDrive::_5mA);
        // led.set_high();

        rtc_loop(rtc, spawner);
    }
}
