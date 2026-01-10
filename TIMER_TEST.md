# Timer Test Firmware Documentation

## Purpose
Diagnose timing issues in the main shadowdark-torch-rs firmware by testing Timer1 accuracy with different prescaler configurations.

## Problem Being Diagnosed
- Main torch firmware LED turns off after ~3 minutes instead of 47+ minutes minimum
- Indicates severe timing acceleration (15.67x too fast)
- Need to verify actual Timer1 interrupt frequency vs expected frequency

## Test Firmware Variants

### 1. timer-test.rs (Prescaler 1024)
**Configuration:**
- Timer1 prescaler: 1024 (same as main firmware)
- Target: LED blink every 5 seconds
- Expected timer period: 32.77ms per overflow
- Overflow count target: 153 overflows = ~5 seconds

**Flash command:** `just test-flash`

**Expected behavior:** LED toggles every 5 seconds exactly

### 2. timer-test-8192.rs (Prescaler 8192)  
**Configuration:**
- Timer1 prescaler: 8192 (8x larger than main firmware)
- Target: LED blink every 5 seconds
- Expected timer period: 262.1ms per overflow
- Overflow count target: 19 overflows = ~5 seconds

**Flash command:** `just test-flash-8192`

**Expected behavior:** LED toggles every 5 seconds exactly

## Timer Calculations (8MHz System Clock)

### Prescaler 1024:
- Timer frequency: 8,000,000 ÷ 1024 = 7,812.5 Hz
- Timer period per tick: 1 ÷ 7,812.5 = 128 µs  
- Overflow period (256 ticks): 256 × 128 µs = **32.77ms**
- For 5 seconds: 5000ms ÷ 32.77ms = **153 overflows**

### Prescaler 8192:
- Timer frequency: 8,000,000 ÷ 8192 = 976.56 Hz
- Timer period per tick: 1 ÷ 976.56 = 1.024 ms
- Overflow period (256 ticks): 256 × 1.024 ms = **262.1ms**
- For 5 seconds: 5000ms ÷ 262.1ms = **19 overflows**

## Diagnostic Procedure

### Step 1: Flash Timer Test (Prescaler 1024)
```bash
just test-flash
```

### Step 2: Measure Actual Timing
- Use stopwatch to time LED blink intervals
- Should be exactly 5 seconds between toggles
- If different, calculate actual vs expected ratio

### Step 3: Flash Timer Test (Prescaler 8192)
```bash
just test-flash-8192  
```

### Step 4: Compare Results
- If both are wrong by same factor: clock frequency issue
- If prescaler 1024 wrong but 8192 correct: prescaler issue
- If both wrong by different factors: complex timing issue

## Expected Diagnostic Results

### Scenario A: Clock Frequency Issue
- Both tests run at wrong speed with same acceleration factor
- Indicates system clock is not 8MHz as assumed
- **Solution:** Verify fuses, measure actual clock frequency

### Scenario B: Prescaler Issue
- Prescaler 1024 test runs wrong speed
- Prescaler 8192 test runs correct speed  
- Indicates prescaler 1024 not working as expected
- **Solution:** Use different prescaler in main firmware

### Scenario C: Timer Configuration Issue
- Both tests show inconsistent timing
- Indicates Timer1 configuration problem
- **Solution:** Review Timer1 HAL usage and register settings

## Troubleshooting Guide

### If LED Blinks Too Fast:
1. **Calculate acceleration factor:** `actual_period / 5_seconds`
2. **Check prescaler effectiveness:** Compare 1024 vs 8192 results
3. **Verify clock frequency:** Use oscilloscope on PB0 during test

### If LED Blinks Too Slow:
1. **Check for overflow in counter logic**
2. **Verify interrupt is firing correctly**
3. **Check Timer1 configuration registers**

### If LED Doesn't Blink:
1. **Verify Timer1 interrupt is enabled**
2. **Check global interrupt enable**
3. **Verify Timer1 prescaler setting**
4. **Check LED connection on PB0**

## Main Firmware Fix Strategy

Based on test results:

### If Prescaler 1024 is 15.67x too fast:
```rust
// Current problematic setting
const TIME_INC: u16 = 33; // Wrong - too large

// Corrected setting
const TIME_INC: u16 = 2; // 33 ÷ 15.67 ≈ 2ms per interrupt
```

### If Need Different Prescaler:
```rust
// Change from prescaler 1024 to 8192
dp.TC1.tccr1().write(|w| w.cs1().prescale_8192());
const TIME_INC: u16 = 262; // Match actual overflow period
```

### If Clock Frequency Wrong:
```rust
// Update clock type and all calculations
type CoreClock = hal::clock::MHz1; // If actually 1MHz
// Recalculate all timer values accordingly
```

## Files Created
- `src/timer_test.rs` - Prescaler 1024 test
- `src/timer_test_8192.rs` - Prescaler 8192 test  
- `dist/timer-test.hex` - Flashable test firmware (1024)
- `dist/timer-test-8192.hex` - Flashable test firmware (8192)

## Usage
1. Flash test firmware: `just test-flash` or `just test-flash-8192`
2. Time LED blinks with stopwatch
3. Calculate timing accuracy
4. Apply findings to main torch firmware

This systematic approach will identify the exact source of the timing issue in the main firmware.