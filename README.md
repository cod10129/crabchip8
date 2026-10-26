# crabchip8

crabchip8 is a library that implements a [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
interpreter with fully pluggable I/O (see the `interpreter::providers` module).

# GIC8

GIC8 (acronym for "Graphical Interpreter for Chip-8") is a graphical frontend for
crabchip8, using [`winit`] and [`softbuffer`] for cross-platform windowing.

[`winit`]: https://crates.io/crates/winit
[`softbuffer`]: https://crates.io/crates/softbuffer

```
Usage: gic8 [FLAGS] [program]

ARGS:
    <program>       The path to the CHIP-8 program to be executed.

FLAGS:
    -d, --debug     Writes debug logs to gic8.debug.log in the current working directory.
                    WARNING: Generates extremely large files (500MB+)
    -h, --help      Prints help information
    -V, --version   Prints version information
```

# License

Both crabchip8 and GIC8 are licensed under the terms of the [Mozilla Public License](MPL),
version 2.0.

[MPL]: https://www.mozilla.org/MPL/
