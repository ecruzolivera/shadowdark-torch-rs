#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// PWM Sweep Test Firmware for ATtiny85 @ 1MHz
// Purpose: Test PWM functionality by sweeping from 0% to 100% brightness
// Uses SAME timer configuration as main.rs for consistency

use attiny_hal as hal;
use embedded_hal::pwm::SetDutyCycle;
use hal::simple_pwm::*;
use panic_halt as _;

// Same timer configuration as main.rs
const TIMER1_PRELOAD: u8 = 0;
const OVERFLOWS_PER_SWEEP_STEP: u16 = 19; // ~5 seconds per step (same as 5-second blink test)

static mut OVERFLOW_COUNTER: u16 = 0;
static mut SWEEP_STEP: u8 = 0; // 0-10 for 0% to 100% in 10% increments

#[avr_device::interrupt(attiny85)]
fn TIMER1_OVF() {
    unsafe {
        (*avr_device::attiny85::TC1::ptr())
            .tcnt1()
            .write(|w| w.bits(TIMER1_PRELOAD));

        OVERFLOW_COUNTER += 1;

        if OVERFLOW_COUNTER >= OVERFLOWS_PER_SWEEP_STEP {
            OVERFLOW_COUNTER = 0;
            SWEEP_STEP = (SWEEP_STEP + 1) % 11; // 0-10 (0%, 10%, 20%, ..., 100%)
        }
    }
}

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    // Same PWM configuration as main.rs
    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale64);
    let mut pwm_led = pins.pb0.into_output().into_pwm(&timer0);
    pwm_led.enable();

    // Same Timer1 configuration as main.rs
    dp.TC1.tccr1().write(|w| w.cs1().prescale_1024());
    dp.TC1.tcnt1().write(|w| unsafe { w.bits(TIMER1_PRELOAD) });
    dp.TC1.timsk().write(|w| w.toie1().set_bit());

    // Enable interrupts globally
    unsafe {
        avr_device::interrupt::enable();
    }

    loop {
        let duty_cycle = unsafe { SWEEP_STEP * 10 }; // 0%, 10%, 20%, ..., 100%

        pwm_led.set_duty_cycle_percent(duty_cycle).unwrap();
        avr_device::asm::sleep();
    }
}
