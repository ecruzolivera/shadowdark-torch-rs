#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]

mod pseudo_rand;
use embedded_hal::pwm::SetDutyCycle;
use panic_halt as _;

use attiny_hal as hal;
use hal::simple_pwm::*;

// Default Clock Source
// The device is shipped with CKSEL = “0010”, SUT = “10”, and CKDIV8 programmed. The default clock source setting
// is therefore the Internal RC Oscillator running at 8 MHz with longest start-up time and an initial system clock
// prescaling of 8, resulting in 1.0 MHz system clock. This default setting ensures that all users can make their
// desired clock source setting using an In-System or High-voltage Programme
//
// the timers are set by default to core clock and therefore at 1 MHz

type CoreClock = hal::clock::MHz8;

const MINUTE: u16 = 60;
const T30: u16 = MINUTE * 30;
const T45: u16 = MINUTE * 45;
const T47: u16 = MINUTE * 47;
const T50: u16 = MINUTE * 50;

const TIMER1_PRELOAD: u8 = 158; // Preload for 100ms overflow with prescaler 1024 and 1 MHz clock
const TIME_INC: u16 = 100;

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

    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale64);

    let mut pwm_led = pins.pb0.into_output().into_pwm(&timer0);
    pwm_led.enable();

    // Configure Timer1 in normal mode with prescaler 1024
    dp.TC1.tccr1().write(|w| w.cs1().prescale_1024());
    // Preload TCNT1
    dp.TC1.tcnt1().write(|w| unsafe { w.bits(TIMER1_PRELOAD) });
    // Enable Timer1 overflow interrupt
    dp.TC1.timsk().write(|w| w.toie1().set_bit());

    let mut adc = hal::adc::Adc::<CoreClock>::new(dp.ADC, Default::default());
    let pb2_adc1 = pins.pb2.into_analog_input(&mut adc).into_channel();
    let seed = adc.read_blocking(&pb2_adc1);

    // Enable interrupts globally
    unsafe {
        avr_device::interrupt::enable();
    }

    let mut rng = pseudo_rand::XorShift8::new(seed as i8);
    let mut miliseconds: u32 = 0;
    let mut chance_for_turning_off = 0; //%
    let mut last_min = 0;
    loop {
        let seconds = (miliseconds / 1000) as u16;
        let minutes = seconds / 60;

        let is_over_t = seconds >= T47;
        let delta = rng.random_between(-40, 40);

        // decide if we turn off the torch
        // only check once per minute after 47 minutes
        // the chance of turning off increases each minute
        // by 1% (so after 53 minutes it is 7%)
        // once it is off, it stays off
        let off = if is_over_t && minutes != last_min {
            let maybe_off = rng.random_between(1, 100);
            chance_for_turning_off += 1;
            maybe_off < chance_for_turning_off
        } else {
            false
        };

        let duty_cycle = flick_torch(seconds, delta);

        pwm_led.set_duty_cycle_percent(duty_cycle).unwrap();

        avr_device::asm::sleep();
        // wait for timer1 overflow interrupt to wake up and then continue
        // increment time by TIME_INC milliseconds
        miliseconds += TIME_INC as u32;

        last_min = minutes;

        if off {
            avr_device::interrupt::disable();
            pwm_led.disable();
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

    if duty_cycle > 99 {
        99
    } else if duty_cycle > 20 && seconds > T50 {
        20
    } else if duty_cycle < 5 {
        // never allow being off due flickering
        5
    } else {
        duty_cycle
    }
}

//
// fn jump_to_reset_vector() -> ! {
//     unsafe {
//         core::arch::asm!("jmp 0x0000", options(noreturn));
//     }
// }
//
//
//
//     // **DISABLE Timer0 overflow interrupt!**
//     dp.TC0.timsk().write(|w| w.toie0().clear_bit());
//     // Configure CPU sleep mode: IDLE + enable sleep
//     // dp.CPU.mcucr.modify(|_, w| w.se().set_bit());
//     //
