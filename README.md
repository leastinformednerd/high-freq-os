# OS
This is my attempt at an x86_64 OS.
It's called 'high-freq-os' because initially I conceived of it as an OS designed to address some of
the issues that high frequency trading companies have with cache / io / etc.

However I have made big leaps and bounds in the field of ideas that sound of neat and will be doing
other stuff with this (if I get that far).

Currently there's a somewhat usable text rendering system.
It's good enough to display debug information enough of the time.
I will write a better one later but I want to do other stuff


### Building
To build the disk image you can currently run `cargo xtask build`

> `cargo xtask test` will build the image and open it QEMU with gdb hooked up.
The gdb startup script assumes alacritty
