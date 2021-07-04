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

    fn initialize(&self, universe: &mut Universe) -> Result<bool, StateChange> {
        let hierarchy = universe.expect_resource::<Hierarchy>();
        let camera = hierarchy.entity_by_name("camera");
        let tiles = hierarchy.entity_by_name("tiles");
        if let (Some(camera), Some(tiles)) = (camera, tiles) {
            let data = match self.load_level_data(universe) {
                Some(data) => data,
                None => return Err(StateChange::Swap(Box::new(MenuState))),
            };

            if let Ok(mut transform) = universe
                .world()
                .query_one::<&mut CompositeTransform>(camera)
            {
                if let Some(transform) = transform.get() {
                    transform.set_translation(Vec2::new(
                        data.cols as Scalar * CELL_SIZE * 0.5,
                        data.rows as Scalar * CELL_SIZE * 0.5,
                    ));
                }
            }
            if let Ok(mut renderable) = universe
                .world()
                .query_one::<&mut CompositeRenderable>(tiles)
            {
                if let Some(renderable) = renderable.get() {
                    renderable.0 = Renderable::Commands(data.build_render_commands(CELL_SIZE));
                }
            }

            let token = universe
                .expect_resource::<AppLifeCycle>()
                .current_state_token();
            for (index, cell) in data.cells.iter().enumerate() {
                let col = index % data.cols;
                let row = index / data.cols;
                match cell.object {
                    LevelObject::Star => Self::create_item(
                        &mut universe.world_mut(),
                        col,
                        row,
                        ItemKind::Star,
                        token,
                    ),
                    LevelObject::Shield => Self::create_item(
                        &mut universe.world_mut(),
                        col,
                        row,
                        ItemKind::Shield,
                        token,
                    ),
                    LevelObject::PlayerStart => {
                        Self::create_player(&mut universe.world_mut(), col, row, self.animal, token)
                    }
                    _ => {}
                }
            }

            if let Some(app) = universe
                .expect_resource_mut::<UserInterface>()
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

    fn load_level_data(&self, universe: &Universe) -> Option<LevelData> {
        let assets = universe.expect_resource::<AssetsDatabase>();
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
        world.spawn((
            Tag("world".into()),
            CompositeRenderable(image.into()),
            CompositeTransform::translation([x, y].into()),
            item,
            NonPersistent(token),
        ));
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
        world.spawn((
            Tag("world".into()),
            CompositeRenderable(image.into()),
            CompositeTransform::translation([x, y].into()),
            animal,
            NonPersistent(token),
        ));
    }
}

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        self.ready = false;
        universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("level-scene", universe)
            .expect("Cannot instantiate level scene");
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        if !self.ready {
            self.ready = match self.initialize(universe) {
                Ok(ready) => ready,
                Err(change) => return change,
            }
        }
        StateChange::None
    }
}
