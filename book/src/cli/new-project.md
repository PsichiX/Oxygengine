# Creating new project

Oxygengine uses a concept called "Game Templates" where instead of starting from
scratch, you create your new project from one of few existing game templates:
- `base` - Contains barebones setup for the simplest game example.
  Use it create "blank project" to later configure with more modules.
- `game` - Contains more complex setup with modules you would use in production-level
  games. Use it if you decided to get deep into developing your dream game.
- `prototype` - Contains ergonomic framework for quick and dirty game prototypes
  with more imperative than data-driven approach.

**Create new game project with default (`game`) game template:**
```bash
oxygengine-ignite new <project-name>
```
for example:
```bash
oxygengine-ignite new dream-game
```

**Create new game project with specified game template to use:**
```bash
oxygengine-ignite new <project-name> -p <game-template-name>
```
for example:
```bash
oxygengine-ignite new dream-game -p game
```

**Create new game project in specified path:**
```bash
oxygengine-ignite new <project-name> -d <path-to-parent-directory>
```
for example:
```bash
oxygengine-ignite new dream-game -d ~/game-projects/
```
