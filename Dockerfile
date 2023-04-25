ARG base="alpine:3.17"

FROM ${base} AS bin-builder

# Disable static linking, see: https://users.rust-lang.org/t/sigsegv-with-program-linked-against-openssl-in-an-alpine-container/52172
# Enable cargo sparse index for faster update times, see: https://blog.rust-lang.org/inside-rust/2023/01/30/cargo-sparse-protocol.html
ENV RUSTFLAGS='-C target-feature=-crt-static' \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN apk add --no-cache cargo openssl-dev musl-dev
# This will build all dependencies and store them in docker's cache.
# This way, it won't be necessary to recompile everything everytime
COPY backend/Cargo.toml .
COPY backend/Cargo.lock .
RUN echo '[[bin]]' >> Cargo.toml && \
    echo 'name = "cache"' >> Cargo.toml && \
    echo 'path = "cache.rs"' >> Cargo.toml && \
    echo 'fn main() {eprintln!("Caching crates...")}' > cache.rs && \
    cargo build --release 
RUN rm cache.rs && \
    rm Cargo.toml
# Build wimple-wkd
COPY backend/Cargo.toml .
COPY backend/src src
RUN cargo build --release


FROM --platform=$BUILDPLATFORM ${base} AS webpage-builder

RUN apk add --no-cache npm
COPY website .
RUN npm install -g pnpm && \
    pnpm install && \
    pnpm run build
COPY assets assets
# Move website in templates folder
RUN mv dist assets/webpage


FROM ${base}

# The final image uses user `wkd` for added security
# It also installs libgcc, because the executable is dynamically linked to it
WORKDIR /wkd
RUN apk add --no-cache libgcc && \
    adduser --no-create-home --disabled-password wkd && \
    chown -R wkd:wkd /wkd
USER wkd
COPY --from=webpage-builder assets assets
COPY --from=bin-builder target/release/simple-wkd wkd

ENTRYPOINT [ "/wkd/wkd" ]