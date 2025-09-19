#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer0Pwm};
use embedded_hal::pwm::SetDutyCycle;
use panic_halt as _;
use ufmt::uwriteln;

mod pseudo_rand;

const MINUTE: u16 = 60;
const T30: u16 = MINUTE * 30;
const T45: u16 = MINUTE * 45;
const T50: u16 = MINUTE * 50;
const T59: u16 = MINUTE * 59;

// const TIMER1_PRELOAD: u16 = 49911; // Preload for 1s overflow with prescaler 64
// const TIMER1_PRELOAD: u16 = 57724; // Preload for 500ms overflow with prescaler 1024
const TIMER1_PRELOAD: u16 = 63973; // Preload for 100ms overflow with prescaler 1024
const TIME_INC: u16 = 100;

#[avr_device::interrupt(atmega2560)]
fn TIMER1_OVF() {
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
    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale1024);

    // Setup serial port at 57600 baud
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut led = pins.d13.into_output().into_pwm(&timer0);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let a0 = pins.a0.into_analog_input(&mut adc).into_channel();
    let seed = adc.read_blocking(&a0);

    uwriteln!(&mut serial, "Hello from Torch, seed value: {}\r", seed).unwrap();

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

    let mut rng = pseudo_rand::XorShift8::new(seed as i8);
    let mut miliseconds = 0;
    loop {
        let seconds = miliseconds / 1000;

        let is_over_t = seconds >= T59;
        let delta = rng.random_between(-50, 50);
        let off = is_over_t && delta < 0;

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
        miliseconds += TIME_INC;

        if off {
            uwriteln!(&mut serial, "Power off").unwrap();
            avr_device::interrupt::disable();
            led.disable();
            avr_device::asm::sleep();
        }
    }
}

fn flick_torch(seconds: u16, delta: i8) -> u8 {
    const T31: u16 = T30 + 1;
    const T46: u16 = T45 + 1;

    // set the baseline
    let duty_cycle: u8 = match seconds {
        0..=T30 => 95,
        T31..=T45 => 70,
        T46..=T50 => 40,
        _ => 20,
    };

    // adding the flickering effect
    let duty_cycle = duty_cycle.saturating_add_signed(delta);

    if duty_cycle > 100 {
        100
    } else if duty_cycle > 20 && seconds > T50 {
        20
    } else if duty_cycle <= 10 && seconds < T59 {
        // never allow being off until min 59
        10
    } else {
        duty_cycle
    }
}
