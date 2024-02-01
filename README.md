# Connect4.xyz

A game inspired by chess over nostr - [jesterui](https://github.com/jesterui/jesterui).

Play Connect4 over `nostr`. https://connect4.xyz

Anyone is free to host this game themselves, but please dont cheat! :D I am deploying to github pages via the docs folder in this repo.


## Why?

You dont have to pay anything to host this game, and it will never go down, because someone can always host it, or run it locally.

web3 games are bullshit centralised facades.

At least with this game you can play it with zero need to trust a single server, and as long as a nostr relay exists somewhere on the internet, this game will continue to work.

Ofc you have to be honest yourself but this is only really meant for playing with friends, you can cheat at connect4 in real life but it would be a bit weird. xD


## Nostr Events Used

Clients connect to relays and check for notes with the tag `connect4.xyz game_id = {}` where game_id is the id of the game, which is randomly generated and added to the URL.

Players can share this url to invite others to play or spectate.

The first two players in the game will listen to each others pubkey to avoid shenanigans. All other players who connect will be in spectate mode.

### 1. New Game

event to list a new game.

**Kind**: `Regular(4444)`

### 2. Join Game

event sent by the second player to join the game lobby.

**Kind**: `Regular(4444)`

### 3. Send Input

To relay game inputs.

**Kind**: `Regular(4444)`

## Building and Running Locally

Install [trunk](https://trunkrs.dev/) to build and serve locally.

### Serve locally with trunk

```
./build.sh dev
```

### Build locally with trunk
```
./build.sh release
```
