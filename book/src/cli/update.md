# Update engine version used in your game project

- reinstall `oxygengine-ignite`:
  ```bash
  cargo install oxygengine-ignite --forced
  OXY_UPDATE_PRESETS=1 oxygengine-ignite --help
  ```
- update `oxygengine` dependency version in `Cargo.toml` to point to the latest engine version.
