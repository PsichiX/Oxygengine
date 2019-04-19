extern crate oxygengine;

use oxygengine::{
    backend::web::*,
    composite_renderer::{
        component::*, composite_renderer::*, math::*, png_image_asset_protocol::*,
    },
    core::assets::{database::AssetsDatabase, protocols::prelude::*},
    prelude::*,
};
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
        world
            .write_resource::<AssetsDatabase>()
            .load("set://assets.txt")
            .expect("cannot load `assets.txt`");
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let assets = &world.read_resource::<AssetsDatabase>();
        if assets.is_ready() {
            StateChange::Swap(Box::new(MainState))
        } else {
            StateChange::None
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
                .expect("`a.txt` is not loaded")
                .get::<TextAsset>()
                .expect("`a.txt` is not TextAsset")
                .get()
                .to_owned()
        };

        world
            .create_entity()
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::rgba(128, 0, 0, 128),
                rect: [100.0, 100.0, 500.0, 100.0].into(),
            })))
            .with(CompositeTransform::default())
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(
                Text {
                    color: Color::yellow(),
                    font: "Verdana".into(),
                    align: TextAlign::Center,
                    text: text.into(),
                    position: [100.0 + 250.0, 100.0 + 50.0 + 12.0].into(),
                    size: 24.0,
                }
                .into(),
            ))
            .with(CompositeTransform::default())
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(
                Path {
                    color: Color::magenta().a(192),
                    elements: vec![
                        PathElement::MoveTo([300.0, 300.0].into()),
                        PathElement::LineTo([400.0, 300.0].into()),
                        PathElement::QuadraticCurveTo([400.0, 400.0].into(), [300.0, 400.0].into()),
                        PathElement::LineTo([300.0, 300.0].into()),
                    ],
                }
                .into(),
            ))
            .with(CompositeTransform::default())
            .with(CompositeRenderableStroke(5.0))
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(Image::new("web.png").into()))
            .with(CompositeTransform::scale([0.5, 0.5].into()))
            .with(CompositeRenderDepth(-1.0))
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(Image::new("logo.png").into()))
            .with(
                CompositeTransform::translation([50.0, 100.0].into())
                    .with_scale([0.2, 0.2].into())
                    .with_rotation(std::f32::consts::PI * -0.15),
            )
            .with(CompositeRenderDepth(1.0))
            .build();
    }
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let app = App::build()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                assets.register(BinaryAssetProtocol);
                assets.register(TextAssetProtocol);
                assets.register(SetAssetProtocol);
                assets.register(PngImageAssetProtocol);
            }),
        )
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state("screen", RenderState::new(Some(Color::black()))),
        )
        // .with_system(DebugSystem, "debug", &[])
        .build(LoadingState, WebAppTimer::default());

    AppRunner::new(app).run::<PlatformAppRunner, _>()?;

    Ok(())
}

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
