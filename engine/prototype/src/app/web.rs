use crate::{app::*, materials::*, systems::render_prototype_stage::*};
use oxygengine_audio_backend_web::prelude::*;
use oxygengine_backend_web::prelude::*;
use oxygengine_core::prelude::*;
use oxygengine_editor_tools_backend_web::prelude::*;
use oxygengine_ha_renderer::prelude::*;
use oxygengine_input::prelude::*;
use oxygengine_input_device_web::prelude::*;
use oxygengine_nodes::*;
use std::collections::HashSet;

pub struct WebPrototypeApp {
    pub initial_state: Box<dyn State>,
    pub canvas_id: String,
    pub clear_color: Rgba,
    pub sprite_filtering: ImageFiltering,
    pub view_size: Scalar,
    pub preload_assets: HashSet<String>,
    pub input_mappings: InputMappings,
    pub nodes: ScriptedNodes,
    pub scripting_registry: Registry,
}

impl WebPrototypeApp {
    pub fn new(initial_state: impl State + 'static) -> Self {
        Self {
            initial_state: Box::new(initial_state),
            canvas_id: "screen".to_owned(),
            clear_color: Rgba::gray(0.2),
            sprite_filtering: Default::default(),
            view_size: 1024.0,
            preload_assets: Default::default(),
            input_mappings: Default::default(),
            nodes: Default::default(),
            scripting_registry: Registry::default().with_basic_types(),
        }
    }

    pub fn canvas_id(mut self, value: impl ToString) -> Self {
        self.canvas_id = value.to_string();
        self
    }
}

impl PrototypeApp for WebPrototypeApp {
    fn clear_color(mut self, value: Rgba) -> Self {
        self.clear_color = value;
        self
    }

    fn sprite_filtering(mut self, value: ImageFiltering) -> Self {
        self.sprite_filtering = value;
        self
    }

    fn view_size(mut self, value: Scalar) -> Self {
        self.view_size = value;
        self
    }

    fn preload_asset(mut self, path: impl ToString) -> Self {
        self.preload_assets.insert(path.to_string());
        self
    }

    fn input_mappings(mut self, mappings: InputMappings) -> Self {
        self.input_mappings = mappings;
        self
    }

    fn nodes(mut self, nodes: ScriptedNodes) -> Self {
        self.nodes = nodes;
        self
    }

    fn scripting_registry(mut self, registry: Registry) -> Self {
        self.scripting_registry = registry;
        self
    }

    fn run(self) {
        #[cfg(feature = "console_error_panic_hook")]
        #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

        #[cfg(debug_assertions)]
        logger_setup(WebLogger);

        let app = App::build::<LinearPipelineBuilder>()
            .with_bundle(
                oxygengine_core::assets::bundle_installer,
                make_assets(&self.preload_assets),
            )
            .unwrap()
            .with_bundle(
                oxygengine_core::scripting::bundle_installer,
                self.scripting_registry,
            )
            .unwrap()
            .with_bundle(
                oxygengine_input::bundle_installer,
                make_inputs(&self.canvas_id, &self.input_mappings),
            )
            .unwrap()
            .with_bundle(
                oxygengine_ha_renderer::bundle_installer,
                make_renderer(&self.canvas_id, self.clear_color),
            )
            .unwrap()
            .with_bundle(
                oxygengine_ha_renderer_debugger::bundle_installer,
                WebBroadcastChannel::new("OxygengineHARD"),
            )
            .unwrap()
            .with_bundle(oxygengine_audio::bundle_installer, WebAudio::default())
            .unwrap()
            .with_bundle(oxygengine_nodes::bundle_installer, self.nodes)
            .unwrap()
            .with_bundle(crate::bundle_installer, |renderables, camera| {
                renderables.sprite_filtering = self.sprite_filtering;
                camera.view_size = self.view_size;
            })
            .unwrap()
            .with_resource(WebStorageEngine)
            .build::<SequencePipelineEngine, _, _>(
                BootState {
                    next_state: Some(self.initial_state),
                    view_size: self.view_size,
                },
                WebAppTimer::default(),
            );

        AppRunner::new(app).run(WebAppRunner).unwrap();
    }
}

fn make_assets(
    preload: &HashSet<String>,
) -> (WebFetchEngine, impl FnMut(&mut AssetsDatabase) + '_) {
    (WebFetchEngine::default(), move |database| {
        #[cfg(debug_assertions)]
        database.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
        oxygengine_ha_renderer::protocols_installer(database);
        oxygengine_audio::protocols_installer(database);

        database.insert(Asset::new(
            "material",
            "@material/graph/prototype/sprite",
            MaterialAsset::Graph {
                default_values: Default::default(),
                draw_options: MaterialDrawOptions::transparent(),
                content: default_prototype_sprite_material_graph(),
            },
        ));

        for path in preload {
            let _ = database.load(path);
        }
    })
}

fn make_inputs(canvas_id: &str, mappings: &InputMappings) -> impl FnMut(&mut InputController) + '_ {
    |input| {
        input.register(WebKeyboardInputDevice::new(get_event_target_document()));
        input.register(WebMouseInputDevice::new(get_event_target_by_id(canvas_id)));
        input.register(WebTouchInputDevice::new(get_event_target_by_id(canvas_id)));
        input.map_config(mappings.to_owned());
    }
}

fn make_renderer(canvas_id: &str, clear_color: Rgba) -> HaRendererBundleSetup {
    let interface =
        WebPlatformInterface::with_canvas_id(canvas_id, WebContextOptions::default()).unwrap();
    let mut renderer = HaRenderer::new(interface)
        .with_stage::<RenderPrototypeStage>("prototype")
        .with_pipeline(
            "prototype",
            PipelineDescriptor::default()
                .render_target("main", RenderTargetDescriptor::Main)
                .stage(
                    StageDescriptor::new("prototype")
                        .render_target("main")
                        .domain("@material/domain/surface/flat")
                        .clear_settings(ClearSettings {
                            color: Some(clear_color),
                            depth: false,
                            stencil: false,
                        }),
                ),
        );

    #[cfg(debug_assertions)]
    renderer.set_error_reporter(LoggerHaRendererErrorReporter);
    HaRendererBundleSetup::new(renderer)
}
