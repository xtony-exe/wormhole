FROM rust:alpine AS builder
WORKDIR /home/rust/src
RUN apk --no-cache add musl-dev
COPY . .
RUN cargo install --path .

FROM scratch
LABEL maintainer="XTONY"
LABEL org.opencontainers.image.title="wormhole"
LABEL org.opencontainers.image.description="A fast, modern TCP tunnel — by THINKING TEAM"
LABEL org.opencontainers.image.version="1.0.0"
COPY --from=builder /usr/local/cargo/bin/wormhole .
USER 1000:1000
ENTRYPOINT ["./wormhole"]
