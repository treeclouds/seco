FROM rust:1.94

RUN apt-get update && \
    apt-get install -y postgresql-client build-essential pkg-config libssl-dev tesseract-ocr libleptonica-dev libtesseract-dev && \
    rm -rf /var/lib/apt/lists/*

# Tools dev
RUN cargo install watchexec-cli loco

WORKDIR /app

# Baru copy semua sumber
COPY . .

# Hanya prefetch dependency, TANPA compile
RUN cargo fetch

EXPOSE 5150

# Hot reload, tapi tanpa extra `cargo build` manual
CMD ["watchexec", "-r", "-i", "target", "-i", ".git", \
    "-w", "src", "-w", "Cargo.toml", \
    "--", "cargo", "loco", "start", "--server-and-worker", "-b", "0.0.0.0"]
