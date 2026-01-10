#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// Simple Overflow Counter Test - Count timer overflows directly
// Every 38 overflows (should be ~10 seconds), toggle LED state
// This eliminates the TIME_INC calculation completely

use attiny_hal as hal;
use embedded_hal::pwm::SetDutyCycle;
use hal::simple_pwm::*;
use panic_halt as _;

const TIMER1_PRELOAD: u8 = 0;
const OVERFLOWS_PER_CYCLE: u16 = 38; // ~10 seconds (19 overflows = 5 sec, so 38 = 10 sec)

static mut OVERFLOW_COUNTER: u16 = 0;
static mut LED_STATE: u8 = 0; // 0=100%, 1=50%, 2=0%

#[avr_device::interrupt(attiny85)]
fn TIMER1_OVF() {
    unsafe {
        (*avr_device::attiny85::TC1::ptr())
            .tcnt1()
            .write(|w| w.bits(TIMER1_PRELOAD));

        OVERFLOW_COUNTER += 1;

        if OVERFLOW_COUNTER >= OVERFLOWS_PER_CYCLE {
            OVERFLOW_COUNTER = 0;
            LED_STATE = (LED_STATE + 1) % 3; // Cycle through 0, 1, 2
        }
    }
}

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale1024); // Back to working prescaler
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

    loop {
        unsafe {
            match LED_STATE {
                0 => {
                    // Full brightness - 100% PWM
                    pwm_led.enable();
                    pwm_led.set_duty_cycle_percent(100).unwrap();
                }
                1 => {
                    // Half brightness - 50% PWM (will be visible as slow blinking due to low PWM freq)
                    pwm_led.enable();
                    pwm_led.set_duty_cycle_percent(50).unwrap();
                }
                _ => {
                    // Completely off - disable PWM entirely
                    pwm_led.disable();
                }
            }
        }
        avr_device::asm::sleep();
    }
}
