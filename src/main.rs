#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer0Pwm};
use core::sync::atomic::{AtomicU8, Ordering};
use embedded_hal::pwm::SetDutyCycle;
use panic_halt as _;
use ufmt::uwriteln;

mod pseudo_rand;

const HOUR: u16 = 3600;
const HALF_HOUR: u16 = HOUR / 2;
const TIMER1_PRELOAD: u16 = 49911; // Preload for 1s overflow with prescaler 1024
static SECONDS: AtomicU8 = AtomicU8::new(0);

#[avr_device::interrupt(atmega2560)]
fn TIMER1_OVF() {
    let secs = SECONDS.load(Ordering::SeqCst);
    SECONDS.store(secs + 1, Ordering::SeqCst);
    unsafe {
        // SAFETY: Accessing hardware register from ISR
        (*avr_device::atmega2560::TC1::ptr())
            .tcnt1
            .write(|w| w.bits(TIMER1_PRELOAD));
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    // **DISABLE Timer0 overflow interrupt!**
    dp.TC0.timsk0.write(|w| w.toie0().clear_bit());
    // Configure CPU sleep mode: IDLE + enable sleep
    dp.CPU.smcr.write(
        |w| {
            w.sm()
                .idle() // choose IDLE mode
                .se()
                .set_bit()
        }, // set SE = sleep enable
    );
    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale64);

    // Setup serial port at 57600 baud
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut led = pins.d13.into_output().into_pwm(&timer0);

    led.enable();
    // Preload TCNT1
    dp.TC1.tcnt1.write(|w| w.bits(TIMER1_PRELOAD));
    // Configure Timer1 in normal mode with prescaler 1024
    dp.TC1.tccr1b.write(|w| w.cs1().prescale_1024());
    // Enable Timer1 overflow interrupt
    dp.TC1.timsk1.write(|w| w.toie1().set_bit());

    // Enable interrupts globally
    unsafe {
        avr_device::interrupt::enable();
    }
    let mut rng = pseudo_rand::XorShift8::new(69);
    loop {
        let seconds = SECONDS.load(Ordering::SeqCst);
        let delta = rng.random_between(-5, 5);
        let duty_cycle = flick_torch(seconds, delta);
        led.set_duty_cycle_percent(duty_cycle).unwrap();
        uwriteln!(
            &mut serial,
            "Duty: {}% Seconds: {} delta: {}\r",
            duty_cycle,
            seconds,
            delta
        )
        .unwrap();
        avr_device::asm::sleep();
        if seconds > 60 {
            avr_device::interrupt::disable();
            led.set_duty_cycle_percent(0).unwrap();
            avr_device::asm::sleep();
        }
    }
}

fn flick_torch(seconds: u8, delta: i8) -> u8 {
    let adjusted_by_time: u8 = match seconds {
        0..=30 => 90,
        31..=45 => 60,
        46..=59 => 40,
        _ => 0,
    };
    let flicked = adjusted_by_time.saturating_add_signed(delta);
    match flicked {
        v if v > 100 => 100,
        _ => flicked,
    }
}
