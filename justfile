list:
  just --list

format:
  cargo fmt --all --manifest-path ./engine/Cargo.toml
  cargo fmt --manifest-path ./templates/desktop-headless-game/Cargo.toml
  cargo fmt --manifest-path ./templates/web-composite-game/Cargo.toml
  cargo fmt --manifest-path ./templates/web-composite-visual-novel-game/Cargo.toml
  cargo fmt --manifest-path ./templates/web-ha-game/Cargo.toml

checks:
  cargo build --all --manifest-path ./engine/Cargo.toml
  cargo clippy --all --manifest-path ./engine/Cargo.toml
  cargo test --all --manifest-path ./engine/Cargo.toml

  cd ./templates/desktop-headless-game/ && cargo build
  cd ./templates/desktop-headless-game/ && cargo clippy

  cd ./templates/web-composite-game/ && cargo build
  cd ./templates/web-composite-game/ && cargo clippy

  cd ./templates/web-composite-visual-novel-game/ && cargo build
  cd ./templates/web-composite-visual-novel-game/ && cargo clippy

  cd ./templates/web-ha-game/ && cargo build
  cd ./templates/web-ha-game/ && cargo clippy

make-presets-pack:
  cargo run --manifest-path ./engine/Cargo.toml --package oxygengine-ignite -- pipeline

list-outdated:
  cargo outdated -R -w -m ./engine/Cargo.toml

update:
  cargo update --manifest-path ./engine/_/Cargo.toml
  cargo update --manifest-path ./engine/animation/Cargo.toml
  cargo update --manifest-path ./engine/audio/Cargo.toml
  cargo update --manifest-path ./engine/audio-backend-web/Cargo.toml
  cargo update --manifest-path ./engine/backend-web/Cargo.toml
  cargo update --manifest-path ./engine/build-tools/Cargo.toml
  cargo update --manifest-path ./engine/composite-renderer/Cargo.toml
  cargo update --manifest-path ./engine/composite-renderer-backend-web/Cargo.toml
  cargo update --manifest-path ./engine/composite-renderer-tools/Cargo.toml
  cargo update --manifest-path ./engine/core/Cargo.toml
  cargo update --manifest-path ./engine/editor-tools/Cargo.toml
  cargo update --manifest-path ./engine/ha-renderer/Cargo.toml
  cargo update --manifest-path ./engine/ha-renderer-tools/Cargo.toml
  cargo update --manifest-path ./engine/ignite/Cargo.toml
  cargo update --manifest-path ./engine/ignite-derive/Cargo.toml
  cargo update --manifest-path ./engine/ignite-types/Cargo.toml
  cargo update --manifest-path ./engine/input/Cargo.toml
  cargo update --manifest-path ./engine/input-device-web/Cargo.toml
  cargo update --manifest-path ./engine/integration-ow-ha/Cargo.toml
  cargo update --manifest-path ./engine/integration-p2d-cr/Cargo.toml
  cargo update --manifest-path ./engine/integration-ui-cr/Cargo.toml
  cargo update --manifest-path ./engine/integration-ui-ha/Cargo.toml
  cargo update --manifest-path ./engine/integration-vn-ui/Cargo.toml
  cargo update --manifest-path ./engine/navigation/Cargo.toml
  cargo update --manifest-path ./engine/network/Cargo.toml
  cargo update --manifest-path ./engine/network-backend-desktop/Cargo.toml
  cargo update --manifest-path ./engine/network-backend-native/Cargo.toml
  cargo update --manifest-path ./engine/network-backend-web/Cargo.toml
  cargo update --manifest-path ./engine/overworld/Cargo.toml
  cargo update --manifest-path ./engine/physics-2d/Cargo.toml
  cargo update --manifest-path ./engine/procedural/Cargo.toml
  cargo update --manifest-path ./engine/script-flow/Cargo.toml
  cargo update --manifest-path ./engine/user-interface/Cargo.toml
  cargo update --manifest-path ./engine/utils/Cargo.toml
  cargo update --manifest-path ./engine/visual-novel/Cargo.toml

  cd ./templates/desktop-headless-game/ && cargo update
  cd ./templates/web-composite-game/ && cargo update
  cd ./templates/web-composite-visual-novel-game/ && cargo update
  cd ./templates/web-ha-game/ && cargo update

publish:
  cargo publish --no-verify --manifest-path ./engine/core/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/backend-web/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/utils/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/animation/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/audio/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/audio-backend-web/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/build-tools/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/composite-renderer/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/composite-renderer-backend-web/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/composite-renderer-tools/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ha-renderer/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ha-renderer-tools/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/editor-tools/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ignite-types/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ignite-derive/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ignite/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/input/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/input-device-web/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/navigation/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/network/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/network-backend-desktop/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/network-backend-native/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/network-backend-web/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/physics-2d/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/procedural/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/script-flow/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/user-interface/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/visual-novel/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/overworld/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/integration-ow-ha/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/integration-p2d-cr/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/integration-ui-cr/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/integration-ui-ha/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/integration-vn-ui/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/_/Cargo.toml
