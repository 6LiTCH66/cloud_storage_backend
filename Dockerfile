# Use the Rust official Docker image as the base
FROM rust:latest as builder

# Set the working directory
WORKDIR /app

# Copy the project files
COPY . .

# Build the application
RUN cargo build --release

# Start a new Docker image
FROM debian:buster-slim

ENV MONGODB_URI="mongodb+srv://ilja200303:ilja2003@cluster0.ahmrjgq.mongodb.net/?retryWrites=true&w=majority"
ENV JWT_TOKEN="eyJhbGciOiJIUzI1NiJ9.eyJSb2xlIjoiQWRtaW4iLCJJc3N1ZXIiOiJJc3N1ZXIiLCJVc2VybmFtZSI6IkphdmFJblVzZSIsImV4cCI6MTY4MzYxMDk3NywiaWF0IjoxNjgzNjEwOTc3fQ.fDF6dk82IXa1BjlYfs8uPR9anvvSlxnlU9UiqiBYCxQ"

# Copy the built binary from the previous stage
COPY --from=builder /app/target/release/cloud_storage_backend /usr/local/bin/cloud_storage_backend

# Expose the port on which your application listens
EXPOSE 3000

# Set the command to run your application
CMD ["cloud_storage_backend"]
