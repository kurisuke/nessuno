# nessuno

Yet another NES emulator written in Rust.

Dependencies:
- [winit](https://github.com/rust-windowing/winit/) for windowing
- [pixels](https://github.com/parasyte/pixels/) for video framebuffer
- [cpal](https://github.com/RustAudio/cpal) for audio output
- [bdf-parser](https://github.com/embedded-graphics/bdf) for the UI font

Using [Cozette](https://github.com/slavfox/Cozette/) bitmap font for UI text.

## Feature support

- NTSC only
- Audio: all channels except DMC
- Mappers: 000, 001, 002, 003, 004, 007
- Input: keyboard, fixed mapping, 1 controller only

## Build

Non-Rust build dependencies (Debian/Ubuntu):

```
sudo apt install libasound2-dev libudev-dev
```

Build & run:

```
cargo run --release --bin nessuno romname.nes
```
