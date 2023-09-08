use crate::{game::GameState, nodes::enemy::EnemyNode};
use oxygengine::prelude::{
    intuicio::derive::*,
    intuicio::{core as intuicio_core, data as intuicio_data, prelude::*},
    *,
};

const GAME_OVER_NORMAL: &str = "Game Over!";
const GAME_OVER_HOVER: &str = "Click to restart";

#[derive(IntuicioStruct, Debug, Default)]
#[intuicio(name = "GuiNode", module_name = "game")]
pub struct GuiNode {
    #[intuicio(ignore)]
    pub game_over: bool,
    #[intuicio(ignore)]
    pub score: usize,
}

impl GuiNode {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::event_draw_gui__define_function(registry));
        registry.add_function(Self::signal_game_over__define_function(registry));
        registry.add_function(Self::signal_game_start__define_function(registry));
        registry.add_function(Self::signal_score_increased__define_function(registry));
    }

    fn popup_widget<T>(
        mut gui: Gui<T>,
        message_normal: &str,
        message_hovers: &str,
        clicked: bool,
    ) -> bool {
        gui.button(move |mut gui, hovers| {
            gui.sprite_sliced(
                crate::assets::image::PANEL,
                Rgba::white(),
                rect(0.4, 0.4, 0.2, 0.2),
                (16.0, 16.0, 16.0, 16.0),
                false,
            );
            gui.cut_left(gui.layout().h, |mut gui| {
                gui.sprite(crate::assets::image::LOGO, Rgba::white());
            });
            gui.margin(12.0, 0.0, 0.0, 0.0, |mut gui| {
                gui.text(
                    crate::assets::font::ROBOTO,
                    if hovers {
                        message_hovers
                    } else {
                        message_normal
                    },
                    30.0,
                    Rgba::white(),
                    0.5,
                );
            });
            clicked
        })
    }
}

#[intuicio_methods(module_name = "game")]
impl GuiNode {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_draw_gui(
        this: &mut Self,
        _transform: &mut HaTransform,
        universe: &Universe,
        _entity: Entity,
    ) {
        let world = &*universe.world();
        let camera = &*universe.expect_resource::<Camera>();
        let inputs = &*universe.expect_resource::<InputController>();
        let renderables = &mut *universe.expect_resource_mut::<Renderables>();
        let mut spawns = universe.expect_resource_mut::<ScriptedNodesSpawns>();
        let mut commands = universe.expect_resource_mut::<UniverseCommands>();
        let pointer = Vec2::from(inputs.multi_axis_or_default(["mouse-x", "mouse-y"]));
        let clicked = inputs.trigger_or_default("mouse-action").is_pressed();

        gui(pointer, camera, renderables, &mut (), |mut gui| {
            gui.margin(16.0, 16.0, 16.0, 16.0, |mut gui| {
                gui.text(
                    crate::assets::font::ROBOTO,
                    &format!("Score: {}", this.score),
                    40.0,
                    Rgba::white(),
                    0.0,
                );

                if this.game_over {
                    gui.freeform_aligned(
                        vec2(300.0, 100.0),
                        vec2(0.5, 0.0),
                        vec2(0.5, 0.0),
                        |mut gui| {
                            if Self::popup_widget(
                                gui.gui(),
                                GAME_OVER_NORMAL,
                                GAME_OVER_HOVER,
                                clicked,
                            ) {
                                spawns.spawn_root(GameState::create_player(vec2(0.0, 0.0)));
                                for entity in
                                    ScriptedNodeEntity::find_all_of_type_raw::<EnemyNode>(world)
                                {
                                    commands.schedule(DespawnEntity(entity));
                                }
                            }
                        },
                    );
                }
            });
        });
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn signal_game_over(this: &mut Self, _entity: Entity) {
        this.game_over = true;
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn signal_game_start(this: &mut Self, _entity: Entity) {
        this.game_over = false;
        this.score = 0;
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn signal_score_increased(this: &mut Self, _entity: Entity) {
        this.score += 1;
    }
}
