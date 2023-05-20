ARG base="alpine:3.18"


FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM ${base} AS bin-builder

ARG TARGETPLATFORM
COPY --from=xx / /

# Enable cargo sparse index for faster update times, see: https://blog.rust-lang.org/inside-rust/2023/01/30/cargo-sparse-protocol.html
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN apk add clang lld musl-dev cargo
COPY backend .
RUN xx-cargo build --release --target-dir ./build && \
    xx-verify ./build/$(xx-cargo --print-target-triple)/release/simple-wkd
RUN mv ./build/$(xx-cargo --print-target-triple)/release/simple-wkd simple-wkd-executable


FROM --platform=$BUILDPLATFORM ${base} AS webpage-builder

RUN apk add npm
COPY website .
RUN npm install -g pnpm && \
    pnpm install && \
    pnpm run build
COPY assets assets
# Move website in templates folder
RUN mv dist assets/webpage


FROM ${base}

# The final image uses user `wkd` for added security
WORKDIR /wkd
RUN apk add --no-cache libgcc && \
    adduser --no-create-home --disabled-password wkd && \
    chown -R wkd:wkd /wkd
USER wkd
COPY --from=webpage-builder assets assets
COPY --from=bin-builder simple-wkd-executable wkd

ENTRYPOINT [ "/wkd/wkd" ]