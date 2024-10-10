FROM rust:1.81
WORKDIR /usr/src/bot
COPY . .

# FROM ubuntu:24.10
# COPY --from=builder /usr/local/cargo/bin/bot /usr/local/bin/bot

# temporary fix for slow cross compile builds.
# this will simply compile and run the code on `docker run`
CMD ["cargo", "run", "--release"]
