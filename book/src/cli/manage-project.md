# Managing your project

Once you've created your project, there is set of commands to run.

> **NOTE:** Each game template contains setup for multiple platforms,
  although for day-to-day development you might want to use **desktop**
  platform commands.

We encourage to use `just` commands with configured `oxygengine-ignite`
calls for best experience.

Each of these `just` commands can have an optional last parameter specifying
platform to run command for (defaults to `desktop` when ommitted).

### Day-to-day development

Bake project assets into asset packs for development cycle to use:
```bash
just bake
```

Format project code:
```bash
just format
```

Compile project:
```bash
just dev build
```

Run project (best to run only for `desktop` platform, because `web` is not
configured here at all):
```bash
just dev run
```

Test project (best to run only for `desktop` platform, because `web` might
not be configured here properly):
```bash
just dev test
```

Live development with code recompilation and assets re-baking on change:
```bash
just live
```
Files will be served from: http://localhost:8080.

### Production distribution

Build game binaries in debug mode:
```bash
just prod build debug
```

Build game binaries in release mode:
```bash
just prod build release
```

Package game distribution in debug mode:
```bash
just prod package debug
```

Package game distribution in release mode:
```bash
just prod package release
```
