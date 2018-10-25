FROM rustlang/rust:nightly

WORKDIR /usr/src
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-web

COPY . .
RUN ./scripts/build-demo.sh

RUN mv ./demo/static /public
