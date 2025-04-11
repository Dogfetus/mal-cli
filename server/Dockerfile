FROM rustlang/rust:nightly as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN useradd -m appuser

WORKDIR /app

COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/templates /app/templates

USER appuser

CMD ["./server"]

