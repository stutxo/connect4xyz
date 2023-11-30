#!/bin/bash

# Set LLVM_PATH variable
LLVM_PATH=$(brew --prefix llvm)

# Check the script argument
if [ "$1" = "release" ]; then
    # Build for release and copy index.html to 404.html
    AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build --release
    cp ./docs/index.html ./docs/404.html
elif [ "$1" = "dev" ]; then
    # Start the Rust development server
    AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve --public-url / --port=1334
else
    echo "Invalid argument. Please use 'release' or 'dev'."
fi

