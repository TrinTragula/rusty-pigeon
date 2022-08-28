# Rusty Pigeon

Another UCI chess engine written in Rust. This time it should be able to beat you up consistently and without afterthoughts.

| Rusty Pigeon                  | Link          |
| -------------                 | ------------- |
| UCI engine                    | https://github.com/TrinTragula/rusty-pigeon/releases  |
| WEB app (WASM, weaker build)  | http://rusty-pigeon.netlify.app/  |
| Lichess BOT                   | https://lichess.org/@/RustyPigeon  |

## Features

- UCI protocol (play against it in your favourite chess program) with simple time management
- Implemented totally from scratch using bitboards, zobrist hashing, magics, all the good stuff
- Interactive play from command line
- WASM build to play against the engine on any web app
- Lichess bot (online randomly, it will be online 24/7 once I find a good home for it)
- (soon) Tauri app to play against the engine locally with a nice GUI

## Missing stuff

- Still needs to handle threefold repetition and the fifty-move rule
- Single threaded
