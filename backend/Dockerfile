FROM rust:1.92-slim as builder

WORKDIR /app
COPY backend ./backend
COPY migration ./migration

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cd backend && cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/backend/target/release/backend /usr/local/bin/backend
CMD ["backend"]
