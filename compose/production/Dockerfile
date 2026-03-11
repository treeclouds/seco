# builder: musl static build with ekidd/rust-musl-builder
FROM ekidd/rust-musl-builder:latest AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y postgresql-client build-essential pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# copy cargo manifest first to cache deps
COPY Cargo.toml Cargo.lock ./
# create dummy src to cache cargo fetch/build layers
RUN mkdir src && echo "fn main() { println!(\"hello\"); }" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# copy full source and build static binary
COPY . .
# if your crate uses openssl or other native libs you may need additional steps here
RUN cargo build --release --target x86_64-unknown-linux-musl

# runtime: very small image (alpine or scratch). Use alpine if you need /bin/sh for debug.
FROM alpine:3.18 AS runtime
# create non-root user
RUN addgroup -S app && adduser -S app -G app
WORKDIR /app

# copy statically linked binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/seco-cli /usr/local/bin/seco-cli
RUN chmod +x /usr/local/bin/seco-cli
USER app

EXPOSE 5150
# set minimal envs if needed
ENV RUST_LOG=info

# recommended to use exec form
ENTRYPOINT ["/usr/local/bin/seco-cli"]
CMD ["start","--server-and-worker","-b","0.0.0.0"]
