# Connect4.xyz

A game inspired by chess over nostr - [jesterui](https://github.com/jesterui/jesterui).

Play Connect4 over `nostr`. https://connect4.xyz

## Nostr Events Used

### 1. New Game

When a new game is initiated from the home page, a replaceable event tagged with the game ID is sent. This game ID, which can be extracted from the URL, can be shared with the opponent.

**Kind**: `Replaceable(1111)`

### 2. Join Game

When an opponent joins the game, a `join game` ephemeral message is dispatched. This message isn't stored on the server and is only sent to the other connected player. It contains the game ID and the public key of the player who just joined.

**Kind**: `Ephemeral`

### 3. Game Started

Once the first player receives the `join game` message, a replaceable `start game` message is sent out, replacing the initial game message with a message containing both players' public keys.

Player 2 then resubscribes exclusively to Player 1's and its own public key and sends its own `start game` message. This ensures Player 1 can subscribe to Player 2's public key.

If a player's public key doesn't match those in the `start game` message, they are designated as spectators.

**Kind**: `Replaceable(1111)`

### 4. Send Input

To relay game inputs.

**Kind**: `Regular(4444)`

### 5. Send Replay

For dispatching replay game messages.

**Kind**: `Regular(4444)`

## Building and Running Locally

Utilize [trunk](https://trunkrs.dev/) to build and serve locally.

### Serve (Run this to open in a local browser)

```
set LLVM_PATH $(brew --prefix llvm)
AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve --public-url /  --port=1334
```

Build
```
set LLVM_PATH $(brew --prefix llvm)
AR="$LLVM_PATH/bin/llvm-ar" CC="$LLVM_PATH/bin/clang" RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build --release
```

