# Creating new project

Oxygengine uses a concept called "Game Templates" where instead of starting from
blank page, you create your new project from one of existing game templates:
- `desktop-headless-game` - used mostly for game servers logic, it doesn not
  provide access to any of rendering or audio features, but gives you all the
  features needed to create a game server logic.
- `web-composite-game` - used for making visually simple games that uses Web
  Canvas rendering.
- `web-composite-visual-novel` - used to jumpstart working on a completely
  asset-driven Visual Novel game.
- `web-ha-game` - used to make visually more demanding games that needs to render
  greater number of entities, manipulate and enhance game visuals with shaders.

**Create new game project with default (`web-ha-game`) game template:**
```bash
oxygengine-ignite new <project-name>
```
for example:
```bash
oxygengine-ignite new dream-game
```

**Create new game with specified game template to use:**
```bash
oxygengine-ignite new <project-name> -p <game-template-name>
```
for example:
```bash
oxygengine-ignite new dream-game-server -p desktop-headless-game
```

**Create new game project in specified path:**
```bash
oxygengine-ignite new <project-name> -d <path-to-parent-directory>
```
for example:
```bash
oxygengine-ignite new dream-game -d ~/game-projects/
```
