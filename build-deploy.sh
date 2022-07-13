#!/bin/sh

set -e

: ${TARGET:=armv7-unknown-linux-gnueabihf}
# : ${TARGET:=aarch64-unknown-linux-gnu}

CARGO_PROFILE_RELEASE_DEBUG=true
RUSTFLAGS='-C force-frame-pointers=y'
cargo build --target $TARGET --release

./deploy.sh