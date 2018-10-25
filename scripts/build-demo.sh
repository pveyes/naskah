## compile both entry point and worker file
## specify target because by default it uses emscripten
cd demo
cargo web build --bin main --target=wasm32-unknown-unknown --release
cargo web build --bin worker --target=wasm32-unknown-unknown --release

## copy compiled code to static dir to be served
cd ..
cp target/wasm32-unknown-unknown/release/main.js demo/static/
cp target/wasm32-unknown-unknown/release/main.wasm demo/static/

# copy worker code, remove symlink for dev env
rm -rf demo/static/worker
mkdir -p demo/static/worker
cp target/wasm32-unknown-unknown/release/worker.js demo/static/worker/
cp target/wasm32-unknown-unknown/release/worker.wasm demo/static/worker/
