# Asset pipeline and its tools

Since Oxygengine is highly data-driven, we put most if not all data into assets,
that's where the idea of asset pipeline was born. Asset pipeline is used to take
source files such as images, levels, sounds, basically every file that contains
some static and read-only data your game wants to use, and it converts it to data
format that suits best engine internals needs.

For example you can have many different source image to be rendered but all engine
cares about is the image data and not its source format, so we use asset pipeline
tools to convert them into engine's internal image format, or to even compress
them. Actually, better example can be found with fonts or levels.

In HA (hardware-accelerated) renderer we use SDF-compatible font map images so we
obviously need to bake them either from BMfont generated files or other font
rasterization software. For game levels we use free LDtk level editor so we have
a LDtk asset pipeline tool that takes LDtk project files and bakes images from
tilesets, prefabs with entities from level layers and additional data assets from
used grid layer values.

Asset pipeline tools are just CLI binaries that uses `oxygengine-build-tools`
crate types to read input data passed to your tool, and your tool job is to write
new files to given path. That means, if you need to have support for custom/additional
asset sources, you can easily make an asset pipeline tool for it.

**IMPORTANT:**
When using HA renderer, you have to also install their asset pipeline tools for
asset pipeline to be able to bake assets.
These are:
- `cargo install oxygengine-ha-renderer-tools`

In general, if some engine module requires asset pipeline tools to work, there
is a companion crate named: `<some-engine-module>-tools`.
