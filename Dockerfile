FROM rust:latest
WORKDIR /usr/src/app
COPY . .
RUN apt-get update && apt-get install -y libssl1.1 libssl-dev
RUN cargo build --release
CMD ["./target/release/haltion"]
