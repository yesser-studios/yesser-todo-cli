FROM rust:1.100.0-slim as build
RUN rustup target add x86_64-unknown-linux-musl && \
	apt update && \
	apt install -y musl-tools musl-dev openssl libssl-dev && \
	update-ca-certificates

COPY . .
RUN cargo build --package yesser-todo-server --target x86_64-unknown-linux-musl --release

FROM alpine:3.24.4

COPY --from=build ./target/x86_64-unknown-linux-musl/release/yesser-todo-server /app/yesser-todo-server
WORKDIR /app

EXPOSE 6982
ENTRYPOINT ["yesser-todo-server"]
