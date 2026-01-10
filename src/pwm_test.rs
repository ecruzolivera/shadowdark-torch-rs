#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// PWM Test Firmware for ATtiny85 @ 1MHz
// Purpose: Test PWM functionality and timing accuracy
// Changes LED brightness every 10 seconds: 100% -> 50% -> 0% -> repeat

use attiny_hal as hal;
use embedded_hal::pwm::SetDutyCycle;
use hal::simple_pwm::*;
use panic_halt as _;

const TIMER1_PRELOAD: u8 = 0;
const TIME_INC: u16 = 262; // Each Timer1 overflow = 262ms at 1MHz/1024 (verified by 5-second timer test)

#[avr_device::interrupt(attiny85)]
fn TIMER1_OVF() {
    unsafe {
        (*avr_device::attiny85::TC1::ptr())
            .tcnt1()
            .write(|w| w.bits(TIMER1_PRELOAD));
    }
}

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale1024);
    let mut pwm_led = pins.pb0.into_output().into_pwm(&timer0);
    pwm_led.enable();

    // Configure Timer1 in normal mode with prescaler 1024
    dp.TC1.tccr1().write(|w| w.cs1().prescale_1024());
    dp.TC1.tcnt1().write(|w| unsafe { w.bits(TIMER1_PRELOAD) });
    dp.TC1.timsk().write(|w| w.toie1().set_bit());

    // Enable interrupts globally
    unsafe {
        avr_device::interrupt::enable();
    }

    let mut miliseconds: u32 = 0;

    loop {
        let seconds = (miliseconds / 1000) as u16;

        // Change brightness every 10 seconds in a cycle: 100% -> 50% -> 0% -> repeat
        let duty_cycle = match (seconds / 10) % 3 {
            0 => 100, // 0-9 seconds: full brightness
            1 => 50,  // 10-19 seconds: half brightness
            _ => 0,   // 20-29 seconds: off, then repeat
        };

        pwm_led.set_duty_cycle_percent(duty_cycle).unwrap();

        avr_device::asm::sleep();
        miliseconds += TIME_INC as u32;
    }
}
