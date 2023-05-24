# Start from the latest Rust image
FROM rust:latest

# Set the current working directory inside the docker image
WORKDIR /usr/src/app

# Copy your source code into the image
COPY . .

# Build your application for release
RUN cargo build --release

# Run the binary
CMD ["/usr/src/app/target/release/cloud_storage_backend"]
