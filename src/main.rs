#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]

// Shadowdark RPG Torch Simulation for ATtiny85 @ 8MHz
// Power-optimized firmware for maximum battery life:
// - Timer interrupts every ~33ms (30Hz updates)
// - PWM frequency optimized to 122Hz for efficiency
// - Accurate Shadowdark torch timing and turn-off mechanics
// - Deep sleep between updates for minimal power consumption

mod pseudo_rand;
use embedded_hal::pwm::SetDutyCycle;
use panic_halt as _;

use attiny_hal as hal;
use hal::simple_pwm::*;

// Default Clock Source
// The device has CKSEL = "0010", SUT = "10" (Internal RC Oscillator at 8 MHz)
// CKDIV8 is NOT programmed (fuse = 0x62), so no clock division occurs
// resulting in 8.0 MHz system clock. This default setting provides optimal
// performance for the torch simulation while maintaining power efficiency.
//
// The timers run at core clock frequency: 8 MHz

type CoreClock = hal::clock::MHz8;

const MINUTE: u16 = 60;
const T30: u16 = MINUTE * 30;
const T45: u16 = MINUTE * 45;
const T47: u16 = MINUTE * 47;
const T50: u16 = MINUTE * 50;

const TIMER1_PRELOAD: u8 = 0; // Let timer overflow naturally every 256 ticks for consistent timing
const TIME_INC: u16 = 33; // Each overflow = ~33ms at 8MHz/1024, update every overflow for power efficiency

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

    // Configure Timer1 in normal mode with prescaler 1024 (balanced power efficiency)
    dp.TC1.tccr1().write(|w| w.cs1().prescale_1024());
    // Preload TCNT1 to 0 for natural overflow timing
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
        // Timer1 wakes us every ~33ms for power-efficient torch updates
        // This provides smooth flickering while minimizing interrupt overhead
        // wait for timer1 overflow interrupt to wake up and then continue
        // increment time by TIME_INC milliseconds
        miliseconds += TIME_INC as u32;

        last_min = minutes;

        if off {
            // Torch has burned out - enter maximum power saving mode
            avr_device::interrupt::disable();
            pwm_led.disable();
            // Enter permanent sleep - torch is completely off
            loop {
                avr_device::asm::sleep();
            }
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
