# explain
A quick and efficient whiteboard program primarily intended to aid with explanation and thinking

## development environment
Follow the steps [on the sdl2 crates page](https://crates.io/crates/sdl2) for your operating system, then `cargo build` should work properly.

## TODO
 - Port to the web!!! [this issue](https://github.com/rust-lang/rust/issues/85821), [and this one](https://github.com/Rust-SDL2/rust-sdl2/issues/884)
 - Use fixed point math for the zooming to avoid small errors exploding out into wrong positions (this will cause the test to pass)