use oxygengine::prelude::{
    intuicio::derive::*,
    intuicio::{core as intuicio_core, data as intuicio_data, prelude::*},
    *,
};

#[derive(IntuicioStruct, Debug, Default)]
#[intuicio(name = "GameUi", module_name = "game")]
pub struct GameUi;

impl GameUi {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::event_draw_gui__define_function(registry));
    }

    pub fn draw(camera: &Camera, inputs: &InputController, renderables: &mut Renderables) {
        let pointer = Vec2::from(inputs.multi_axis_or_default(["mouse-x", "mouse-y"]));

        gui(pointer, camera, renderables, &mut (), |mut gui| {
            gui.margin(16.0, 16.0, 16.0, 16.0, |mut gui| {
                gui.cut_bottom(75.0, |mut gui| {
                    Self::message_panel_widget(
                        gui.gui(),
                        crate::assets::image::LOGO,
                        "Welcome to Oxygengine's prototyping module, let's have some fun!",
                        "Use WSAD keys to move Ferris around!",
                    );
                });
            });
        });
    }

    fn message_panel_widget<T>(
        mut gui: Gui<T>,
        avatar: &str,
        message_normal: &str,
        message_hovers: &str,
    ) {
        gui.button(move |mut gui, hovers| {
            gui.sprite_sliced(
                crate::assets::image::PANEL,
                Rgba::white(),
                rect(0.4, 0.4, 0.2, 0.2),
                (16.0, 16.0, 16.0, 16.0),
                false,
            );
            gui.cut_left(gui.layout().h, |mut gui| {
                Self::avatar_widget(gui.gui(), avatar, hovers);
            });
            gui.text(
                crate::assets::font::ROBOTO,
                if hovers {
                    message_hovers
                } else {
                    message_normal
                },
                20.0,
                Rgba::white(),
                0.0,
            );
            false
        });
    }

    fn avatar_widget<T>(mut gui: Gui<T>, image: &str, hovers: bool) {
        if hovers {
            gui.clip(|mut gui| {
                gui.scale(1.25, 0.5, |mut gui| {
                    gui.sprite(image, Rgba::white());
                });
            });
        } else {
            gui.sprite(image, Rgba::white());
        };
    }
}

#[intuicio_methods(module_name = "game")]
impl GameUi {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_draw_gui(
        _this: &mut Self,
        _transform: &mut HaTransform,
        camera: &Camera,
        inputs: &InputController,
        renderables: &mut Renderables,
    ) {
        Self::draw(camera, inputs, renderables);
    }
}
