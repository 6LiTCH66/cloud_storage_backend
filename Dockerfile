# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the project files into the container
COPY . .

# Build the Rust application
RUN cargo build --release

# Create a new image without the build tools
FROM debian:buster-slim

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/cloud_storage_backend .

# Expose any necessary ports
EXPOSE 8000

# Set the entry point command for the container
CMD ["./cloud_storage_backend"]
