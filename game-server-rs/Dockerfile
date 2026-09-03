FROM rust:1.87-slim AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/openao-server /usr/local/bin/openao-server
COPY --from=builder /app/data /app/data

WORKDIR /app
ENV BIND_ADDR=0.0.0.0:7666
ENV HTTP_ADDR=0.0.0.0:7667
ENV DATABASE_URL=sqlite:/data/openao.db
VOLUME /data

EXPOSE 7666 7667

CMD ["openao-server"]
