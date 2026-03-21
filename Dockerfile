FROM rust:1-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/lauyer /usr/local/bin/
EXPOSE 3000
CMD ["lauyer", "serve", "--port", "3000", "--host", "0.0.0.0"]
