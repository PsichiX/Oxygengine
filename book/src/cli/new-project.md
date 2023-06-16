# Creating new project

Oxygengine uses a concept called "Game Templates" where instead of starting from
blank page, you create your new project from one of existing game templates:
- `ha-base` - contains barebones setup for the simplest game example.
- `ha-game` - used to make visually more demanding games that needs to render
  greater number of entities, manipulate and enhance game visuals with materials.

**Create new game project with default (`ha-game`) game template:**
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
oxygengine-ignite new dream-game -p ha-game
```

**Create new game project in specified path:**
```bash
oxygengine-ignite new <project-name> -d <path-to-parent-directory>
```
for example:
```bash
oxygengine-ignite new dream-game -d ~/game-projects/
```
