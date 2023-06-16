# Oxygengine Ignite CLI tool

Although Oxygengine can be used like any other crate, it's better to install
Ignite CLI tool that will govern most vital operations on your game project
development:
```bash
cargo install oxygengine-ignite
```

Additionally it's encouraged to install `just` CLI too:
```bash
cargo install just
```
Each created game project contains `justfile` with set of handy shortcut
commands for common day-to-day operations useful when developing a game.
Once in game project root directory, run `just list` to see list of commands.