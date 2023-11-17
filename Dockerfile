FROM rust:slim-bullseye as builder
WORKDIR /usr/src/currency-bot
COPY . .
RUN cargo install --path .
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/currency-bot /usr/local/bin/
COPY .env .
COPY daily_messages/ .

CMD ["currency-bot"]