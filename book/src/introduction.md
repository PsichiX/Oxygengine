# Introduction

In this book we will cover all essential information regarding usage of Oxygengine.
That being said, we should explain first what Oxygengine is, what it aims to be
and what's definitely not made for.

## The goal

Oxygengine is an all-batteries-included kind of engine, focused on being Web-first.
The **goal** is for it to be a Rust version of Unreal Engine for 2D games, which
made us take a very important decision from the beggining, for engine being
completely asset-driven game engine with game editor dedicated mostly for **Game
Designers** to use. That means, all architecture decisions are made to ease
interacting with al parts of the engine from game editor with next to none use of
code.

This also means that while Oxygengine is a general purpose 2D game engine, it aims
to provide specialized modules to ease work on specific game genres. At the moment
we already have specialized game modules for these kind of games:

- [Visual Novel](https://github.com/PsichiX/Oxygengine/tree/master/engine/visual-novel)
- [Role-Playing Game](https://github.com/PsichiX/Oxygengine/tree/master/engine/overworld)

## Where we are now

At this point Oxygengine has near its stable version from the code architecture
point of view, but **does not have yet** a game editor ready to use, although
editor is still work in progress and it might be released in 2022, not yet decided
on particular month.

## Where could i use Oxygengine

Since this engine is Web-first and 2D-focused and it aims to give you a specific
games genre solutions to let you just make a game, you most likely would want to
use it if you aim for making one of these kind of games:
- Visual Novels
- RPGs

Genres we will cover soon in new specialized engine modules:
- Platformers
- Shooters

Every other genres, although they are possible to make, we do not provide or
rather not yet plan to provide a specialized engine module.

## Where i can't use Oxygengine

This engine is basically useless (yet) for any kind of 3D game (maybe except
original Doom-like games, but these would require heavy hammering).
Also definitely do not use it (yet) to make desktop or gaming console titles
(but there are plans for desktop/steam platforms and if we get lucky to sign a
partnership with gaming console producents, we might provide a private dedicated
engine modules).
