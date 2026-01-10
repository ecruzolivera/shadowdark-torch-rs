#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// Simple Overflow Counter Test with Mixed PWM/Digital Output
// Every 38 overflows (~10 seconds), cycle through: PWM 100% -> PWM 50% -> Digital OFF

use attiny_hal as hal;
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;
use hal::simple_pwm::*;
use panic_halt as _;

const TIMER1_PRELOAD: u8 = 0;
const OVERFLOWS_PER_CYCLE: u16 = 38; // ~10 seconds

static mut OVERFLOW_COUNTER: u16 = 0;
static mut LED_STATE: u8 = 0; // 0=100%, 1=50%, 2=OFF

#[avr_device::interrupt(attiny85)]
fn TIMER1_OVF() {
    unsafe {
        (*avr_device::attiny85::TC1::ptr())
            .tcnt1()
            .write(|w| w.bits(TIMER1_PRELOAD));

        OVERFLOW_COUNTER += 1;

        if OVERFLOW_COUNTER >= OVERFLOWS_PER_CYCLE {
            OVERFLOW_COUNTER = 0;
            LED_STATE = (LED_STATE + 1) % 3;
        }
    }
}

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale64); // ~61Hz PWM
    let mut pwm_led = pins.pb0.into_output().into_pwm(&timer0);
    let mut digital_led = pins.pb0.into_output(); // For true OFF state

    // Configure Timer1 in normal mode with prescaler 1024
    dp.TC1.tccr1().write(|w| w.cs1().prescale_1024());
    dp.TC1.tcnt1().write(|w| unsafe { w.bits(TIMER1_PRELOAD) });
    dp.TC1.timsk().write(|w| w.toie1().set_bit());

    // Enable interrupts globally
    unsafe {
        avr_device::interrupt::enable();
    }

    loop {
        unsafe {
            match LED_STATE {
                0 => {
                    // 100% PWM
                    pwm_led.enable();
                    pwm_led.set_duty_cycle_percent(100).unwrap();
                }
                1 => {
                    // 50% PWM
                    pwm_led.enable();
                    pwm_led.set_duty_cycle_percent(50).unwrap();
                }
                _ => {
                    // Digital OFF (disable PWM, set pin low)
                    pwm_led.disable();
                    digital_led.set_low().ok();
                }
            }
        }
        avr_device::asm::sleep();
    }
}
