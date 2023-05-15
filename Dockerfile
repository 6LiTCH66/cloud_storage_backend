FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy your project's source code and manifest
COPY . .

# Build the application
RUN cargo build --release

# Use the official Debian slim image for the runtime environment
FROM debian:buster-slim

# Install necessary libraries
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the application binary from the build stage
COPY --from=build /usr/src/app/target/release/cloud_storage_backend /usr/local/bin/cloud_storage_backend

# Expose the port your application will run on
EXPOSE 3000

# Start the application
CMD ["cloud_storage_backend"]

