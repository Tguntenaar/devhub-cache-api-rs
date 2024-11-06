# --- Cargo Chef Stage (for caching dependencies) ---
FROM rust:1.78.0-bookworm as chef
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Dependency Caching Stage ---
FROM chef as cache
WORKDIR /app

RUN apt-get update -y && apt-get install -y \
  libssl-dev \
  ca-certificates \
  libudev-dev \
  libusb-1.0-0-dev \
  pkg-config \
  libudev-dev \
  build-essential

COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# --- Application Build Stage ---
FROM cache as builder
WORKDIR /app
RUN apt-get update -y && apt-get install -y \
  libssl-dev \
  ca-certificates \
  libudev-dev \
  libusb-1.0-0-dev \
  pkg-config \
  libudev-dev \
  build-essential
COPY . .
COPY --from=cache /app/target target
ENV SQLX_OFFLINE=true
RUN cargo build --release

# --- Final Stage with Minimal Base Image ---
FROM debian:bookworm-slim
RUN apt-get update -y && apt-get install -y \
  libssl-dev \
  ca-certificates \
  libudev-dev \
  libusb-1.0-0-dev \
  pkg-config \
  libudev-dev \
  build-essential
RUN useradd -ms /bin/bash app
USER app
WORKDIR /app
COPY --from=builder /app/target/release/devhub-cache-api .
EXPOSE 8000
CMD ["./devhub-cache-api"]
