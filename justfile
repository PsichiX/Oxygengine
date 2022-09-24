list:
  just --list

format-engine:
  cargo fmt --all --manifest-path ./engine/Cargo.toml

format-templates:
  cargo fmt --manifest-path ./templates/desktop-headless-game/Cargo.toml
  cargo fmt --manifest-path ./templates/web-composite-game/Cargo.toml
  cargo fmt --manifest-path ./templates/web-composite-visual-novel-game/Cargo.toml
  cargo fmt --manifest-path ./templates/web-ha-base/Cargo.toml
  cargo fmt --manifest-path ./templates/web-ha-game/Cargo.toml
  cargo fmt --manifest-path ./templates/web-ha-prototype/Cargo.toml

format-demos-wip:
  cargo fmt --manifest-path ./demos/wip/pokemon/Cargo.toml
  cargo fmt --manifest-path ./demos/wip/skinning-testbed/Cargo.toml
  
format-demos:
  just format-demos-wip
  
format:
  just format-engine
  just format-templates
  just format-demos

build-engine:
  cargo build --all --manifest-path ./engine/Cargo.toml

build-templates:
  cd ./templates/desktop-headless-game/ && cargo build
  cd ./templates/web-composite-game/ && cargo build
  cd ./templates/web-composite-visual-novel-game/ && cargo build
  cd ./templates/web-ha-base/ && cargo build
  cd ./templates/web-ha-game/ && cargo build

build-demos-wip:
  cd ./demos/wip/pokemon/ && cargo build
  cd ./demos/wip/skinning-testbed/ && cargo build

build-demos:
  just build-demos-wip

build:
  just build-engine
  just build-templates
  just build-demos

test-engine:
  cargo test --all --manifest-path ./engine/Cargo.toml

test:
  just test-engine

clippy-engine:
  cargo clippy --all --manifest-path ./engine/Cargo.toml

clippy-templates:
  cd ./templates/desktop-headless-game/ && cargo clippy
  cd ./templates/web-composite-game/ && cargo clippy
  cd ./templates/web-composite-visual-novel-game/ && cargo clippy
  cd ./templates/web-ha-base/ && cargo clippy
  cd ./templates/web-ha-game/ && cargo clippy
  cd ./templates/web-ha-prototype/ && cargo clippy

clippy-demos-wip:
  cd ./demos/wip/pokemon/ && cargo clippy
  cd ./demos/wip/skinning-testbed/ && cargo clippy

clippy-demos:
  just clippy-demos-wip

clippy:
  just clippy-engine
  just clippy-templates
  just clippy-demos

checks-engine:
  just build-engine
  just test-engine
  just clippy-engine

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

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

make-presets-pack:
  OXY_DONT_AUTO_UPDATE=1 cargo run --manifest-path ./engine/ignite/Cargo.toml -- pipeline

update-ignite-presets:
  just make-presets-pack
  OXY_UPDATE_PRESETS=1 OXY_UPDATE_FILE=./target/oxygengine-presets.pack cargo run --manifest-path ./engine/ignite/Cargo.toml -- --help

install-ignite:
  cargo install --path ./engine/ignite

install-tools-composite-renderer:
  cargo install --path ./engine/composite-renderer-tools

install-tools-ha-renderer:
  cargo install --path ./engine/ha-renderer-tools

install-tools:
  just install-ignite
  just install-tools-composite-renderer
  just install-tools-ha-renderer

list-outdated:
  cd ./engine && cargo outdated -R -w

book:
  mdbook build book
  mdbook test book -L ./target/debug/deps

update-engine:
  cargo update --manifest-path ./engine/Cargo.toml --workspace

update-templates:
  cd ./templates/desktop-headless-game/ && cargo update
  cd ./templates/web-composite-game/ && cargo update
  cd ./templates/web-composite-visual-novel-game/ && cargo update
  cd ./templates/web-ha-base/ && cargo update
  cd ./templates/web-ha-game/ && cargo update
  cd ./templates/web-ha-prototype/ && cargo update

update-demos-wip:
  cd ./demos/wip/pokemon/ && cargo update
  cd ./demos/wip/skinning-testbed/ && cargo update

update-demos:
  just update-demos-wip

update:
  just update-engine
  just update-templates
  just update-demos

publish:
  cargo publish --no-verify --manifest-path ./engine/ignite-types/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ignite-derive/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ignite/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/build-tools/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/core/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/backend-web/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/utils/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ai/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/animation/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/audio/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/audio-backend-web/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/composite-renderer/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/composite-renderer-backend-web/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/composite-renderer-tools/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ha-renderer/Cargo.toml
  sleep 20
  cargo publish --no-verify --manifest-path ./engine/ha-renderer-tools/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/ha-renderer-debugger/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/editor-tools/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/editor-tools-backend-web/Cargo.toml
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
  cargo publish --no-verify --manifest-path ./engine/user-interface/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/visual-novel/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/overworld/Cargo.toml
  sleep 15
  cargo publish --no-verify --manifest-path ./engine/prototype/Cargo.toml
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

new-project PATH NAME:
  just update-ignite-presets
  cargo run --manifest-path ./engine/ignite/Cargo.toml -- new --dont-build -d {{PATH}} {{NAME}}
