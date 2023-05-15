FROM rust:latest as builder

WORKDIR /usr/src/cloud_storage_backend

COPY . .

RUN cargo install --path .

FROM debian:buster-slim

RUN apt-get update && apt-get install -y openssl

COPY --from=builder /usr/local/cargo/bin/cloud_storage_backend /usr/local/bin/cloud_storage_backend

CMD ["cloud_storage_backend"]
