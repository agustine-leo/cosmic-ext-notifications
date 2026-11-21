name := 'cosmic-notifications'
export APPID := 'com.system76.CosmicNotifications'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

export INSTALL_DIR := base-dir / 'share'

cargo-target-dir := env('CARGO_TARGET_DIR', 'target')
bin-src := cargo-target-dir / 'release' / name

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# `cargo clean` and removes vendored dependencies
clean-dist: clean
    rm -rf vendor vendor.tar

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' '--locked' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Runs after compiling a release build
run: build-release
    {{cargo-target-dir}}/release/cosmic-notifications

# Build and run with tokio-console enabled
tokio-console: build-release
    env TOKIO_CONSOLE=1 {{cargo-target-dir}}/release/cosmic-notifications

# Stock daemon version this fork is based on (upstream tag epoch-1.11.0)
stock-version := '1.11.0'
bin-dst := '/usr/local/bin/cosmic-notifications'

# Installs the fork to /usr/local/bin, ahead of the stock daemon (run `just build-release` first)
install:
    #!/usr/bin/env sh
    set -e
    [ "$(rpm -q --qf '%{VERSION}' cosmic-notifications)" = {{stock-version}} ] || { echo "cosmic-notifications is not {{stock-version}} — rebase the fork first"; exit 1; }
    [ -x {{bin-src}} ] || { echo "{{bin-src}} missing — run 'just build-release' first"; exit 1; }
    sudo install -Dm755 {{bin-src}} {{bin-dst}}
    echo "Installed. Takes effect at next login."

# Removes the fork; COSMIC's stock daemon is used again from next login
uninstall:
    sudo rm -f {{bin-dst}}

# Vendor dependencies locally
vendor:
    cp .cargo/config.default .cargo/config.toml
    cargo vendor --sync Cargo.toml \
        | head -n -1 >> .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    rm -rf vendor/winapi*gnu*/lib/*.a; \
    tar pcf vendor.tar vendor
    rm -rf vendor

# Extracts vendored dependencies
vendor-extract:
    #!/usr/bin/env sh
    rm -rf vendor
    tar pxf vendor.tar
