FROM devkitpro/devkita64:20260219

# We set the "maintainer" label just to override the value inherited from the devkitPro image
LABEL maintainer="doukutsu-rs maintainers" \
      org.opencontainers.image.authors="doukutsu-rs maintainers" \
      org.opencontainers.image.description="Build environment for the Horizon port of doukutsu-rs." \
      org.opencontainers.image.source="https://github.com/doukutsu-rs/doukutsu-rs"

# Setup Rust environment vars
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.92.0 \
    # The custom toolchain is missing the `cargo` binary, but it falls back to binary from the nightly toolchain
    # (this works only for `nightly`, any other toolchains, even if they're nightly, won't work)
    RUST_DEFAULT_TOOLCHAIN=nightly

# Install build dependencies
RUN apt update; \
    apt install -y clang libclang-dev zstd;

# Install Rust
RUN set -eux; \
    curl --proto '=https' --tlsv1.2 -fSL https://sh.rustup.rs | sh -s -- --default-toolchain $RUST_DEFAULT_TOOLCHAIN --profile minimal -c rust-src -y; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup default $RUST_DEFAULT_TOOLCHAIN;

# Install patched toolchain
RUN \
    curl -fSL https://github.com/doukutsu-rs/rust-hos/releases/download/${RUST_VERSION}-horizon/${RUST_VERSION}-switch_x86_64-unknown-linux-gnu.tar.zst | \
    tar -x --zstd -f - -C ${RUSTUP_HOME}/toolchains/;

# For testing local toolchains (comment the previous `RUN` instruction)
# COPY ${RUST_VERSION}-switch ${RUSTUP_HOME}/toolchains/${RUST_VERSION}-switch

WORKDIR /workspace/drshorizon

ENTRYPOINT ["bash"]
