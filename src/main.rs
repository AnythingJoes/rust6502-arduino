#![no_std]
#![no_main]
#![macro_use]
#![feature(abi_avr_interrupt)]

use panic_halt as _;

use arduino_hal::prelude::*;
use core::sync::atomic::{AtomicBool, Ordering};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[avr_device::interrupt(atmega2560)]
fn INT0() {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let address_pins = [
        // Byte 2
        pins.d52.into_floating_input().downgrade(),
        pins.d50.into_floating_input().downgrade(),
        pins.d48.into_floating_input().downgrade(),
        pins.d46.into_floating_input().downgrade(),
        pins.d44.into_floating_input().downgrade(),
        pins.d42.into_floating_input().downgrade(),
        pins.d40.into_floating_input().downgrade(),
        pins.d38.into_floating_input().downgrade(),
        // byte 1
        pins.d36.into_floating_input().downgrade(),
        pins.d34.into_floating_input().downgrade(),
        pins.d32.into_floating_input().downgrade(),
        pins.d30.into_floating_input().downgrade(),
        pins.d28.into_floating_input().downgrade(),
        pins.d26.into_floating_input().downgrade(),
        pins.d24.into_floating_input().downgrade(),
        pins.d22.into_floating_input().downgrade(),
    ];

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    ufmt::uwriteln!(&mut serial, "Serial setup").void_unwrap();


    // Rising edge
    dp.EXINT.eicra.modify(|_, w| w.isc0().bits(0x02));
    // Enables INT0
    dp.EXINT.eimsk.modify(|_, w| w.int().bits(0x01));

    unsafe { avr_device::interrupt::enable(); }

    loop {
        if INTERRUPTED.load(Ordering::SeqCst) {
            INTERRUPTED.store(false, Ordering::SeqCst);
            let mut address: u16 = 0;

            for pin in address_pins.iter() {
                address <<= 1;
                let bit = pin.is_high() as u16;
                ufmt::uwrite!(&mut serial, "{}", bit).void_unwrap();
                address += bit
            }
            ufmt::uwrite!(&mut serial, " ").void_unwrap();
            let buffer = format_address(address);
            for character in buffer.iter().rev() {
                ufmt::uwrite!(&mut serial, "{}", character).void_unwrap();
            }
            ufmt::uwrite!(&mut serial, "\n").void_unwrap();
        }
        avr_device::asm::sleep();
    }
}

fn format_address(address: u16) -> [char; 4] {
    let mut buffer = ['0'; 4];
    let mut address = address;
    let mut counter = 0;
    while address != 0 {
        let remainder = address % 16;
        if remainder < 10 {
            buffer[counter] = ('0' as u8 + remainder as u8) as char;
        } else {
            buffer[counter] = ('7' as u8 + remainder as u8) as char;
        }
        address /= 16;
        counter += 1;
    }
    buffer
}
