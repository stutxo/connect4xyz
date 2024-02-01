#!/bin/bash

# This script is used to build the Rust code and start the development server.
LLVM_PATH=$(brew --prefix llvm)


if [ "$1" = "release" ]; then
    # Build for release and copy index.html to 404.html
    AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build --release
     # We need this for GitHub Pages so when you go to /gameid it will load index.html instead of 404
    cp ./docs/index.html ./docs/404.html
elif [ "$1" = "dev" ]; then
    AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve --public-url / --port=1334
else
    echo "Invalid argument. Please use 'release' or 'dev'."
fi
