# Variables
binary_name := "shadowdark-torch-rs"
blink_test := "test-blink-5secs"
pwm_sweep_test := "test-pwm-sweep"
dist_dir := "dist"
main_hex := dist_dir + "/" + binary_name + ".hex"
blink_hex := dist_dir + "/" + blink_test + ".hex"
sweep_hex := dist_dir + "/" + pwm_sweep_test + ".hex"

# Default recipe
default: release

# Create dist directory if it doesn't exist
create-dist:
    @mkdir -p {{dist_dir}}

# Build main torch firmware
release: create-dist
    cargo build --release
    cargo objcopy --release --bin {{binary_name}} -- -O ihex {{main_hex}}
    @echo "✅ Generated {{main_hex}}"

# Flash main torch firmware
flash: release
    avrdude -p attiny85 -c usbtiny -U flash:w:{{main_hex}}:i
    @echo "🔥 Flashed Shadowdark torch firmware"

# Show memory usage
size: release
    @echo "📊 Memory usage for {{binary_name}}:"
    cargo objdump --release --bin {{binary_name}} -- --section-headers

# Clean everything
clean: 
    cargo clean
    rm -rf {{dist_dir}}
    @echo "🧹 Cleaned build artifacts and dist directory"

# List available recipes
list: 
    @just --list

# Development commands
dev:
    cargo build

check:
    cargo check

fmt:
    cargo fmt

lint:
    cargo clippy

# 5-Second Blink Test Commands
blink-build: create-dist
    cargo build --release --bin {{blink_test}}
    cargo objcopy --release --bin {{blink_test}} -- -O ihex {{blink_hex}}
    @echo "✅ Generated {{blink_hex}}"

blink-flash: blink-build
    avrdude -p attiny85 -c usbtiny -U flash:w:{{blink_hex}}:i
    @echo "🔥 Flashed 5-second blink test - LED should blink every 5 seconds"

# PWM Sweep Test Commands
sweep-build: create-dist
    cargo build --release --bin {{pwm_sweep_test}}
    cargo objcopy --release --bin {{pwm_sweep_test}} -- -O ihex {{sweep_hex}}
    @echo "✅ Generated {{sweep_hex}}"

sweep-flash: sweep-build
    avrdude -p attiny85 -c usbtiny -U flash:w:{{sweep_hex}}:i
    @echo "🔥 Flashed PWM sweep test - LED sweeps 0%-100% every 5 seconds per step"