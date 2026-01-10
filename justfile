# Variables
binary_name := "shadowdark-torch-rs"
test_binary_name := "test-blink-5secs"
dist_dir := "dist"
hex_file := dist_dir + "/" + binary_name + ".hex"
test_hex_file := dist_dir + "/" + test_binary_name + ".hex"

# Default recipe
default:  release

# Create dist directory if it doesn't exist
create-dist:
    @mkdir -p {{dist_dir}}

# Build release and generate hex in dist directory
release:  create-dist
    cargo build --release
    cargo objcopy --release --bin {{binary_name}} -- -O ihex {{hex_file}}
    @echo "✅ Generated {{hex_file}}"

# Flash to ATtiny85
flash: release
    avrdude -p attiny85 -c usbtiny -U flash:w:{{hex_file}}:i

# Flash with specific programmer (adjust as needed)
flash-usbasp: release
    avrdude -p attiny85 -c usbasp -U flash:w:{{hex_file}}:i

# Show memory usage
size: release
    @echo "📊 Memory usage for {{binary_name}}:"
    cargo objdump --release --bin {{binary_name}} -- --section-headers

# Show detailed disassembly
disasm: release
    cargo objdump --release --bin {{binary_name}} -- --disassemble --no-show-raw-insn

# Clean everything
clean: 
    cargo clean
    rm -rf {{dist_dir}}
    @echo "🧹 Cleaned build artifacts and dist directory"

# List available recipes
list: 
    @just --list

# Build and show size in one command
build-info:  release size

# Development build (no hex generation)
dev:
    cargo build

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run clippy lints
lint:
    cargo clippy

# Full development workflow
dev-flow:  fmt lint check dev

# Release workflow with size info
release-flow: fmt lint release size

# Timer Test Firmware Commands
test-build: create-dist
    cargo build --release --bin {{test_binary_name}}
    cargo objcopy --release --bin {{test_binary_name}} -- -O ihex {{test_hex_file}}
    @echo "✅ Generated {{test_hex_file}}"

# Flash timer test firmware
test-flash: test-build
    avrdude -p attiny85 -c usbtiny -U flash:w:{{test_hex_file}}:i
    @echo "🔥 Flashed 5-second blink test firmware - LED should blink every 5 seconds"

# Flash timer test with USBasp
test-flash-usbasp: test-build
    avrdude -p attiny85 -c usbasp -U flash:w:{{test_hex_file}}:i
    @echo "🔥 Flashed 5-second blink test firmware - LED should blink every 5 seconds"

# Timer Test with Prescaler 8192
test-build-8192: create-dist
    cargo build --release --bin timer-test-8192
    cargo objcopy --release --bin timer-test-8192 -- -O ihex {{dist_dir}}/timer-test-8192.hex
    @echo "✅ Generated timer-test-8192.hex (prescaler 8192)"

test-flash-8192: test-build-8192
    avrdude -p attiny85 -c usbtiny -U flash:w:{{dist_dir}}/timer-test-8192.hex:i
    @echo "🔥 Flashed timer test (8192) - LED should blink every 5 seconds"

# PWM Test Firmware (tests PWM functionality and timing)
pwm-test-build: create-dist
    cargo build --release --bin pwm-test
    cargo objcopy --release --bin pwm-test -- -O ihex {{dist_dir}}/pwm-test.hex
    @echo "✅ Generated pwm-test.hex"

pwm-test-flash: pwm-test-build
    avrdude -p attiny85 -c usbtiny -U flash:w:{{dist_dir}}/pwm-test.hex:i
    @echo "🔥 Flashed PWM test firmware - LED should cycle: 100% (10s) -> 50% (10s) -> 0% (10s) -> repeat"

# Overflow Counter Test (direct overflow counting, no TIME_INC calculation)
overflow-test-build: create-dist
    cargo build --release --bin overflow-test
    cargo objcopy --release --bin overflow-test -- -O ihex {{dist_dir}}/overflow-test.hex
    @echo "✅ Generated overflow-test.hex"

overflow-test-flash: overflow-test-build
    avrdude -p attiny85 -c usbtiny -U flash:w:{{dist_dir}}/overflow-test.hex:i
    @echo "🔥 Flashed overflow test - LED should cycle every ~10s: 100% -> 50% -> 0% -> repeat (direct overflow counting)"
