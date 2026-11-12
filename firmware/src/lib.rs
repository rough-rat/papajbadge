//! papajbadge: RTC sleep loop and ABC tune playback.
#![no_std]


use {ch58x_hal as hal};
use ch58x_hal::soft_reset;
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


use qingke::riscv::asm::{nop, wfi};


static barka: &'static [u8] = b"
    X:1
    T:Barka
    M:6/8
    L:1/8
    R:jig
    K:C
    C6 | A3 A3- | AAB cBA | G3 G3- | G3 F2 E | F3 F3- | FFG AGF | E3 E3- |
    E3 C2 C | A3 A3- | AAB cBA | G3 G3- | G3 F2 E | F3 F3- | FDE FED | C6 | x6 |
    E6- | EDE FED | C3 C3- | C3 D2 E | F3 F3-  | F2 F FFE | D3 D3- | D2 G, C2 D |
    E3 E3- | E2 E F2 D | C3 C3 | x6 |
    C6 | A3 A3- | AAB cBA | G3 G3- | G3 F2 E | F3 F3- | FFG AGF | E3 E3- |
    E3 C2 C | A3 A3- | AAB cBA | G3 G3- | G3 F2 E | F3 F3- | FDE FED | C6 | x6 |
    ";

static nokia: &'static [u8] = b"
    X:1
    T:nokia
    M:4/4
    L:1/8
    R:jig
    K:C
    C3 x C x C x | G7 | C2 G6 x |
    ";

pub fn rtc_loop(mut rtc: Rtc,  spawner: Spawner) -> ! {
    rtc.enable_timing(hal::rtc::TimingMode::_1S);
    rtc.ack_timing();

    unsafe {
        qingke::pfic::enable_interrupt(Interrupt::RTC as u8);
    }
    enable_sleep();

    let mut counter: u32 = 0;
    let mut delay = CycleDelay;


    let usrbutpin = unsafe { ch58x_hal::peripherals::PB22::steal() };
    let usrbut = Input::new(usrbutpin, Pull::None);

    let rstbutpin = unsafe { ch58x_hal::peripherals::PB23::steal() };
    let rstbut = Input::new(rstbutpin, Pull::None);

    loop{
        enter_sleep();       
        delay.delay_us(1000);
        let now = rtc.now();
        // log!("T{:02}:{:02}:{:02}, loop {}\n", 
        //     now.hour, now.minute, now.second, counter);
        let ledpin = unsafe { ch58x_hal::peripherals::PA8::steal() };
        let mut led = Output::new(ledpin, Level::Low, OutputDrive::_5mA);
        led.set_low();

        if now.second < 1 {
            // spawner.spawn(play_abc()).unwrap();
            // play_abc();
            // Timer::after(Duration::from_millis(5000)).await;
            led.set_high();
            delay.delay_us(1000);
            led.set_low();
        }
        // if now.hour == 20 && now.minute == 44 && now.second < 5 {
        //     log!("It's 21:36! Time for a tune!\n");
        //     play_abc(barka);
        // }
        if now.year > 2024 && now.hour == 21 && now.minute == 37 && now.second < 10 {
            log!("It's 21:37! Time for a tune!\n");
            play_abc(barka);
        }
        if usrbut.is_low() {
            led.set_high();
            delay.delay_ms(5000);
            led.set_low();
        }
        if rstbut.is_low() {
            log!("Resetting...\n");
            unsafe{soft_reset()};
        }
        counter += 1;
        delay.delay_us(500);
    }
}



// Remove local Event/AbcIter definitions; use the versions from the `abc` crate
use ch58x_hal::peripherals::{PA8,PA4};
pub fn play_abc(tune: &'static [u8]) {
    let iter = AbcIter::new(tune, 320).expect("valid header");
    // let iter = AbcIter::new(abc, 320).expect("valid header");

    let mut tmr = setup_pwm();
    let ledpin = unsafe { PA8::steal() };

    let ena = unsafe { PA4::steal() };
    let mut enapin = Output::new(ena, Level::Low, OutputDrive::_5mA);
    enapin.set_high();

    let mut led = Output::new(ledpin, Level::Low, OutputDrive::_5mA);

    const scaler: u32 = 1000_00;
    let mut delayer = CycleDelay;


    for ev in iter {
        log!("{:?}", ev);
        let delay = ev.duration*320;
        // let delay = ev.duration*300;
        // let period_us :Option<u32> = match ev.freq {
        //     Some(f) => Some(scaler / f),
        //     None => None,
        // };
        // tmr = set_pwm(tmr, period_us, 50);
        tmr = set_pwm(tmr, ev.freq, 50);
        // Timer::after(Duration::from_millis(delay as u64)).await;
        
        delayer.delay_ms(delay);
        led.toggle();
    }
    led.set_low();
    enapin.set_low();


    // while let Some(event) = iter.next() {
    //     tmr = set_pwm(tmr, event.freq, 50);
    //     Timer::after(Duration::from_millis(event.duration as u64)).await;
    // }
}


fn setup_pwm() -> ch58x_hal::pac::TMR0 {
    unsafe {
        let tmr = ch58x_hal::pac::TMR0::steal();
        tmr.ctrl_mod().write(|w| w.tmr_all_clear().bit(true));
        tmr.cnt_end().write(|w| w.cnt_end().bits(0x100));
        tmr.ctrl_mod().write(|w| w.tmr_all_clear().bit(true));
        tmr.ctrl_mod().write(|w| {
            w.tmr_all_clear()
                .bit(false)
                .tmr_mode_in()
                .bit(false)
                .tmr_count_en()
                .bit(true)
                .tmr_out_polar__tmr_cap_count()
                .bit(true)
                .tmr_pwm_repeat__tmr_cap_edge()
                .bits(0b11)
                .tmr_out_en()
                .bit(true)
        });
        tmr.fifo().write(|w| w.fifo().bits(0x08));
        tmr
    }
}


fn set_pwm(tmr: ch58x_hal::pac::TMR0, freq: Option<u32>, duty: u32) -> ch58x_hal::pac::TMR0 {
    {
        unsafe {
            if let Some(freq) = freq {
                let cnt_end = 24_000_000 / freq;
                let duty = (cnt_end * duty) / 100;

                log!("Freq: {}, cnt_end: {}, duty: {}", freq, cnt_end, duty);
                tmr.cnt_end().write(|w| w.cnt_end().bits(cnt_end));
                tmr.fifo().write(|w| w.fifo().bits(duty));
            } else {
                tmr.fifo().write(|w| w.fifo().bits(0x08));
            }
            tmr 
        }
    }
}
