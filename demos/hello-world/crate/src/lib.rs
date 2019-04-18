extern crate oxygengine;

use oxygengine::{
    backend::web::*,
    composite_renderer::{component::*, composite_renderer::*, math::*},
    core::{
        assets::{database::AssetsDatabase, protocols::text::TextAsset},
        fetch::engines::map::MapFetchEngine,
    },
    prelude::*,
};
use std::{borrow::Cow, collections::HashMap};
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

struct DebugSystem;

impl<'s> System<'s> for DebugSystem {
    type SystemData = ReadExpect<'s, PlatformCompositeRenderer>;

    fn run(&mut self, renderer: Self::SystemData) {
        console_log!("{:#?}", renderer.state().stats());
    }
}

struct LoadingState;

impl State for LoadingState {
    fn on_enter(&mut self, world: &mut World) {
        let assets = &mut world.write_resource::<AssetsDatabase>();
        assets.load("set://assets.txt").unwrap();
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let assets = &world.read_resource::<AssetsDatabase>();
        if assets.is_ready() {
            StateChange::Swap(Box::new(MainState))
        } else {
            StateChange::Quit
        }
    }
}

struct MainState;

impl State for MainState {
    fn on_enter(&mut self, world: &mut World) {
        let text = {
            let assets = &world.read_resource::<AssetsDatabase>();
            assets
                .asset_by_path("txt://a.txt")
                .unwrap()
                .get::<TextAsset>()
                .unwrap()
                .get()
                .to_owned()
        };

        world
            .create_entity()
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::blue(),
                rect: [100.0, 100.0, 500.0, 100.0].into(),
            })))
            .with(CompositeTransform::default())
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(Renderable::Text(Text {
                color: Color::yellow(),
                font: Cow::from("Verdana"),
                align: TextAlign::Center,
                text: text.into(),
                position: [100.0 + 250.0, 100.0 + 50.0 + 12.0].into(),
                size: 24.0,
            })))
            .with(CompositeTransform::default())
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(Renderable::Path(Path {
                color: Color::white(),
                elements: vec![
                    PathElement::MoveTo([300.0, 300.0].into()),
                    PathElement::LineTo([400.0, 300.0].into()),
                    PathElement::QuadraticCurveTo([400.0, 400.0].into(), [300.0, 400.0].into()),
                    PathElement::LineTo([300.0, 300.0].into()),
                ],
            })))
            .with(CompositeTransform::default())
            .with(CompositeRenderableStroke(5.0))
            .build();
    }
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let mut assets = HashMap::new();
    assets.insert(
        "set://assets.txt".to_owned(),
        br#"
            txt://a.txt
            txt://b.txt
        "#
        .to_vec(),
    );
    assets.insert("txt://a.txt".to_owned(), b"AAA".to_vec());
    assets.insert("txt://b.txt".to_owned(), b"BBB".to_vec());

    let app = App::build()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            // (WebFetchEngine::default(), |_| {}),
            (MapFetchEngine::new(assets), |_| {}),
        )
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state("screen", RenderState::new(Some(Color::black()))),
        )
        .with_system(DebugSystem, "debug", &[])
        .build(LoadingState);

    AppRunner::new(app).run::<PlatformAppRunner, _>()?;

    Ok(())
}

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
