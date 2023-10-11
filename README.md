# CANOpen for Rust 

## Move test to demo repo
Test cases are move to canopen-demo repository, because we need to share some testing utils for tests/ and examples/ folder together. This is helpful for end-to-end tests with RP2040 & x86 together.

It is the best way we choose, sorry for any inconvenience. If you've better ideas, please feel free to let us know. (zephyr@atomi.ai)

## How to build
To build the crate in "x86_64-unknown-linux-gnu" and "thumbv6m-none-eabi".
```shell
cargo build
cargo build --target=thumbv6m-none-eabi 
```

And you can test the project:
```shell
cargo test
```