# Building your project

### Launch live development

With active code recompilation and assets baking in the background on change:
```bash
oxygengine-ignite live
```

Additionally to allow it to start http server to serve your build files in the
browser, run:
```bash
oxygengine-ignite live -- -p 8080
```
files will be served from: http://localhost:8080.

### Build binaries in debug or release mode

Binaries will be put in `/bin` folder.

With debug symbols:
```bash
oxygengine-ignite build
```

Optimized release mode:
```bash
oxygengine-ignite build -p release
```

### Build only the crate

```bash
cargo build
```

### Package application build with assets ready for distribution

Package files will be put in `/dist` folder:
```bash
oxygengine-ignite package
```
this command will run release build, assets pipeline and bundle package.

To produce a debug mode package you have to run:
```bash
oxygengine-ignite package -d
```
