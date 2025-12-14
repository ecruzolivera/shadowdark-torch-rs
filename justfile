# Variables
binary_name := "shadowdark-torch-rs"
dist_dir := "dist"
hex_file := dist_dir + "/" + binary_name + ".hex"

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
