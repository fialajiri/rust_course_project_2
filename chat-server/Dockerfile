FROM rust:latest

WORKDIR /app

# Copy the entire workspace
COPY . .

RUN cargo install diesel_cli --no-default-features --features postgres

# Build and run the server directly (removed cargo-watch for simplicity)
CMD ["cargo", "run", "--bin", "chat-server"]