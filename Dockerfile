FROM rust:1.90.0-slim as build
RUN rustup target add x86_64-unknown-linux-musl && \
	apt update && \
	apt install -y musl-tools musl-dev openssl libssl-dev && \
	update-ca-certificates

COPY ./server .
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM rust:1.90-alpine3.22
COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
COPY --from=build ./target/x86_64-unknown-linux-musl/release/yesser-todo-server /app/yesser-todo-server
ENTRYPOINT ["./app/yesser-todo-server"]
