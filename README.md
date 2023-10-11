# connect4.xyz

```
set LLVM_PATH $(brew --prefix llvm)
AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve --public-url /  --port=1334
```

Build
```
set LLVM_PATH $(brew --prefix llvm)
AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build --release
```

