FROM rust:1.68-alpine3.17 AS bin-builder

# Install dependencies
RUN apk add --no-cache openssl-dev openssl-libs-static musl-dev

# This will build all dependencies and store them in docker's cache.
# This way, it won't be necessary to recompile everything everytime
# OPENSSL_STATIC is required, see: https://users.rust-lang.org/t/sigsegv-with-program-linked-against-openssl-in-an-alpine-container/52172 and https://users.rust-lang.org/t/how-to-link-openssl-statically/14912
COPY backend/Cargo.toml .
COPY backend/Cargo.lock .
RUN echo '[[bin]]' >> Cargo.toml && \
    echo 'name = "cache"' >> Cargo.toml && \
    echo 'path = "cache.rs"' >> Cargo.toml && \
    echo 'fn main() {eprintln!("Caching crates...")}' > cache.rs && \
    export OPENSSL_STATIC=1 && \
    export OPENSSL_LIB_DIR=/usr/lib && \
    export OPENSSL_INCLUDE_DIR=/usr/include/openssl && \
    cargo build --release 
RUN rm cache.rs && \
    rm Cargo.toml

# Build wimple-wkd
COPY backend/Cargo.toml .
COPY backend/src src
RUN export OPENSSL_STATIC=1 && \
    export OPENSSL_LIB_DIR=/usr/lib &&\ 
    export OPENSSL_INCLUDE_DIR=/usr/include/openssl && \
    cargo build --release


FROM node:19-alpine3.17 AS webpage-builder

# Build website
COPY website .
RUN npm install -g pnpm && \
    pnpm install && \
    pnpm run build
COPY assets assets

# Move website in templates folder
RUN mv dist assets/webpage


FROM alpine:3.17

# Put everything together
# It uses user `wkd` for added security
WORKDIR /wkd
RUN adduser --no-create-home --disabled-password wkd && \
    chown -R wkd:wkd /wkd
USER wkd
COPY --from=webpage-builder assets assets
COPY --from=bin-builder target/release/simple-wkd wkd

ENTRYPOINT [ "/wkd/wkd" ]