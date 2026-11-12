#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![no_std]
#![no_main]

use ch58x_hal as hal;
use embassy_executor::Spawner;
use hal::gpio::{Level, Output, OutputDrive, Pin};
use hal::uart::UartTx;
use hal::{ble};
use ch58x_rs_kit::logger::init as init_logger;
use ch58x_rs_kit::{get_configured_rtc, log, tmos_mainloop};

use ch58x_rs_kit::ble_periph::{common_init, devinfo_init, peripheral};
// use ch58x_rs_kit::ble_periph::blinky_service::{blinky_init, blinky_service_loop};
use ch58x_rs_kit::ble_periph::current_time_service::current_time_init;

use embassy_time::Duration;
use embassy_time::Timer;
use ch58x_hal::gpio::AnyPin;

use papajbadge::play_abc;


static minibarka: &'static [u8] = b"
    X:1
    T:Barka
    M:4/8
    L:1/8
    R:jig
    K:C
    DEFE | DC x2 |
    ";

#[embassy_executor::task]
async fn async_blink(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, OutputDrive::_5mA);

    loop {
        led.set_high();
        Timer::after(Duration::from_millis(150)).await;
        led.set_low();
        Timer::after(Duration::from_millis(150)).await;
    }
}


#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(spawner: Spawner) -> ! {
    let mut config = hal::Config::default();
    config.clock.use_pll_60mhz().enable_lse();
    let p = hal::init(config);
    hal::embassy::init();

    let mut ena = Output::new(p.PA4, Level::Low, OutputDrive::_5mA);
    ena.set_low();
    let _ = Output::new(p.PA9, Level::Low, OutputDrive::_20mA);

    let serial = UartTx::new(p.UART0, p.PB7, Default::default()).unwrap();
    init_logger(serial);
    log!("\n\n\nHello World!");

    let ledpin = unsafe { ch58x_hal::peripherals::PA8::steal() };
    let mut led = Output::new(ledpin, Level::Low, OutputDrive::_5mA);

    play_abc(minibarka);
    led.set_high();

    // spawner.spawn(async_blink(p.PA8.degrade())).unwrap();

    let rtc = get_configured_rtc();

    log!("System Clocks: {}", hal::sysctl::clocks().hclk);
    log!("ChipID: 0x{:02x}", hal::signature::get_chip_id());
    log!("RTC datetime: {}", rtc.now());

    log!("BLE Lib Version: {}", ble::lib_version());

    let mut ble_config = ble::Config::default();
    ble_config.pa_config = None;
    ble_config.mac_addr = [0x21, 0x37, 0x04, 0x20, 0x69, 0x96].into();
    let (task_id, sub) = hal::ble::init(ble_config).unwrap();
    log!("BLE hal task id: {}", task_id);

    unsafe {
        common_init();
        devinfo_init();
        // blinky_init();
        current_time_init();
    }

    // Spawn a processor for WCH's "os" that runs in parallel with Embassy 
    spawner.spawn(tmos_mainloop()).unwrap();

    // Application code
    peripheral(spawner, task_id, sub, rtc, Some(papajbadge::rtc_loop)).await
}
