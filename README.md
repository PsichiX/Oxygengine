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

## Project Setup
Create Oxygen Engine project with `oxygengine-ignite`:
```bash
cd /path/to/parent/
oxygengine-ignite new <project-name>
```
Which will create default web game project using `web-composite-game` preset.
You can create projects with different presets:
- __desktop-headless-game__ - typical server-like project without graphics.
- __hybrid-script-game__ - it's a `web-composite-game` with JavaScript scripting module to make prototyping with Oxygengine faster and easier. Please note that JS scripting compared to pure Rust version is slower so it's not a great idea to use it in a production-ready game with lots of entities and logic, it should only help to prototype a game systems that will be later rewritten in Rust for best optimization. __NOTE: Because of engine not using NPM at all since version 0.12.0, this template is forced to use version 0.11.2 of the engine until new way of JavaScript scripting method will be developed.__

using:
```bash
cd /path/to/parent/
oxygengine-ignite new <project-name> -p desktop-headless-game
```
You can also tell it where to create project:
```bash
oxygengine-ignite new <project-name> -d /path/to/parent/
```

**Updating to new engine version:**
- reinstall `oxygengine-ignite`:
  ```bash
  cargo install oxygengine-ignite --forced
  OXY_UPDATE_PRESETS=1 oxygengine-ignite --help
  ```
- update `oxygengine` version either in `Cargo.toml` or by calling: `cargo update`
- upgrading from versions before 0.12.0 requires to create new project with  latest ignite tool, then copy by hand your source files to the new project sources, as well as put assets from old project `/static/assets` directory into new project `/assets` directory, preferably using new way of assets directory structure (so you won't have to modify pipeline.json file a lot).

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

## Roadmap
Current milestone progress: https://github.com/PsichiX/Oxygengine/projects/1
