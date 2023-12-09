FROM rust:1.74.0-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --locked --release


FROM debian:bookworm-slim AS final

RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN apt-get install -y ca-certificates

RUN apt-get install -y libmariadb-dev-compat libmariadb-dev

WORKDIR /app
COPY --from=builder /app/target/release/kf5-articles-exporter /app
COPY --from=builder /app/website /app/website
COPY --from=builder /app/data /app/data
CMD ["./kf5-articles-exporter"]
