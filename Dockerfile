ARG PROJECT='paper'
ARG VERSION='1.21'

FROM rust:slim AS builder
WORKDIR /app
RUN RUSTUP_DIST_SERVER="https://rsproxy.cn" \ 
    RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup" \
    rustup target add x86_64-unknown-linux-musl

COPY ./fetch-paper .
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch AS downloader
WORKDIR /app
COPY --from=builder /app/target/release/fetchpaper /app
RUN ["/app/fetchpaper", "$PROJECT", "-v", "$VERSION", "--path", "/target.jar"]
