# Connect4.xyz

A game inspired by [jesterui](https://github.com/jesterui/jesterui). Play Connect4 over `nostr`.

## Nostr Events

### 1. New Game

When a new game is created from the home page, a replaceable event with the game id as the tag is sent. The game ID can be retrieved from the URL, which can be shared with the opponent.

**Kind**: `Replaceable(1111)`

### 2. Join Game

Upon an opponent joining the game, a `join game` ephemeral message is sent. This message isn't stored on the server and is transmitted to the other connected player. The message includes the game id and the public key of the joining player.

**Kind**: `Ephemeral`

### 3. Game Started

When the first player recieves the `join game` message, a `start game` replaceable message is dispatched, replacing the new game message.


player 2 ressubscribes to player 1's public key only and then sends its own `start game` message so that player 1 can subscribe to player 2's public key.


If a does not match the public keys in the start game message, then they are set to spectator mode.

**Kind**: `Replaceable(1111)`

### 4. Send Input

send game inputs

**Kind**: `Regular(4444)`

### 5. Send Replay

for sending replay game message

**Kind**: `Regular(4444)`



build locally using trunk. https://trunkrs.dev/


```
set LLVM_PATH $(brew --prefix llvm)
AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve --public-url /  --port=1334
```

Build
```
set LLVM_PATH $(brew --prefix llvm)
AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build --release
```

