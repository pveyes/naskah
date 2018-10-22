# naskah-demo

This demo is built in Rust using [Yew](https://github.com/DenisKolodin/yew) framework.

## Overview
It consists of two entry point inside `src/bin/`

 - `main.rs` App entry point, component initialization and initial render
 - `worker.rs` Compiler run inside web worker

Most of the time, we only need to watch & rebuild `main.rs`. But for any worker changes, we need to rebuild manually (see setup below).

## Setup

Run this inside `demo` directory

```sh
cargo install cargo-web
# build worker script
cargo web build --bin worker --target=wasm32-unknown-unknown --release
# run app & watch for changes
cargo web start --bin main --target=wasm32-unknown-unknown --release
# visit [::1]:8000
open http://localhost:8000
```
