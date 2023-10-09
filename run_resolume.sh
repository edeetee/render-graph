#!/bin/bash

set -e
trap cleanup EXIT

function cleanup {
  pkill -x Arena
  cd $OLDDIR
}

pkill -x Arena || true

OLDDIR="$(pwd)"

cd "$(dirname "$0")"

echo "Building"
cargo build --release -p ffgl-lib

echo "Copying to plugin bundle"
cp "target/release/libffgl_lib.dylib" "/Library/Graphics/FreeFrame Plug-Ins/FFGLRsTest.bundle/Contents/MacOS/FFGLRsTest"

echo "Running resolume"
open "/Applications/Resolume Arena/Arena.app" --args --nosplash

echo "Listening to resolume logs"
tail -F "/Users/edwardtaylor/Library/Logs/Resolume Arena/Resolume Arena log.txt"