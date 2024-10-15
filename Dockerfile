FROM rust:1.81 AS builder
WORKDIR /usr/src/bot
COPY . .

RUN cargo update
RUN cargo install --path .

FROM ubuntu:24.10
COPY --from=builder /usr/local/cargo/bin/bot /usr/local/bin/bot

CMD ["bot"]
