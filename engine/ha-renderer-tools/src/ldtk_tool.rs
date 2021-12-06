use ldtk_rust::*;
use oxygengine_build_tools::AssetPipelineInput;
use oxygengine_core::{
    ecs::components::{Name, NonPersistentPrefabProxy, Tag},
    prefab::{Prefab, PrefabScene, PrefabSceneEntity, PrefabSceneEntityData, PrefabValue},
    Scalar,
};
use oxygengine_ha_renderer::{
    asset_protocols::{atlas::*, image::*, tilemap::*},
    components::{
        camera::*, gizmo::*, material_instance::*, mesh_instance::*, sprite_animation_instance::*,
        tilemap_instance::*, transform::*, virtual_image_uniforms::*, visibility::*,
    },
    ha_renderer::*,
    image::*,
    material::*,
    math::*,
    mesh::*,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs::{copy, create_dir_all, read_to_string, write},
    io::Error,
    path::{Path, PathBuf},
};

const DEFAULT_SPRITE_MATERIAL_ASSET: &str = "@material/graph/surface/flat/texture";
const DEFAULT_SPRITE_UNIFORMS_MATERIAL_ASSET: &str =
    "@material/graph/surface/flat/virtual-uniform-texture";
const DEFAULT_SPRITE_MESH_ASSET: &str = "@mesh/surface/quad/pt";
const DEFAULT_SPRITE_IMAGE: &str = "Uniforms";

#[derive(Debug, Clone, Deserialize)]
struct Params {
    pub input_projects: Vec<PathBuf>,
    #[serde(default)]
    pub output: PathBuf,
    #[serde(default)]
    pub assets_path_prefix: String,
    #[serde(default)]
    pub image_filtering: ImageFiltering,
    #[serde(default)]
    pub tile_margin: Vec2,
    #[serde(default)]
    pub material_name: Option<String>,
    #[serde(default)]
    pub image_folder_name: Option<String>,
    #[serde(default)]
    pub atlas_folder_name: Option<String>,
    #[serde(default)]
    pub prefab_folder_name: Option<String>,
    #[serde(default)]
    pub data_folder_name: Option<String>,
    #[serde(default)]
    pub rules: Vec<ComponentRule>,
}

fn main() -> Result<(), Error> {
    let (source, destination, params) = AssetPipelineInput::<Params>::consume().unwrap();
    let output = destination.join(&params.output);
    ensure_path(&output);
    let rules = params
        .rules
        .iter()
        .map(|rule| {
            let content = read_to_string(&rule.macro_file)
                .unwrap_or_else(|_| panic!("Could not open macro file: {:?}", rule.macro_file));
            (rule.name.as_str(), content)
        })
        .collect::<Vec<_>>();

    for path in &params.input_projects {
        let path = source.join(path);
        let dirname = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let project = Project::new(path.to_str().unwrap().to_owned());
        bake_project(
            project,
            &dirname,
            &output,
            params.image_filtering,
            params.tile_margin,
            match &params.material_name {
                Some(name) => name.as_str(),
                None => DEFAULT_SPRITE_MATERIAL_ASSET,
            },
            &params.assets_path_prefix,
            params.image_folder_name.as_deref().unwrap_or("images"),
            params.atlas_folder_name.as_deref().unwrap_or("atlases"),
            params.prefab_folder_name.as_deref().unwrap_or("prefabs"),
            params.data_folder_name.as_deref().unwrap_or("data"),
            &rules,
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn bake_project(
    project: Project,
    input: &Path,
    output: &Path,
    filtering: ImageFiltering,
    tile_margin: Vec2,
    material_name: &str,
    assets_path_prefix: &str,
    image_folder_name: &str,
    atlas_folder_name: &str,
    prefab_folder_name: &str,
    data_folder_name: &str,
    rules: &[(&str, String)],
) {
    let mut image_bytes = HashMap::<_, _>::default();
    let mut images = HashMap::<_, _>::default();
    let mut atlases = HashMap::<_, _>::default();

    for tileset in &project.defs.tilesets {
        let image_bytes_name = format!(
            "{}{}/{}.png",
            assets_path_prefix, image_folder_name, tileset.identifier
        );
        let image_bytes_path = output
            .join(image_folder_name)
            .join(&tileset.identifier)
            .with_extension("png");
        ensure_path(&image_bytes_path);
        let source_path = input.join(&tileset.rel_path);
        copy(source_path, image_bytes_path)
            .unwrap_or_else(|_| panic!("Could not copy file: {}", tileset.rel_path));
        image_bytes.insert(tileset.uid, image_bytes_name.to_owned());

        let image_name = format!(
            "{}{}/{}.yaml",
            assets_path_prefix, image_folder_name, tileset.identifier
        );
        let asset = ImageAssetSource::Png {
            bytes_path: image_bytes_name,
            descriptor: ImageDescriptor {
                filtering,
                ..Default::default()
            },
        };
        let image_path = output
            .join(image_folder_name)
            .join(&tileset.identifier)
            .with_extension("yaml");
        ensure_path(&image_path);
        write(
            &image_path,
            serde_yaml::to_string(&asset).unwrap_or_else(|_| {
                panic!(
                    "Could not serialize image asset for tileset: {:?}",
                    tileset.identifier
                )
            }),
        )
        .unwrap_or_else(|_| panic!("Could not write image asset to file: {:?}", image_path));
        images.insert(tileset.uid, image_name.to_owned());

        let atlas_name = format!(
            "{}{}/{}.yaml",
            assets_path_prefix, atlas_folder_name, tileset.identifier
        );
        let asset = AtlasAssetSource::TileSet({
            let mut map = HashMap::with_capacity(1);
            map.insert(
                image_name.to_owned(),
                TileSetPage {
                    cols: tileset.c_wid as _,
                    rows: tileset.c_hei as _,
                    cell_size: Vec2::new(tileset.tile_grid_size as _, tileset.tile_grid_size as _),
                    padding: Vec2::new(tileset.padding as _, tileset.padding as _),
                    spacing: Vec2::new(tileset.spacing as _, tileset.spacing as _),
                    tile_margin,
                },
            );
            map
        });
        let atlas_path = output
            .join(atlas_folder_name)
            .join(&tileset.identifier)
            .with_extension("yaml");
        ensure_path(&atlas_path);
        write(
            &atlas_path,
            serde_yaml::to_string(&asset).unwrap_or_else(|_| {
                panic!(
                    "Could not serialize image asset for tileset: {}",
                    tileset.identifier
                )
            }),
        )
        .unwrap_or_else(|_| panic!("Could not write tileset asset to file: {:?}", atlas_path));
        atlases.insert(tileset.uid, atlas_name.to_owned());
    }

    let mut instance_types_used = HashSet::new();

    for level in &project.levels {
        if level_field_value("Ignore", level)
            .and_then(|v| v.as_bool())
            .unwrap_or_default()
        {
            continue;
        }

        let variables = level_variables(level);

        if let Some(layer_instances) = &level.layer_instances {
            let mut assets_used = HashSet::new();
            let mut asset = PrefabScene {
                template_name: Some(level.identifier.to_owned()),
                ..Default::default()
            };

            let mut data_asset = Option::<TileMapAsset>::None;

            for layer in layer_instances.iter().rev() {
                if !layer.visible {
                    continue;
                }
                let tileset_id = match layer.tileset_def_uid {
                    Some(id) => id,
                    None => continue,
                };
                let tileset = project
                    .defs
                    .tilesets
                    .iter()
                    .find(|t| t.uid == tileset_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "Tileset not found: {} for layer: {}",
                            tileset_id, layer.identifier
                        )
                    });
                let atlas = atlases
                    .get(&tileset_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "Tileset: {} atlas not found for layer: {}",
                            tileset.identifier, layer.identifier
                        )
                    })
                    .to_owned();
                assets_used.insert(format!("atlas://{}", atlas));
                let tiles = if !layer.auto_layer_tiles.is_empty() {
                    &layer.auto_layer_tiles
                } else if !layer.grid_tiles.is_empty() {
                    &layer.grid_tiles
                } else {
                    panic!("Layer has no tiles available: {}", layer.identifier);
                };
                let tiles = tiles
                    .iter()
                    .map(|tile| {
                        let col = tile.px[0] / layer.grid_size;
                        let row = tile.px[1] / layer.grid_size;
                        let tcol = tile.t % tileset.c_wid;
                        let trow = tile.t / tileset.c_wid;
                        HaTileMapTile {
                            col: col as _,
                            row: row as _,
                            atlas_item: format!("{}x{}", tcol, trow),
                        }
                    })
                    .collect::<Vec<_>>();

                if let Some(data_asset) = &mut data_asset {
                    if data_asset.values.len() == layer.int_grid_csv.len() {
                        for (a, b) in data_asset.values.iter_mut().zip(layer.int_grid_csv.iter()) {
                            if *b > 0 {
                                *a = *b as _;
                            }
                        }
                    }
                } else {
                    data_asset = Some(TileMapAsset {
                        x: level.world_x as _,
                        y: level.world_y as _,
                        cols: layer.c_wid as _,
                        rows: layer.c_hei as _,
                        cell_size: Vec2::new(layer.grid_size as _, layer.grid_size as _),
                        values: layer.int_grid_csv.iter().map(|v| *v as _).collect(),
                    });
                }

                let mut entity_data = PrefabSceneEntityData::default();
                for (name, content) in rules {
                    if let Some(value) = level_field_value(name, level) {
                        let lines = value.as_str().unwrap_or_else(|| {
                            panic!(
                                "{} field of level: {} is not a string!",
                                name, level.identifier
                            )
                        });
                        let variables = variables
                            .iter()
                            .map(|(k, v)| (k.to_owned(), v.to_owned()))
                            .chain(
                                lines
                                    .split(is_separator)
                                    .filter(|line| !line.is_empty())
                                    .filter_map(|line| {
                                        line.find(':').map(|index| {
                                            (
                                                line[..index].trim().to_owned(),
                                                line[(index + 1)..].trim().to_owned(),
                                            )
                                        })
                                    }),
                            )
                            .collect::<HashMap<_, _>>();
                        let components = process_macro(content, variables, name);
                        let components = ComponentsPrefab::from_prefab_str(&components)
                            .unwrap_or_else(|_| {
                                panic!(
                                    "Could not deserialize components map string for level: {}",
                                    level.identifier
                                )
                            });
                        for (name, data) in components.0 {
                            entity_data.components.insert(name, data);
                        }
                    }
                }
                entity_data.components.insert(
                    "NonPersistent".to_owned(),
                    NonPersistentPrefabProxy
                        .to_prefab()
                        .unwrap_or_else(|_| panic!("Could not serialize NonPersistent to prefab")),
                );
                entity_data.components.insert(
                    "HaTransform".to_owned(),
                    HaTransform::default()
                        .with_translation(Vec3::new(level.world_x as _, level.world_y as _, 0.0))
                        .to_prefab()
                        .unwrap_or_else(|_| panic!("Could not serialize HaTransform to prefab")),
                );
                let mut instance = HaTileMapInstance::default();
                instance.set_atlas(atlas);
                instance.set_cols(layer.c_wid as _);
                instance.set_rows(layer.c_hei as _);
                instance.set_tiles(tiles);
                instance.set_cell_size(Vec2::new(layer.grid_size as _, layer.grid_size as _));
                entity_data.components.insert(
                    "HaTileMapInstance".to_owned(),
                    instance.to_prefab().unwrap_or_else(|_| {
                        panic!(
                            "Could not serialize HaTileMapInstance to prefab for layer: {}",
                            layer.identifier
                        )
                    }),
                );
                entity_data.components.insert(
                    "HaMeshInstance".to_owned(),
                    HaMeshInstance::default()
                        .to_prefab()
                        .unwrap_or_else(|_| panic!("Could not serialize HaTransform to prefab")),
                );
                entity_data.components.insert(
                    "HaMaterialInstance".to_owned(),
                    HaMaterialInstance {
                        reference: MaterialInstanceReference::Asset(material_name.to_owned()),
                        ..Default::default()
                    }
                    .to_prefab()
                    .unwrap_or_else(|_| panic!("Could not serialize HaTransform to prefab")),
                );
                asset.entities.push(PrefabSceneEntity::Data(entity_data));
            }

            if let Some(data_asset) = data_asset {
                let data_name = format!(
                    "{}{}/{}.yaml",
                    assets_path_prefix, data_folder_name, level.identifier
                );
                let data_path = output
                    .join(data_folder_name)
                    .join(&level.identifier)
                    .with_extension("yaml");
                ensure_path(&data_path);
                write(
                    &data_path,
                    serde_yaml::to_string(&data_asset).unwrap_or_else(|_| {
                        panic!(
                            "Could not serialize tilemap data asset for level: {:?}",
                            level.identifier
                        )
                    }),
                )
                .unwrap_or_else(|_| {
                    panic!("Could not write tilemap asset to file: {:?}", data_path)
                });
                assets_used.insert(format!("tilemap://{}", data_name));
            }

            for layer in layer_instances {
                if !layer.visible {
                    continue;
                }

                let mut variables = variables.to_owned();
                variables.extend(layer_variables(layer));

                for entity in &layer.entity_instances {
                    let entity_def = project
                        .defs
                        .entities
                        .iter()
                        .find(|e| e.uid == entity.def_uid)
                        .unwrap_or_else(|| {
                            panic!("Could not find entity definition: {}", entity.def_uid)
                        });
                    if let Some(value) = entity_field_value("Assets", entity, entity_def) {
                        if let Some(value) = value.as_str() {
                            for name in value.split(is_separator) {
                                assets_used.insert(name.to_owned());
                            }
                        } else {
                            for value in value.as_array().unwrap_or_else(|| {
                                panic!(
                                    "Assets field oof entity: {} is not an array!",
                                    entity.identifier
                                )
                            }) {
                                let name = value.as_str().unwrap_or_else(|| {
                                    panic!(
                                        "Assets array item field of entity: {} is not a string!",
                                        entity.identifier
                                    )
                                });
                                assets_used.insert(name.to_owned());
                            }
                        }
                    }
                    if let Some(value) = entity_field_value("Template", entity, entity_def) {
                        let name = value.as_str().unwrap_or_else(|| {
                            panic!(
                                "Template field of entity: {} is not a string!",
                                entity.identifier
                            )
                        });
                        asset
                            .entities
                            .push(PrefabSceneEntity::Template(name.to_owned()));
                        continue;
                    }

                    let mut variables = variables.to_owned();
                    variables.extend(entity_variables(entity));

                    let mut entity_data = PrefabSceneEntityData::default();
                    for (name, content) in rules {
                        if let Some(value) = entity_field_value(name, entity, entity_def) {
                            let lines = value.as_str().unwrap_or_else(|| {
                                panic!(
                                    "{} field of entity: {} is not a string!",
                                    name, entity.identifier
                                )
                            });
                            let variables = variables
                                .iter()
                                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                                .chain(
                                    lines
                                        .split(is_separator)
                                        .filter(|line| !line.is_empty())
                                        .filter_map(|line| {
                                            line.find(':').map(|index| {
                                                (
                                                    line[..index].trim().to_owned(),
                                                    line[(index + 1)..].trim().to_owned(),
                                                )
                                            })
                                        }),
                                )
                                .collect::<HashMap<_, _>>();
                            let components = process_macro(content, variables, name);
                            let components = ComponentsPrefab::from_prefab_str(&components)
                                .unwrap_or_else(|_| {
                                    panic!(
                                    "Could not deserialize components map string for entity: {}",
                                    entity.identifier
                                )
                                });
                            for (name, data) in components.0 {
                                entity_data.components.insert(name, data);
                            }
                        }
                    }
                    if entity_field_value("Singleton", entity, entity_def)
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default()
                        && instance_types_used.contains(&entity.identifier)
                    {
                        println!(
                            "Skipping more than one entity instance of singleton: {} in level: {}",
                            entity.identifier, level.identifier,
                        );
                        continue;
                    }
                    let persistent = entity_field_value("Persistent", entity, entity_def)
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default();
                    let no_transform = entity_field_value("NoTransform", entity, entity_def)
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default();
                    let mesh_asset = entity_field_value("MeshAsset", entity, entity_def)
                        .map(|v| v.as_str().unwrap_or(DEFAULT_SPRITE_MESH_ASSET));
                    let sprite_image = entity_field_value("SpriteImage", entity, entity_def)
                        .map(|v| v.as_str().unwrap_or(DEFAULT_SPRITE_IMAGE));
                    let material_asset = entity_field_value("MaterialAsset", entity, entity_def)
                        .map(|v| {
                            v.as_str().unwrap_or(match sprite_image {
                                Some("Uniforms") => DEFAULT_SPRITE_UNIFORMS_MATERIAL_ASSET,
                                _ => DEFAULT_SPRITE_MATERIAL_ASSET,
                            })
                        });
                    let sprite_image_name = if let (Some(_), Some(tileset_id), Some(tile_id)) =
                        (sprite_image, entity_def.tileset_id, entity_def.tile_id)
                    {
                        let tileset = project
                            .defs
                            .tilesets
                            .iter()
                            .find(|t| t.uid == tileset_id)
                            .unwrap_or_else(|| panic!("Could not find tileset: {}", tileset_id));
                        let atlas = atlases.get(&tileset_id).unwrap_or_else(|| {
                            panic!("Could not find atlas for tileset: {}", tileset_id)
                        });
                        let tcol = tile_id % tileset.c_wid;
                        let trow = tile_id / tileset.c_wid;
                        assets_used.insert(format!("atlas://{}", atlas));
                        Some(format!("{}@{}x{}", atlas, tcol, trow))
                    } else {
                        None
                    };
                    if let Some(name) = entity_field_value("Name", entity, entity_def) {
                        let name = name.as_str().unwrap_or_else(|| {
                            panic!(
                                "Could not get name string for entity: {}",
                                entity.identifier
                            )
                        });
                        entity_data.components.insert(
                            "Name".to_owned(),
                            Name(name.to_owned().into())
                                .to_prefab()
                                .unwrap_or_else(|_| panic!("Could not serialize Name to prefab")),
                        );
                    }
                    if let Some(tag) = entity_field_value("Tag", entity, entity_def) {
                        let tag = tag.as_str().unwrap_or_else(|| {
                            panic!("Could not get tag string for entity: {}", entity.identifier)
                        });
                        entity_data.components.insert(
                            "Tag".to_owned(),
                            Tag(tag.to_owned().into())
                                .to_prefab()
                                .unwrap_or_else(|_| panic!("Could not serialize Tag to prefab")),
                        );
                    }
                    if let Some(value) = entity_field_value("Visibility", entity, entity_def) {
                        let visible = value.as_bool().unwrap_or_else(|| {
                            panic!(
                                "Could not get visibility bool for entity: {}",
                                entity.identifier
                            )
                        });
                        entity_data.components.insert(
                            "HaVisibility".to_owned(),
                            HaVisibility(visible).to_prefab().unwrap_or_else(|_| {
                                panic!("Could not serialize HaVisibility to prefab")
                            }),
                        );
                    }
                    if let Some(value) = entity_field_value("Gizmo", entity, entity_def) {
                        let visible = value.as_bool().unwrap_or_else(|| {
                            panic!("Could not get gizmo bool for entity: {}", entity.identifier)
                        });
                        entity_data.components.insert(
                            "HaGizmo".to_owned(),
                            HaGizmo {
                                visible,
                                ..Default::default()
                            }
                            .to_prefab()
                            .unwrap_or_else(|_| panic!("Could not serialize HaGizmo to prefab")),
                        );
                    }
                    if !persistent {
                        entity_data.components.insert(
                            "NonPersistent".to_owned(),
                            NonPersistentPrefabProxy.to_prefab().unwrap_or_else(|_| {
                                panic!("Could not serialize NonPersistent to prefab")
                            }),
                        );
                    }
                    let has_camera = if let Some(pipeline) =
                        entity_field_value("Camera", entity, entity_def)
                    {
                        let pipeline = pipeline
                            .as_str()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not get camera pipeline string for entity: {}",
                                    entity.identifier
                                )
                            })
                            .to_owned();
                        let inside = entity_field_value("CameraInsideView", entity, entity_def)
                            .and_then(|v| v.as_bool())
                            .unwrap_or_default();
                        let mut camera = HaCamera::default();
                        camera.projection =
                            HaCameraProjection::Orthographic(HaCameraOrthographic {
                                scaling: HaCameraOrtographicScaling::FitToView(
                                    Vec2::new(entity.width as _, entity.height as _),
                                    inside,
                                ),
                                centered: true,
                                ignore_depth_planes: false,
                            });
                        camera.pipeline = PipelineSource::Registry(pipeline);
                        entity_data.components.insert(
                            "HaCamera".to_owned(),
                            camera.to_prefab().unwrap_or_else(|_| {
                                panic!("Could not serialize HaCamera to prefab")
                            }),
                        );
                        let is_default = entity_field_value("DefaultCamera", entity, entity_def)
                            .and_then(|v| v.as_bool())
                            .unwrap_or_default();
                        if is_default {
                            entity_data.components.insert(
                                "HaDefaultCamera".to_owned(),
                                HaDefaultCamera.to_prefab().unwrap_or_else(|_| {
                                    panic!("Could not serialize HaDefaultCamera to prefab")
                                }),
                            );
                        }
                        true
                    } else {
                        false
                    };
                    if !no_transform {
                        let position = if has_camera {
                            Vec3::new(
                                entity.px[0] as Scalar + entity.width as Scalar * 0.5,
                                entity.px[1] as Scalar + entity.height as Scalar * 0.5,
                                0.0,
                            )
                        } else {
                            Vec3::new(entity.px[0] as _, entity.px[1] as _, 0.0)
                        };
                        let scale = if sprite_image_name.is_some() {
                            Vec3::new(entity.width as _, entity.height as Scalar, 1.0)
                        } else {
                            Vec3::one()
                        };
                        entity_data.components.insert(
                            "HaTransform".to_owned(),
                            HaTransform::default()
                                .with_translation(position)
                                .with_scale(scale)
                                .to_prefab()
                                .unwrap_or_else(|_| {
                                    panic!("Could not serialize HaTransform to prefab")
                                }),
                        );
                    }
                    if let Some(name) = mesh_asset {
                        entity_data.components.insert(
                            "HaMeshInstance".to_owned(),
                            HaMeshInstance {
                                reference: MeshInstanceReference::Asset(name.to_owned()),
                            }
                            .to_prefab()
                            .unwrap_or_else(|_| {
                                panic!("Could not serialize HaTransform to prefab")
                            }),
                        );
                    }
                    if let Some(name) = material_asset {
                        entity_data.components.insert(
                            "HaMaterialInstance".to_owned(),
                            HaMaterialInstance {
                                reference: MaterialInstanceReference::Asset(name.to_owned()),
                                ..Default::default()
                            }
                            .to_prefab()
                            .unwrap_or_else(|_| {
                                panic!("Could not serialize HaTransform to prefab")
                            }),
                        );
                    }
                    if let (Some("Uniforms"), Some(name)) = (sprite_image, &sprite_image_name) {
                        let mut data = HaVirtualImageUniforms::default();
                        data.set("mainImage", name);
                        entity_data.components.insert(
                            "HaVirtualImageUniforms".to_owned(),
                            data.to_prefab().unwrap_or_else(|_| {
                                panic!("Could not serialize HaTransform to prefab")
                            }),
                        );
                    }
                    if let Some(value) = entity_field_value("SpriteAnimation", entity, entity_def) {
                        let name = value.as_str().unwrap_or_else(|| {
                            panic!(
                                "Could not get sprite animation string for entity: {}",
                                entity.identifier
                            )
                        });
                        let mut data = HaSpriteAnimationInstance::default();
                        data.playing = true;
                        data.set_animation(name);
                        entity_data.components.insert(
                            "HaSpriteAnimationInstance".to_owned(),
                            data.to_prefab().unwrap_or_else(|_| {
                                panic!("Could not serialize HaSpriteAnimationInstance to prefab")
                            }),
                        );
                        assets_used.insert(format!("sanim://{}", name));
                    }
                    if let Some(value) = entity_field_value("Components", entity, entity_def) {
                        let components = value.as_str().unwrap_or_else(|| {
                            panic!(
                                "Could not get components map string for entity: {}",
                                entity.identifier
                            )
                        });
                        let components = ComponentsPrefab::from_prefab_str(components)
                            .unwrap_or_else(|_| {
                                panic!(
                                    "Could not deserialize components map string for entity: {}",
                                    entity.identifier
                                )
                            });
                        for (name, data) in components.0 {
                            entity_data.components.insert(name, data);
                        }
                    }
                    asset.entities.push(PrefabSceneEntity::Data(entity_data));
                    instance_types_used.insert(entity.identifier.to_owned());
                }
            }

            let prefab_name = format!(
                "{}{}/{}.yaml",
                assets_path_prefix, prefab_folder_name, level.identifier
            );
            assets_used.insert(format!("prefab://{}", prefab_name));
            let prefab_path = output
                .join(prefab_folder_name)
                .join(&level.identifier)
                .with_extension("yaml");
            ensure_path(&prefab_path);
            write(
                &prefab_path,
                asset.to_prefab_string().unwrap_or_else(|_| {
                    panic!(
                        "Could not serialize prefab asset for level: {}",
                        level.identifier
                    )
                }),
            )
            .unwrap_or_else(|_| panic!("Could not write prefab asset to file: {:?}", prefab_path));

            let set_path = output.join(&level.identifier).with_extension("txt");
            ensure_path(&set_path);
            write(
                &set_path,
                assets_used.into_iter().collect::<Vec<_>>().join("\n"),
            )
            .unwrap_or_else(|_| panic!("Could not write set asset to file: {:?}", set_path));
        }
    }
}

fn level_field_value<'a>(name: &str, level: &'a Level) -> Option<&'a Value> {
    level.field_instances.iter().find_map(|f| {
        if f.identifier == name {
            f.value.as_ref()
        } else {
            None
        }
    })
}

fn entity_field_value<'a>(
    name: &str,
    instance: &'a EntityInstance,
    definition: &'a EntityDefinition,
) -> Option<&'a Value> {
    instance
        .field_instances
        .iter()
        .find_map(|f| {
            if f.identifier == name {
                f.value.as_ref()
            } else {
                None
            }
        })
        .or_else(|| {
            definition.field_defs.iter().find_map(|f| {
                if f.identifier == name {
                    f.default_override.as_ref()
                } else {
                    None
                }
            })
        })
}

fn ensure_path(path: &Path) {
    if path.extension().is_some() {
        let path = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| panic!("Could not get directory of file: {:?}", path));
        create_dir_all(&path)
            .unwrap_or_else(|_| panic!("Could not create directories: {:?}", path));
    } else {
        create_dir_all(path).unwrap_or_else(|_| panic!("Could not create directories: {:?}", path));
    }
}

fn process_macro(content: &str, variables: HashMap<String, String>, name: &str) -> String {
    chrobry_core::generate(content, "\n", variables, |_| Ok(Default::default()))
        .unwrap_or_else(|_| panic!("Could not process macro for component: {}", name))
}

fn is_separator(c: char) -> bool {
    c == '\r' || c == '\n' || c == '|'
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ComponentsPrefab(pub HashMap<String, PrefabValue>);
impl Prefab for ComponentsPrefab {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComponentRule {
    pub name: String,
    pub macro_file: PathBuf,
}

fn level_variables(level: &Level) -> HashMap<String, String> {
    let mut result = HashMap::with_capacity(5);
    result.insert("level_identifier".to_owned(), level.identifier.to_string());
    result.insert("level_px_wid".to_owned(), level.px_wid.to_string());
    result.insert("level_px_hei".to_owned(), level.px_hei.to_string());
    result.insert("level_world_x".to_owned(), level.world_x.to_string());
    result.insert("level_world_y".to_owned(), level.world_y.to_string());
    result.insert(
        "level_world_col".to_owned(),
        (level.world_x / level.px_wid).to_string(),
    );
    result.insert(
        "level_world_row".to_owned(),
        (level.world_y / level.px_hei).to_string(),
    );
    result
}

fn layer_variables(layer: &LayerInstance) -> HashMap<String, String> {
    let mut result = HashMap::with_capacity(8);
    result.insert("layer_identifier".to_owned(), layer.identifier.to_string());
    result.insert("layer_c_wid".to_owned(), layer.c_wid.to_string());
    result.insert("layer_c_hei".to_owned(), layer.c_hei.to_string());
    result.insert("layer_grid_size".to_owned(), layer.grid_size.to_string());
    result.insert(
        "layer_px_total_offset_x".to_owned(),
        layer.px_total_offset_x.to_string(),
    );
    result.insert(
        "layer_px_total_offset_y".to_owned(),
        layer.px_total_offset_y.to_string(),
    );
    result.insert(
        "layer_px_offset_x".to_owned(),
        layer.px_offset_x.to_string(),
    );
    result.insert(
        "layer_px_offset_y".to_owned(),
        layer.px_offset_y.to_string(),
    );
    result
}

fn entity_variables(entity: &EntityInstance) -> HashMap<String, String> {
    let mut result = HashMap::with_capacity(9);
    result.insert(
        "entity_identifier".to_owned(),
        entity.identifier.to_string(),
    );
    result.insert("entity_grid_col".to_owned(), entity.grid[0].to_string());
    result.insert("entity_grid_row".to_owned(), entity.grid[1].to_string());
    result.insert("entity_pivot_x".to_owned(), entity.pivot[0].to_string());
    result.insert("entity_pivot_y".to_owned(), entity.pivot[1].to_string());
    result.insert(
        "entity_center_x".to_owned(),
        (entity.width as Scalar * entity.pivot[0] as Scalar).to_string(),
    );
    result.insert(
        "entity_center_y".to_owned(),
        (entity.height as Scalar * entity.pivot[1] as Scalar).to_string(),
    );
    result.insert("entity_px_x".to_owned(), entity.px[0].to_string());
    result.insert("entity_px_y".to_owned(), entity.px[1].to_string());
    result.insert("entity_width".to_owned(), entity.width.to_string());
    result.insert("entity_height".to_owned(), entity.height.to_string());
    result
}
