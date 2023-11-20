FROM rust:1.70
WORKDIR /app
COPY . .
EXPOSE 8080
CMD cargo run --release