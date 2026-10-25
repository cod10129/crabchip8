# crabchip8

crabchip8 is a library that implements a [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
interpreter with fully pluggable I/O (see the `interpreter::providers` module).

# GIC8

GIC8 (`/gic8` directory) is a graphical frontend for crabchip8, using [`winit`]
and [`softbuffer`] for cross-platform windowing.

[`winit`]: https://crates.io/crates/winit
[`softbuffer`]: https://crates.io/crates/softbuffer
