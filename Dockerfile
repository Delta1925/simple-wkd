FROM rust:1.68-alpine3.17 AS bin-builder

COPY backend .
RUN apk add --no-cache openssl-dev openssl-libs-static musl-dev
RUN export OPENSSL_STATIC=1 && \
    export OPENSSL_LIB_DIR=/usr/lib &&\ 
    export OPENSSL_INCLUDE_DIR=/usr/include/openssl && \
    cargo build --release


FROM node:19-alpine3.17 AS webpage-builder

COPY website .
RUN npm install -g pnpm && \
    pnpm install && \
    pnpm run build
COPY assets assets
RUN mv dist assets/webpage


FROM alpine:3.17

WORKDIR /wkd
RUN adduser --no-create-home --disabled-password wkd && \
    chown -R wkd:wkd /wkd
USER wkd
COPY --from=webpage-builder assets assets
COPY --from=bin-builder target/release/simple-wkd wkd

ENTRYPOINT [ "/wkd/wkd" ]