#!/bin/sh

: ${TARGET:=armv7-unknown-linux-gnueabihf}
# : ${TARGET:=aarch64-unknown-linux-gnu}

scp target/$TARGET/release/rpi-rust pi@192.168.86.245:~/stars