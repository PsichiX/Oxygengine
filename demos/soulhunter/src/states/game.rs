use crate::{
    assets::level::{LevelAsset, LevelData, LevelObject},
    components::{animal_kind::AnimalKind, item_kind::ItemKind},
    states::menu::MenuState,
    ui::screens::gui::{GuiProps, GuiRemoteProps},
};
use oxygengine::{prelude::*, user_interface::raui::core::widget::WidgetId};
use std::str::FromStr;

const CELL_SIZE: Scalar = 128.0;

#[derive(Debug)]
pub struct GameState {
    level_path: String,
    animal: AnimalKind,
    ready: bool,
}

impl GameState {
    pub fn new(level_path: String, animal: AnimalKind) -> Self {
        Self {
            level_path,
            animal,
            ready: false,
        }
    }

    fn initialize(&self, world: &mut World) -> Result<bool, StateChange> {
        let camera = entity_find_world("camera", world);
        let tiles = entity_find_world("tiles", world);
        if let (Some(camera), Some(tiles)) = (camera, tiles) {
            let data = match self.load_level_data(world) {
                Some(data) => data,
                None => return Err(StateChange::Swap(Box::new(MenuState))),
            };

            if let Some(transform) = world
                .write_component::<CompositeTransform>()
                .get_mut(camera)
            {
                transform.set_translation(Vec2::new(
                    data.cols as Scalar * CELL_SIZE * 0.5,
                    data.rows as Scalar * CELL_SIZE * 0.5,
                ));
            }
            if let Some(renderable) = world
                .write_component::<CompositeRenderable>()
                .get_mut(tiles)
            {
                renderable.0 = Renderable::Commands(data.build_render_commands(CELL_SIZE));
            }

            let token = world.read_resource::<AppLifeCycle>().current_state_token();
            for (index, cell) in data.cells.iter().enumerate() {
                let col = index % data.cols;
                let row = index / data.cols;
                match cell.object {
                    LevelObject::Star => Self::create_item(world, col, row, ItemKind::Star, token),
                    LevelObject::Shield => {
                        Self::create_item(world, col, row, ItemKind::Shield, token)
                    }
                    LevelObject::PlayerStart => {
                        Self::create_player(world, col, row, self.animal, token)
                    }
                    _ => {}
                }
            }

            if let Some(app) = world
                .write_resource::<UserInterfaceRes>()
                .application_mut("gui")
            {
                let binding = GuiRemoteProps::new_bound(
                    GuiProps {
                        steps: 42,
                        collected_stars: 2,
                        collected_shields: 3,
                    },
                    app.change_notifier(),
                );
                app.send_message(&WidgetId::from_str("gui:gui").unwrap(), binding);
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn load_level_data(&self, world: &World) -> Option<LevelData> {
        let assets = world.read_resource::<AssetsDatabase>();
        if let Some(asset) = assets.asset_by_path(&self.level_path) {
            if let Some(asset) = asset.get::<YamlAsset>() {
                if let Ok(asset) = asset.deserialize::<LevelAsset>() {
                    match asset.build_data() {
                        Ok(data) => return Some(data),
                        Err(error) => oxygengine::error!(
                            "* Level data can't be built from asset: {}\nError: {:?}",
                            self.level_path,
                            error
                        ),
                    }
                } else {
                    oxygengine::error!("* Level asset can't be deserialized: {}", self.level_path);
                }
            } else {
                oxygengine::error!("* Level asset is not YAML: {}", self.level_path);
            }
        } else {
            oxygengine::error!("* Level asset is not loaded: {}", self.level_path);
        }
        None
    }

    fn create_item(world: &mut World, col: usize, row: usize, item: ItemKind, token: StateToken) {
        let image = item.build_image(CELL_SIZE);
        let x = (col as Scalar + 0.5) * CELL_SIZE;
        let y = (row as Scalar + 0.5) * CELL_SIZE;
        world
            .create_entity()
            .with(Tag("world".into()))
            .with(CompositeRenderable(image.into()))
            .with(CompositeTransform::translation([x, y].into()))
            .with(item)
            .with(NonPersistent(token))
            .build();
    }

    fn create_player(
        world: &mut World,
        col: usize,
        row: usize,
        animal: AnimalKind,
        token: StateToken,
    ) {
        let image = animal.build_image(CELL_SIZE);
        let x = (col as Scalar + 0.5) * CELL_SIZE;
        let y = (row as Scalar + 0.5) * CELL_SIZE;
        world
            .create_entity()
            .with(Tag("world".into()))
            .with(CompositeRenderable(image.into()))
            .with(CompositeTransform::translation([x, y].into()))
            .with(animal)
            .with(NonPersistent(token))
            .build();
    }
}

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        self.ready = false;
        world
            .write_resource::<PrefabManager>()
            .instantiate_world("level-scene", world)
            .expect("Cannot instantiate level scene");
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        if !self.ready {
            self.ready = match self.initialize(world) {
                Ok(ready) => ready,
                Err(change) => return change,
            }
        }
        StateChange::None
    }
}
