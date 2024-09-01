# wordplay - multiplayer word games
https://wordplay.lol/

## features
- pretty & fast!
- word bomb
- anagrams
- epic post game information
- in game chat
- game rejoining
- public rooms
- message rate limiting
- spectating

## uses
- `/client`
  - [TypeScript](https://www.typescriptlang.org/)
  - [SolidJS](https://www.solidjs.com/)
  - [Vite](https://vitejs.dev)
  - [TailwindCSS](https://tailwindcss.com)
- `/server`
  - [Rust](https://www.rust-lang.org/)
  - [axum](https://github.com/tokio-rs/axum)
- `/prompt_gen` (script for generating word bomb prompts)
  - [Rust](https://www.rust-lang.org/)

[Docker](https://docs.docker.com/) / [Docker Compose](https://docs.docker.com/compose/) is used to build and manage deployments

## special thanks
- avatar generation is taken from [vercel/avatar](https://github.com/vercel/avatar)
- most icons are from [SVG Repo](https://www.svgrepo.com/)
- friends for playtesting and feedback