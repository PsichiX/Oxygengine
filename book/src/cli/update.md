# Update engine version used in your game project

- reinstall `oxygengine-ignite`:
  ```bash
  cargo install oxygengine-ignite --forced
  OXY_UPDATE_PRESETS=1 oxygengine-ignite --help
  ```
- update `oxygengine` version either in `Cargo.toml` or by calling: `cargo update`
- upgrading from versions before 0.12.0 requires to create new project with
  latest ignite tool, then copy by hand your source files to the new project
  sources, as well as put assets from old project `/static/assets` directory
  into new project `/assets` directory, preferably using new way of assets
  directory structure (so you won't have to modify `pipeline.json` file to much).
