FROM minik8s.xyz/rust:latest AS builder
WORKDIR /root/src
COPY . .
RUN cargo build --release
CMD ["gpu_server"]

FROM ubuntu:latest
WORKDIR /root/
COPY --from=builder /root/src/target/release/gpu_server ./
CMD ["./gpu_server"]
