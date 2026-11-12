# papajbadge firmware

Rust firmware for papajbadge (CH582): BLE peripheral with current time service, RTC, and a
low-power loop that plays a tune at 21:37.

- `cebula_fw`: final firmware
- `examples/sleepmode.rs`: sleep mode / RTC loop PoC
- `utils/`: `cts_writer.html` sets the badge clock over Web Bluetooth, `blogpost.md` about the badge

## Building and flashing

Requirements: [rustup](https://rustup.rs) (the toolchain is pinned in `rust-toolchain.toml`),
[wlink](https://github.com/ch32-rs/wlink) and a WCH-LinkE probe (`cargo install wlink`).

```sh
cargo run --release --bin cebula_fw     # build, flash with wlink, reset
```

Logs are printed on UART0 TX (PB7) at 115200 baud, e.g. via the WCH-LinkE's serial port.

Shared code comes from [ch58x-rs-kit](https://github.com/cr1tbit/ch58x-rs-playground) and
[ch58x-hal](https://github.com/cr1tbit/ch58x-hal) (git dependencies, pinned in `Cargo.lock`).
To work on them locally, add a temporary override to `Cargo.toml`:

```toml
[patch."https://github.com/cr1tbit/ch58x-rs-playground.git"]
ch58x-rs-kit = { path = "../../ch58x-rs-playground/ch58x-rs-kit" }
```

## Progress

- [x] environment
    - [x] extract required files from https://github.com/ch32-rs/ch58x-hal to this repo
    - [x] fix TIM0 SVD description
    - [x] prepare gdb environment
    - [x] Logs via UART
- [ ] board bringup
    - [x] check if bluetooth works at all
    - [x] run PoC PWM output
    - [ ] PWM + DMA for audio output (TIM0 DOES NOT WORK WITH DMA)
    - [x] speaker (works but requires manual rework, full power bootloops the device)
    - [ ] power
        - [ ] check power consumption + sleep modes
            - [x] run (6mA @ 6mhz)
            - [x] idle (1.6mA, agrees with datasheet)
            - [x] halt (150-250uA, should be 320uA)
            - [ ] sleep
            - [ ] BT listen
            - [ ] BT ADV
            - [ ] BT connection
        - [x] check CR2032/USB power XORing (removed in rev2)
        - [x] check if LDO stable (removed in rev2)
    - [x] clocks
        - [x] check LSE (waveform visible without activating in code, probing glitches time counting without halting MCU)
        - [x] check HSE (probing glitches UART baudrate)
        - [x] check RTC (set/read works, counting works)
    - [ ] onboard flash
        - [x] chip ID + read
        - [ ] write
        - [ ] DMA
    - [ ] SD card
    - [x] bluetooth
        - [x] run BLE connection (using examples from hal)
        - [x] run bluetooth scan/ADV reception 
- [ ] firmware
    - [x] integrate embassy async
    - [ ] UART not working in debug builds (clock not settable in debug other than 6mhz)
    - [x] UART logging
    - [ ] bluetooth features
        - [x] peripheral PoC
        - [x] set RTC from BLE (almost working, but RTC cannot be set from callback context(?))
        - [ ] badge-to-badge comm
        - [ ] listen to meme adv packets
