rustup default nightly
cargo install wasm-pack
cargo build
cd demo
wasm-pack build --target web --out-name wasm --out-dir ./static/assets