# Shadowdark torch rs

[![Latest Release](https://img.shields.io/github/v/release/ecruzolivera/shadowdark-torch-rs)](https://github.com/ecruzolivera/shadowdark-torch-rs/releases/latest)
[![Download Firmware](https://img.shields.io/badge/download-firmware.hex-blue)](https://github.com/ecruzolivera/shadowdark-torch-rs/releases/latest/download/shadowdark-torch-rs.hex)

Rust project for the _attiny85_.

## Hardware Requirements

- **Microcontroller:** ATtiny85 (8KB flash, 512B RAM)
- **Programmer:** USBtiny or USBasp ISP programmer

## Documentation

- https://learn.sparkfun.com/tutorials/tiny-avr-programmer-hookup-guide/all
- https://www.sparkfun.com/tiny-avr-programmer.html
- https://www.thearcanelibrary.com/products/shadowdark-rpg-quickstart-set-pdf

## Build Instructions

1. Install prerequisites Arch Linux: `sudo pacman -S avr-binutils avr-libc avrdude avr-gdb avr-gcc just`
2. Run `just flash` to flash the firmware using the avr tiny programmer (usbtiny)

## USB Programmer Setup (Arch Linux)

Enable flashing without sudo:

1. Create udev rules:

   ```bash
   sudo tee /etc/udev/rules.d/99-avr-programmers.rules << 'EOF'
   # USBtiny programmer
   SUBSYSTEM=="usb", ATTRS{idVendor}=="1781", ATTRS{idProduct}=="0c9f", GROUP="wheel", MODE="0664"
   # USBasp programmer
   SUBSYSTEM=="usb", ATTRS{idVendor}=="16c0", ATTRS{idProduct}=="05dc", GROUP="wheel", MODE="0664"
   EOF
   ```

2. Ensure you're in the wheel group:

   ```bash
   sudo usermod -a -G wheel $USER
   ```

3. Apply changes:

   ```bash
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   ```

4. Reconnect your programmer and test: `just flash`

**Troubleshooting:** If flashing fails with "Permission denied", run:

```bash
just release
sudo avrdude -p attiny85 -c usbtiny -U flash:w:dist/shadowdark-torch-rs.hex:i
```
