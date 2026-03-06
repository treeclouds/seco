# Dockerfile — for local development
FROM rust:1.91

# install minimal runtime tools (pg_isready)
RUN apt-get update && \
    apt-get install -y postgresql-client build-essential pkg-config libssl-dev tesseract-ocr libleptonica-dev libtesseract-dev && \
    rm -rf /var/lib/apt/lists/*

# install cargo-watch once
RUN cargo install cargo-watch || true
# install loco for migrations in dev container
RUN cargo install --force loco || true

WORKDIR /usr/src/seco

# create non-root user to match prod pattern (optional)
RUN useradd --create-home --uid 1000 devuser
USER devuser

# Expose port for dev
EXPOSE 5150

# Default command: run entrypoint which will run migrations then start dev loop
# entrypoint.sh expects to be present in mounted project directory (we mount ./:/usr/src/seco)
CMD [ "sh", "-c", "/usr/src/seco/entrypoint.sh & cargo watch -x 'run'" ]
