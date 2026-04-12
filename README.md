# battlecontrol

Live demo: https://gsenden.codeberg.page/battlecontrol/

## Source Of Truth

- Frontend app: `frontend/`
- Browser game code: `frontend/src/lib/game/`
- Deterministic battle logic: `game-logic/`
- Browser bridge: `game-logic-wasm/`
- Backend: `server/`
- Shared YAML/input definitions: `shared/`
- Shared generated/common Rust code: `common/`

The old root frontend no longer exists. Root `package.json` is now only the entrypoint for workspace commands.

## Dev

- Frontend dev: `npm run dev`
- Server dev: `npm run dev:server`
- Full stack dev: `npm run dev:all`
- Frontend build: `npm run build`
- Frontend tests: `npm run test`

`npm run dev` and `npm run build` automatically build `game-logic-wasm` first.

## Local Workflow

1. Install dependencies with `npm install`
2. Start only the frontend with `npm run dev`
3. Start only the server with `npm run dev:server`
4. Start both together with `npm run dev:all`

The root scripts are the standard entrypoints:
- root `package.json` orchestrates `frontend/` and wasm builds
- `frontend/package.json` contains only frontend-specific scripts
- `server/` runs via Cargo

## Build And Test

- Browser/WASM build: `npm run build:wasm`
- Frontend productiebuild: `npm run build`
- Frontend preview: `npm run preview`
- Frontend tests: `npm run test`
- Server run: `cargo run -p server`

## Routes

- `/` landing / app shell
- `/battle` Phaser battle scene
- `/credits` credits and licensing

## Credits And Licensing

BattleControl is a fan-made open source project inspired by The Ur-Quan Masters and the UQM HD MegaMod.

I have been a devoted fan of The Ur-Quan Masters since the 1990s, and the game gave me and my friends unforgettable memories. BattleControl is a tribute to that experience.

The Ur-Quan Masters project distinguishes between code and content:

- UQM code: GPLv2
- UQM content such as voice, dialogue, graphics, sounds, and music: CC BY-NC-SA 2.5

See [CREDITS.md](./CREDITS.md) for attribution and licensing notes.
