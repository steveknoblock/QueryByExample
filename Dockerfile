FROM rust:latest
RUN cargo install cargo-watch
WORKDIR /app

# Cache dependencies separately from source code
COPY Cargo.toml ./

# Build a dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now copy real source and build
COPY src ./src
RUN cargo build --release

CMD ["./target/release/my-rust-app"]
