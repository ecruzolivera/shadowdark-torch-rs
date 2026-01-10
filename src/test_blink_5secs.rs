#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// Timer Test Firmware for ATtiny85 @ 1MHz
// Purpose: Verify timer accuracy by blinking LED every 5 seconds
// This helps diagnose timing issues in the main torch firmware
// CORRECTED: Clock is actually 1MHz, not 8MHz as originally assumed

use attiny_hal as hal;
use panic_halt as _;

// Timer configuration for 5-second LED blink
// Strategy: Use Timer1 with prescaler 1024 - CORRECTED for actual 1MHz clock
//
// ACTUAL CLOCK: 1MHz (not 8MHz as originally assumed)
// 1MHz ÷ 1024 = 976.56 Hz
// 256 ticks ÷ 976.56 Hz = 262.1ms per overflow
// For 5 seconds: 5000ms ÷ 262.1ms = 19.1 overflows needed

const TIMER1_PRELOAD: u8 = 0; // Let timer overflow naturally
const OVERFLOW_COUNT_TARGET: u16 = 19; // ~5 seconds with prescaler 1024 at 1MHz

static mut OVERFLOW_COUNTER: u16 = 0;
static mut LED_STATE: bool = false;

#[avr_device::interrupt(attiny85)]
fn TIMER1_OVF() {
    unsafe {
        // Reset timer for next overflow
        (*avr_device::attiny85::TC1::ptr())
            .tcnt1()
            .write(|w| w.bits(TIMER1_PRELOAD));

        // Count overflows for 5-second timing
        OVERFLOW_COUNTER += 1;

        if OVERFLOW_COUNTER >= OVERFLOW_COUNT_TARGET {
            OVERFLOW_COUNTER = 0;
            LED_STATE = !LED_STATE;
        }
    }
}

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    // Configure PB0 as digital output (no PWM for simpler testing)
    let mut led = pins.pb0.into_output();

    // Start with LED off
    led.set_low();

    // Configure Timer1 with prescaler 1024 (same as main firmware)
    dp.TC1.tccr1().write(|w| w.cs1().prescale_1024());

    // Initialize timer
    dp.TC1.tcnt1().write(|w| unsafe { w.bits(TIMER1_PRELOAD) });

    // Enable Timer1 overflow interrupt
    dp.TC1.timsk().write(|w| w.toie1().set_bit());

    // Enable interrupts globally
    unsafe {
        avr_device::interrupt::enable();
    }

    // Main loop: update LED state based on timer
    loop {
        unsafe {
            if LED_STATE {
                led.set_high();
            } else {
                led.set_low();
            }
        }

        // Sleep until next timer interrupt
        avr_device::asm::sleep();
    }
}
