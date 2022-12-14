# donut.live

Similar to [parrot.live](https://github.com/hugomd/parrot.live), but here it is in donut form.

## Usage
+ Build with `cargo` from [rustup](https://rustup.rs/) $\to$ `cargo build --release`
+ Run the executable from wherever `./donut-live`
+ That's it. You can also specify an address like so: `./donut-live 127.0.0.1:80`
+ If you don't specify an address, it will automatically default to *localhost:80*

## Description
This only works if the terminal being used supports [ANSI Escape Sequences](https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797)

## Other
- The `donuts` function within `donut.rs` is somewhat of a simple transpilation of the original [donut.c](https://www.a1k0n.net/2011/07/20/donut-math.html) script. The donuts that are generated here are completely identical to those generated by the original [donut.c](https://www.a1k0n.net/2011/07/20/donut-math.html)
- Automatically rate limits each stream to one session at a time

## Purpose
- *donut*