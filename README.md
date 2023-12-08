# CANOpen for Rust 

## Move test to demo repo
Integration test cases are move to [canopen-demo](https://github.com/atomi-ai/canopen-demo) repository, because we need to share some testing utils for tests/ and examples/ folder together. This is helpful for end-to-end tests with RP2040 & x86 together. If you've better ideas, please let us know. (zephyr@atomi.ai)

## How to build
To build the crate in "x86_64-unknown-linux-gnu" and "thumbv6m-none-eabi".
```shell
cargo build --target=x86_64-unknown-linux-gnu
cargo build --target=thumbv6m-none-eabi 
```

And you can test the project:
```shell
cargo test
```
We still have some unit tests here.
