FROM rust:bookworm as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Trunk and WASM target
RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy the source code
COPY . .

# Build the application
RUN trunk build --release

# Use a lightweight nginx image for serving the static files
FROM nginx:alpine

# Copy the built files from the builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"] 