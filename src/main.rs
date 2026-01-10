#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]

// Shadowdark RPG Torch Simulation for ATtiny85 @ 1MHz
// Power-optimized firmware with VALIDATED timing:
// - Direct timer overflow counting (verified accurate with test firmware)
// - Timer interrupts every ~262ms, 19 overflows = 5 seconds
// - Accurate Shadowdark torch timing and turn-off mechanics
// - Deep sleep between updates for minimal power consumption

mod pseudo_rand;
use embedded_hal::pwm::SetDutyCycle;
use panic_halt as _;

use attiny_hal as hal;
use hal::simple_pwm::*;

// Default Clock Source - CORRECTED AFTER TIMER TESTING
// The device has CKSEL = "0010", SUT = "10" (Internal RC Oscillator at 8 MHz)
// However, CKDIV8 appears to be enabled despite fuse reading 0x62,
// resulting in actual 1.0 MHz system clock (verified by timer test).
//
// The timers run at core clock frequency: 1 MHz

type CoreClock = hal::clock::MHz1;

// Shadowdark timing in timer overflows (verified by test firmware):
// 19 overflows = 5 seconds, so we calculate all timing from this base
const OVERFLOWS_PER_MINUTE: u32 = 19 * 12; // 19 overflows = 5 sec, * 12 = 60 sec
const T30_OVERFLOWS: u32 = OVERFLOWS_PER_MINUTE * 30; // 30 minutes = 6,840 overflows
const T45_OVERFLOWS: u32 = OVERFLOWS_PER_MINUTE * 45; // 45 minutes = 10,260 overflows
const T47_OVERFLOWS: u32 = OVERFLOWS_PER_MINUTE * 47; // 47 minutes = 10,716 overflows
const T50_OVERFLOWS: u32 = OVERFLOWS_PER_MINUTE * 50; // 50 minutes = 11,400 overflows

const TIMER1_PRELOAD: u8 = 0;

// Global overflow counter - replaces the old millisecond calculation
static mut OVERFLOW_COUNTER: u32 = 0;
static mut LAST_MINUTE_CHECK: u32 = 0;

#[avr_device::interrupt(attiny85)]
fn TIMER1_OVF() {
    unsafe {
        (*avr_device::attiny85::TC1::ptr())
            .tcnt1()
            .write(|w| w.bits(TIMER1_PRELOAD));

        // Direct overflow counting - no more TIME_INC calculations!
        OVERFLOW_COUNTER += 1;
    }
}

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale64); // ~61Hz PWM for smooth flickering

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
    let mut chance_for_turning_off = 0; //%

    loop {
        let current_overflows = unsafe { OVERFLOW_COUNTER };
        let current_minutes = current_overflows / OVERFLOWS_PER_MINUTE;

        let is_over_t47 = current_overflows >= T47_OVERFLOWS;
        let delta = rng.random_between(-40, 40);

        // decide if we turn off the torch
        // only check once per minute after 47 minutes
        // the chance of turning off increases each minute
        // by 1% (so after 53 minutes it is 7%)
        // once it is off, it stays off
        let off = unsafe {
            if is_over_t47 && current_minutes != LAST_MINUTE_CHECK {
                let maybe_off = rng.random_between(1, 100);
                chance_for_turning_off += 1;
                LAST_MINUTE_CHECK = current_minutes;
                maybe_off < chance_for_turning_off
            } else {
                false
            }
        };

        let duty_cycle = flick_torch_by_overflows(current_overflows, delta);

        pwm_led.set_duty_cycle_percent(duty_cycle).unwrap();

        avr_device::asm::sleep();
        // Timer1 wakes us every ~262ms, direct overflow counting provides accurate timing

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

fn flick_torch_by_overflows(overflows: u32, delta: i8) -> u8 {
    // set the baseline brightness based on elapsed time (in overflows)
    let duty_cycle: u8 = match overflows {
        0..T30_OVERFLOWS => 95,             // 0-30 minutes: 95% bright
        T30_OVERFLOWS..T45_OVERFLOWS => 70, // 30-45 minutes: 70% bright
        T45_OVERFLOWS..T50_OVERFLOWS => 40, // 45-50 minutes: 40% bright
        _ => 20,                            // 50+ minutes: 20% bright
    };

    // adding the flickering effect
    let duty_cycle = duty_cycle.saturating_add_signed(delta);

    if duty_cycle > 99 {
        99
    } else if duty_cycle > 20 && overflows > T50_OVERFLOWS {
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
