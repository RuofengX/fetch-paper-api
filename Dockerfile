ARG PROJECT='paper'
ARG VERSION='1.21'
FROM docker.1ms.run/rust:bookworm AS builder
WORKDIR /app
RUN sed -i 's@deb.debian.org@repo.huaweicloud.com@g' /etc/apt/sources.list.d/debian.sources
RUN apt update && apt install -y pkg-config libssl-dev
COPY ./fetch-paper .
RUN cargo build --release

FROM docker.1ms.run/rust:bookworm AS downloader
WORKDIR /app
COPY --from=builder /app/target/release/fetchpaper /app
RUN /app/fetchpaper paper -v 1.21 --path /target.jar

FROM scratch AS file
COPY --from=builder /target.jar /target.jar
