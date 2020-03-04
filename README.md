![logo](https://raw.githubusercontent.com/PsichiX/Oxygengine/master/media/oxygengine-dark-logo.svg?sanitize=true)

# Oxygengine ![travis-ci status](https://travis-ci.org/PsichiX/Oxygengine.svg?branch=master) ![crates-io version](https://raster.shields.io/crates/v/oxygengine.png) ![GitHub tag](https://img.shields.io/github/v/release/PsichiX/Oxygengine?include_prereleases&style=social)
### The hottest HTML5 + WASM game engine for games written in Rust with `web-sys`.

## Table of contents
1. [Understanding ECS](#understanding-ecs)
1. [Installation](#installation)
1. [Teaser](#teaser)
1. [Project Setup](#project-setup)
1. [Building for development and production](#building-for-development-and-production)
1. [Roadmap](#todo--roadmap)

## Understanding ECS
Oxygengine is highly based on `specs` crate used for its ECS framework.
You can get understanding of it by reading `specs` book and tutorials here: https://specs.amethyst.rs/docs/tutorials/

## Installation
1. Make sure that you have latest `node.js` with `npm` tools installed (https://nodejs.org/)
1. Make sure that you have latest `wasm-pack` toolset installed (https://rustwasm.github.io/wasm-pack/installer/)
1. Make sure that you have latest `oxygengine-ignite` application installed (from project repository releases)

## Teaser
![Visual Novel](https://raw.githubusercontent.com/PsichiX/Oxygengine/master/media/oxygengine-visual-novel-teaser.gif)

## Project Setup
Create Oxygen Engine project with `oxygengine-ignite`:
```bash
cd /path/to/parent/
oxygengine-ignite new <project-name>
```
Which will create default web game project using `web-composite-game` preset.
Then you have to go to your project directory and run `npm install` to install all nodejs dependencies.
You can create projects with different presets:
- __desktop-headless-game__ - typical server-like project without graphics.
- __hybrid-script-game__ - it's a `web-composite-game` with JavaScript scripting module to make prototyping with Oxygengine faster and easier. Please note that JS scripting compared to pure Rust version is slower so it's not a great idea to use it in a production-ready game with lots of entities and logic, it should only help to prototype a game systems that will be later rewritten in Rust for best optimization.

using:
```bash
cd /path/to/parent/
oxygengine-ignite new <project-name> -p desktop-headless-game
```
You can also tell it where to create project:
```bash
oxygengine-ignite new <project-name> -d /path/to/parent/
```

## Building for development and production
- Launch live development with hot reloading (app will be automatically
  recompiled in background):
```bash
npm start
```
- Build production distribution (will be available in `/dist` folder):
  with debug symbols:
  ```bash
  npm run build
  ```
  optimized release mode:
  ```bash
  OXY_RELEASE=1 npm run build
  ```
- Build crate without of running dev env:
```bash
cargo build
```

## TODO / Roadmap
- Engine editor (standalone Electron application?)
- Hardware renderer (WebGPU hardware renderer backend)
