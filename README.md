# Connect4.xyz

A game inspired by chess over nostr - [jesterui](https://github.com/jesterui/jesterui).

Play Connect4 over `nostr`. https://connect4.xyz

Anyone is free to host this game themselves, but please dont cheat! :D I am deploying to github pages via the docs folder in this repo.

## Nostr Events Used

Clients connect to relays and check for notes with the tag `connect4.xyz game_id = {}` where game_id is the id of the game, which is randomly generated and added to the URL.

Players can share this url to invite others to play or spectate.

The first two players in the game will listen to each others pubkey only to avoid shenanigans. All other players who connect will be in spectate mode.

### 1. New Game

event to list a new game.

**Kind**: `Regular(4444)`

### 2. Join Game

event sent by the second player to join the game lobby.

**Kind**: `Regular(4444)`

### 3. Send Input

To relay game inputs.

**Kind**: `Regular(4444)`

### 4. Send Replay

For dispatching replay game messages.

**Kind**: `Regular(4444)`

## Building and Running Locally

Utilize [trunk](https://trunkrs.dev/) to build and serve locally.

### server dev (Run this to open in a local browser)

```
./build.sh dev
```

release
```
./build.sh release
```
