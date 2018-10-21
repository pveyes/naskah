## specify target because by default it uses emscripten
cd demo
cargo web build --target=wasm32-unknown-unknown --release

## copy compiled code to static dir to be served
cd ..
cp target/wasm32-unknown-unknown/release/naskah-demo.js demo/src/static/
cp target/wasm32-unknown-unknown/release/naskah-demo.wasm demo/src/static/