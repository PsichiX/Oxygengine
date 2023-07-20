# Introduction

In this book we will cover all essential information regarding usage of Oxygengine.
That being said, we should explain first what Oxygengine is, what it aims to be
and what's definitely not made for.

## The goal

Oxygengine is an all-batteries-included kind of engine, focused on being Web-first
but slowly moving more into more platforms such as desktops and consoles.
The **goal** is for it to be a Rust version of Unreal Engine for 2D games, which
made us take a very important decision from the beggining, for it being a completely
asset-driven game engine with game editor dedicated mostly for **Game Designers** and
**Artists** to use. That means, all architecture decisions are made to ease interacting
with all parts of the engine from game editor with next to none use of code.

This also means that while Oxygengine is a general purpose 2D game engine, it aims
to provide specialized modules to ease work on specific game genres. At the moment
we already have specialized game modules for these kind of games:

- [Visual Novel](https://github.com/PsichiX/Oxygengine/tree/master/engine/visual-novel)
- [Role-Playing Game](https://github.com/PsichiX/Oxygengine/tree/master/engine/overworld)

## Where we are now

At this point Oxygengine is near its stable version from the code architecture
point of view, but **does not have** a game editor ready to use yet, although
editor is still work in progress and it might be released in 2024, not yet decided
on any particular time - we already have moved from web-based to desktop-based editor.

## Where could i use Oxygengine

Since this engine is Web-first (and desktop/console-second) 2D-focused and it aims to
give you a specific games genre solutions to let you just make a game and not reinvent
the wheel, you most likely would want to use it if you aim for making one of these kind
of games:
- RPGs
- Visual Novels

Genres we will cover soon in new specialized engine modules:
- Shooters
- Platformers
- Puzzle

Every other genres, although they are possible to make, we do not provide or rather not
yet plan to provide a specialized engine module.

## Where i can't use Oxygengine

This engine is basically useless (yet) for any kind of 3D games (maybe except original
Doom-like games, but these would require heavy hammering). Also definitely do not use
it (yet) to make gaming console titles (although there are plans for these platforms
and if we get lucky to sign a partnership with gaming console producents, we might
provide private dedicated engine modules and game templates).
