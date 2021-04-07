![logo](https://raw.githubusercontent.com/PsichiX/Oxygengine/master/media/oxygengine-dark-logo.svg?sanitize=true)

# Oxygengine ![GitHub CI](https://github.com/PsichiX/Oxygengine/workflows/Rust/badge.svg) ![crates-io version](https://raster.shields.io/crates/v/oxygengine.png) ![GitHub tag](https://img.shields.io/github/v/release/PsichiX/Oxygengine?include_prereleases&style=social)
### The hottest HTML5 + WASM game engine for games written in Rust with `web-sys`.

## Table of contents
1. [Understanding ECS](#understanding-ecs)
1. [Installation](#installation)
1. [Teaser](#teaser)
1. [Project Setup](#project-setup)
1. [Building for development and production](#building-for-development-and-production)
1. [Roadmap](#roadmap)

## Understanding ECS
Oxygengine is highly based on `specs` crate used for its ECS framework.
You can get understanding of it by reading `specs` book and tutorials here: https://specs.amethyst.rs/docs/tutorials/

## Installation
Make sure that you have latest `oxygengine-ignite` binary installed (`cargo install oxygengine-ignite`) - this binary is a set of vital tools that will govern managing most of your project.

## Teaser
![Visual Novel](https://raw.githubusercontent.com/PsichiX/Oxygengine/master/media/oxygengine-visual-novel-teaser.gif)
![RPG](https://raw.githubusercontent.com/PsichiX/Oxygengine/master/media/oxygengine-raui-navigation.gif)

## Project Setup
Create Oxygen Engine project with `oxygengine-ignite`:
```bash
cd /path/to/parent/
oxygengine-ignite new <project-name>
```
Which will create default web game project using `web-composite-game` preset.
You can create projects with different presets:
- __desktop-headless-game__ - typical server-like project without graphics.

using:
```bash
cd /path/to/parent/
oxygengine-ignite new <project-name> -p desktop-headless-game
```
You can also tell it where to create project:
```bash
oxygengine-ignite new <project-name> -d /path/to/parent/
```

#### Updating to new engine version:
- reinstall `oxygengine-ignite`:
  ```bash
  cargo install oxygengine-ignite --forced
  OXY_UPDATE_PRESETS=1 oxygengine-ignite --help
  ```
- update `oxygengine` version either in `Cargo.toml` or by calling: `cargo update`
- upgrading from versions before 0.12.0 requires to create new project with  latest ignite tool, then copy by hand your source files to the new project sources, as well as put assets from old project `/static/assets` directory into new project `/assets` directory, preferably using new way of assets directory structure (so you won't have to modify pipeline.json file a lot).

#### Speeding up compilation for new projects (best use case for gamejams and quick feature prototypes):
- install SCCACHE, a tool for caching and sharing prebuilt dependencies between multiple game projects (https://github.com/mozilla/sccache):
  ```bash
  cargo install sccache
  ```
- add these lines to the `Cargo.toml`:
  ```toml
  [package.metadata]
  # path to the sccache binary
  sccache_bin = "sccache.exe"
  # path to the sccache cache directory
  sccache_dir = "D:\\sccache"
  ```

From now on you will have to wait for full long engine build only once, for any other new game project you create, it will perform first compilation in matter of minute, not 20.

## Building for development and production
- Launch live development with active code recompilation and assets baking in the background on change:
  ```bash
  oxygengine-ignite live
  ```
  additionally to allow it to start http server to serve your build files in the browser, run:
  ```bash
  oxygengine-ignite live -- -p 8080
  ```
  files will be served from: http://localhost:8080.
- Build binaries in debug or release mode (binaries will be put in `/bin` folder):

  with debug symbols:
  ```bash
  oxygengine-ignite build
  ```
  optimized release mode:
  ```bash
  oxygengine-ignite build -p release
  ```
- Build only the crate:
  ```bash
  cargo build
  ```
- Package application build with assets ready for distribution (package files will be put in `/dist` folder):
  ```bash
  oxygengine-ignite package
  ```
  this command will run release build, assets pipeline and bundle package.

  To produce a debug mode package you have to run:
  ```bash
  oxygengine-ignite package -d
  ```

## Roadmap
Current milestone progress: https://github.com/PsichiX/Oxygengine/projects/1
