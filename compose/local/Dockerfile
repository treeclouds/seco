FROM rust:1.91

# Install watchexec for auto reload
RUN cargo install watchexec-cli

# Install cargo-loco CLI (dibutuhkan untuk reload)
RUN cargo install loco

# Set workspace
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy source code
COPY . .

# Expose port default Loco (5150)
EXPOSE 5150

# Jalankan auto-reload
CMD ["watchexec", "-r", "-i", "target", "-i", ".git", "-w", "src", "-w", "Cargo.toml", "--", "cargo", "loco", "start", "--server-and-worker", "-b", "0.0.0.0"]
#CMD ["cargo", "loco", "start", "--server-and-worker", "-b", "0.0.0.0"]


