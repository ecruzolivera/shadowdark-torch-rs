#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer0Pwm};
use core::sync::atomic::{AtomicU8, Ordering};
use embedded_hal::pwm::SetDutyCycle;
use panic_halt as _;
use ufmt::uwriteln;

static DUTY_CYCLE: AtomicU8 = AtomicU8::new(90);
const TIMER1_PRELOAD: u16 = 49911; // Preload for 1s overflow with prescaler 1024

#[avr_device::interrupt(atmega2560)]
fn TIMER1_OVF() {
    let current = DUTY_CYCLE.load(Ordering::SeqCst);
    DUTY_CYCLE.store(current / 2, Ordering::SeqCst);
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

    loop {
        let duty = DUTY_CYCLE.load(Ordering::Relaxed);
        led.set_duty_cycle_percent(duty).unwrap();
        uwriteln!(&mut serial, "Duty: {}%\r", duty).unwrap();
        // Actually sleep until next interrupt (Timer1 OVF)
        avr_device::asm::sleep();
    }
}
