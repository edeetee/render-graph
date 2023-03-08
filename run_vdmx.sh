#!/bin/bash

set -e
trap cleanup EXIT

function cleanup {
  pkill -x VDMX5
  cd $OLDDIR
}

OLDDIR="$(pwd)"

cd "$(dirname "$0")"

echo "Building"
cargo build --release --lib --features=ffgl_plugin

echo "Copying to plugin bundle"
cp "target/release/librender_graph.dylib" "/Library/Graphics/FreeFrame Plug-Ins/FFGLRsTest.bundle/Contents/MacOS/FFGLRsTest"

echo "Running VDMX"
open "/Applications/VDMX5.app"

sleep 1
cd "/Users/edwardtaylor/Library/Logs/VDMX5"
RECENT_LOG=$(ls -Art | tail -n 1)
echo "Opening file $RECENT_LOG"
tail -F $RECENT_LOG