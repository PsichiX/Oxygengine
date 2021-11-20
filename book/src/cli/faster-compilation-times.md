# Speed up compilation times for new projects

Best use case for gamejams and quick feature prototypes.

- install SCCACHE, a tool for caching and sharing prebuilt dependencies between
  multiple game projects (https://github.com/mozilla/sccache):
  ```bash
  cargo install sccache
  ```
- add these lines to the `Cargo.toml`:
  ```toml
  [package.metadata]
  # path to the sccache binary
  sccache_bin = "sccache.exe"
  # path to the sccache cache directory
  sccache_dir = "D:\\sccache"
  ```

From now on you will have to wait for full long engine build only once, for any
other new game project you create, it will perform first compilation in matter
of minute, not 20.
