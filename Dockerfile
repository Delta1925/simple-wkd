FROM rust:1.68 AS bin-builder

RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install clang llvm pkg-config nettle-dev -y
COPY . .
RUN cargo build --release


FROM node:19 AS webpage-builder

COPY website .
RUN npm install -g pnpm && \
    pnpm install && \
    pnpm run build
COPY assets assets
RUN mv dist assets/webpage


FROM debian:bullseye-slim

WORKDIR /simplewkd
RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install clang llvm pkg-config nettle-dev -y && \
    rm -rf /var/lib/apt/lists/* && adduser --no-create-home simplewkd && \
    chown -R simplewkd:simplewkd /simplewkd && \
    chmod -R 777 /simplewkd
USER simplewkd
COPY --from=webpage-builder assets assets
COPY --from=bin-builder target/release/simple-wkd simple-wkd

ENTRYPOINT [ "./simple-wkd" ]