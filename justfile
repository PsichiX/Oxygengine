list:
  just --list

format:
  cargo fmt --all --manifest-path ./Cargo.toml

build-engine:
  cargo build --manifest-path ./Cargo.toml

build-templates:
  cd ./templates/base/ && cargo build --manifest-path ./platforms/desktop/Cargo.toml
  cd ./templates/game/ && cargo build --manifest-path ./platforms/desktop/Cargo.toml
  cd ./templates/prototype/ && cargo build --manifest-path ./platforms/desktop/Cargo.toml

build-demos-wip:
  cd ./demos/wip/rig-testbed/ && cargo build --manifest-path ./platforms/desktop/Cargo.toml

build-demos:
  just build-demos-wip

build:
  just build-engine
  just build-templates
  just build-demos

test-engine:
  cargo test --manifest-path ./Cargo.toml

test:
  just test-engine

clippy-engine:
  cargo clippy --manifest-path ./Cargo.toml

clippy-templates:
  cd ./templates/base/ && cargo clippy --manifest-path ./platforms/desktop/Cargo.toml
  cd ./templates/game/ && cargo clippy --manifest-path ./platforms/desktop/Cargo.toml
  cd ./templates/prototype/ && cargo clippy --manifest-path ./platforms/desktop/Cargo.toml

clippy-demos-wip:
  cd ./demos/wip/rig-testbed/ && cargo clippy --manifest-path ./platforms/desktop/Cargo.toml

clippy-demos:
  just clippy-demos-wip

clippy:
  just clippy-engine
  just clippy-templates
  just clippy-demos

checks-engine:
  just build-engine
  just clippy-engine
  just test-engine

checks-templates:
  just build-templates
  just clippy-templates

checks-demos:
  just build-demos
  just clippy-demos

checks:
  just checks-engine
  just checks-templates
  just checks-demos

clean:
  find . -name target -type d -exec rm -r {} +
  just remove-lockfiles

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

make-presets-pack:
  OXY_DONT_AUTO_UPDATE=1 cargo run --manifest-path ./engine/ignite/Cargo.toml -- pipeline

update-ignite-presets:
  just make-presets-pack
  OXY_UPDATE_PRESETS=1 OXY_UPDATE_FILE=./target/oxygengine-presets.pack cargo run --manifest-path ./engine/ignite/Cargo.toml -- --help

install-ignite:
  cargo install --path ./engine/ignite

install-tools-ha-renderer:
  cargo install --path ./engine/ha-renderer-tools

install-tools:
  just install-ignite
  just install-tools-ha-renderer

list-outdated:
  cd ./engine && cargo outdated -R -w

book:
  mdbook build book
  mdbook test book -L ./target/debug/deps

book-live:
  mdbook serve book -o

update-engine:
  cargo update --manifest-path ./Cargo.toml --workspace --aggressive

update-templates:
  cd ./templates/base/ && cargo update --aggressive --manifest-path ./platforms/desktop/Cargo.toml
  cd ./templates/game/ && cargo update --aggressive --manifest-path ./platforms/desktop/Cargo.toml
  cd ./templates/prototype/ && cargo update --aggressive --manifest-path ./platforms/desktop/Cargo.toml

update-demos-wip:
  cd ./demos/wip/rig-testbed/ && cargo update --aggressive --manifest-path ./platforms/desktop/Cargo.toml

update-demos:
  just update-demos-wip

update:
  just update-engine
  just update-templates
  just update-demos

publish:
  cargo publish --no-verify --manifest-path ./engine/ignite/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/build-tools/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/core/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/backend-web/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/utils/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/ai/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/animation/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/audio/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/audio-backend-web/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/ha-renderer/Cargo.toml
  sleep 20
  cargo publish --no-verify --manifest-path ./engine/ha-renderer-tools/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/ha-renderer-debugger/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/editor-tools/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/editor-tools-backend-web/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/input/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/input-device-web/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/navigation/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/network/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/network-backend-desktop/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/network-backend-native/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/network-backend-web/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/physics-2d/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/procedural/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/user-interface/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/visual-novel/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/overworld/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/prototype/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/integration-ow-ha/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/integration-p2d-cr/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/integration-ui-cr/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/integration-ui-ha/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/integration-vn-ui/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./engine/_/Cargo.toml

new-project PATH NAME:
  just update-ignite-presets
  cargo run --manifest-path ./engine/ignite/Cargo.toml -- new --dont-build -d {{PATH}} {{NAME}}
