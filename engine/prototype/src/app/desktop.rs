use crate::{app::*, materials::*, systems::render_prototype_stage::*};
// use oxygengine_audio_backend_desktop::prelude::*;
use oxygengine_backend_desktop::prelude::*;
use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;
use oxygengine_input::prelude::*;
use oxygengine_input_device_desktop::prelude::*;
use std::{collections::HashSet, sync::Arc};

pub struct DesktopPrototypeApp {
    pub initial_state: Box<dyn State>,
    pub clear_color: Rgba,
    pub sprite_filtering: ImageFiltering,
    pub view_size: Scalar,
    pub preload_assets: HashSet<String>,
    pub input_mappings: InputMappings,
}

impl DesktopPrototypeApp {
    pub fn new(initial_state: impl State + 'static) -> Self {
        Self {
            initial_state: Box::new(initial_state),
            clear_color: Rgba::gray(0.2),
            sprite_filtering: Default::default(),
            view_size: 1024.0,
            preload_assets: Default::default(),
            input_mappings: Default::default(),
        }
    }
}

impl PrototypeApp for DesktopPrototypeApp {
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

    fn run(self) {
        #[cfg(debug_assertions)]
        logger_setup(DefaultLogger);

        let runner = DesktopAppRunner::new(DesktopAppConfig::default());
        let app = App::build::<LinearPipelineBuilder>()
            .with_bundle(
                oxygengine_core::assets::bundle_installer,
                make_assets(&self.preload_assets),
            )
            .unwrap()
            .with_bundle(
                oxygengine_input::bundle_installer,
                make_inputs(&self.input_mappings),
            )
            .unwrap()
            .with_bundle(
                oxygengine_ha_renderer::bundle_installer,
                make_renderer(runner.context_wrapper(), self.clear_color),
            )
            .unwrap()
            // .with_bundle(oxygengine_audio::bundle_installer, DesktopAudio::default())
            // .unwrap()
            .with_bundle(crate::bundle_installer, |renderables, camera| {
                renderables.sprite_filtering = self.sprite_filtering;
                camera.view_size = self.view_size;
            })
            .unwrap()
            .with_resource(FsStorageEngine::default())
            .with_resource(DesktopAppEvents::default())
            .build::<SequencePipelineEngine, _, _>(
                BootState {
                    next_state: Some(self.initial_state),
                    view_size: self.view_size,
                },
                StandardAppTimer::default(),
            );

        AppRunner::new(app).run(runner).unwrap();
    }
}

fn make_assets(preload: &HashSet<String>) -> (FsFetchEngine, impl FnMut(&mut AssetsDatabase) + '_) {
    (FsFetchEngine::default(), move |database| {
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

fn make_inputs(mappings: &InputMappings) -> impl FnMut(&mut InputController) + '_ {
    |input| {
        input.register(DesktopKeyboardInputDevice::default());
        input.register(DesktopMouseInputDevice::default());
        input.map_config(mappings.to_owned());
    }
}

fn make_renderer(
    context_wrapper: Arc<DesktopContextWrapper>,
    clear_color: Rgba,
) -> HaRendererBundleSetup {
    let interface = DesktopPlatformInterface::with_context_wrapper(context_wrapper);
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
