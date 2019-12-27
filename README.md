![logo](https://raw.githubusercontent.com/PsichiX/Oxygengine/master/media/oxygengine-dark-logo.svg?sanitize=true)

# Oxygengine
### The hottest HTML5 + WASM game engine for games written in Rust with `web-sys`.

## Table of contents
1. [Installation](#installation)
1. [Project Setup](#project-setup)
1. [Building for development and production](#building-for-development-and-production)
1. [Roadmap](#roadmap)

## Installation
1. Make sure that you have latest `node.js` with `npm` tools installed (https://nodejs.org/)
1. Make sure that you have latest `wasm-pack` toolset installed (https://rustwasm.github.io/wasm-pack/installer/)
1. Make sure that you have latest `oxygengine-ignite` application installed (from project repository releases)

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
```bash
npm run build
```
- Build crate without of running dev env:
```bash
cargo build --all
```

## TODO / Roadmap
- UI widgets
- Prefabs (loading scenes from asset)
- Hardware renderer
- WebGL hardware renderer backend
- 2D physics
