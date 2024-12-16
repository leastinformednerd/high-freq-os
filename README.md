# OS
This is my attempt at an x86_64 OS.
Currently I'm trying to get debug graphics working using the embedded-graphics crate. It's somewhat inexplicably not working

### Building
To build the disk image you can currently run `cargo xtask build`

> `cargo xtask test` will build the image and open it QEMU with gdb hooked up.
The gdb startup script assumes alacritty
