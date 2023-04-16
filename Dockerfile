FROM rust:1.68 AS bin-builder

RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install clang nettle-dev pkg-config libssl-dev -y
COPY backend .
RUN cargo build --release


FROM node:19 AS webpage-builder

COPY website .
RUN npm install -g pnpm && \
    pnpm install && \
    pnpm run build
COPY assets assets
RUN mv dist assets/webpage


FROM debian:bullseye-slim

WORKDIR /wkd
RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install ca-certificates -y && \
    rm -rf /var/lib/apt/lists/* && \
    adduser --no-create-home wkd && \
    chown -R wkd:wkd /wkd && \
    chmod -R 777 /wkd
USER wkd
COPY --from=webpage-builder assets assets
COPY --from=bin-builder target/release/simple-wkd wkd

ENTRYPOINT [ "/wkd/wkd" ]