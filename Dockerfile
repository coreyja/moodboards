FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
RUN rustc --version; cargo --version; rustup --version
RUN cargo install -f wasm-bindgen-cli

# RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 && \
#   chmod +x tailwindcss-linux-x64 && \
#   mv tailwindcss-linux-x64 tailwindcss

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

# COPY tailwind.config.js .
# RUN ./tailwindcss -i server/src/styles/tailwind.css -o target/tailwind.css

RUN rustup target add wasm32-unknown-unknown
RUN cargo build --target wasm32-unknown-unknown --release -p frontend
RUN wasm-bindgen target/wasm32-unknown-unknown/release/frontend.wasm --out-dir frontend/out --target web

RUN SQLX_OFFLINE=true cargo build --release --locked --bin server

# Start building the final image
FROM debian:stable-slim as final
WORKDIR /app

COPY --from=builder /app/target/release/server .

EXPOSE 3000

ENTRYPOINT ["./server"]
