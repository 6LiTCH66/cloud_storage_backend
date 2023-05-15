FROM rust:latest as builder

WORKDIR /usr/src/cloud-storage-backend-heroku

COPY . .

RUN cargo install --path .

FROM debian:buster-slim

RUN apt-get update && apt-get install -y openssl

COPY --from=builder /usr/local/cargo/bin/cloud-storage-backend-heroku /usr/local/bin/cloud-storage-backend-heroku

CMD ["cloud-storage-backend-heroku"]
