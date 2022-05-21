FROM ubuntu:latest
COPY ./target/release/ /gpu_server/
RUN apt update && apt install -y openssl libssl-dev
