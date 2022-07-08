#!/bin/sh

set -e

: ${TARGET:=armv7-unknown-linux-gnueabihf}
# : ${TARGET:=aarch64-unknown-linux-gnu}

cargo build --target $TARGET --release
./deploy.sh