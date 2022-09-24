mod atlas;
mod document;
mod meta;

use crate::{atlas::*, document::*, meta::*};
use oxygengine_animation::{phase::*, spline::*};
use oxygengine_build_tools::*;
use oxygengine_core::{assets::protocol::*, Scalar};
use oxygengine_ha_renderer::{
    asset_protocols::{atlas::*, image::*, mesh::*, skeletal_animation::*, skeleton::*},
    components::transform::*,
    material::domains::surface::skinned::sprite::*,
    math::*,
    mesh::skeleton::*,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::Write,
    fs::{copy, create_dir_all, read_to_string, write},
    io::Error,
    path::PathBuf,
};

#[derive(Debug, Clone, Deserialize)]
struct Params {
    pub input_skeletons: Vec<PathBuf>,
    #[serde(default)]
    pub output: PathBuf,
    #[serde(default)]
    pub assets_path_prefix: String,
}

impl ParamsFromArgs for Params {
    fn params_from_args(args: impl Iterator<Item = String>) -> Option<Self> {
        let mut args = StructuredArguments::new(args);
        let input_skeletons = args.consume_default().map(|v| v.into()).collect();
        let output = args
            .consume_many(&["o", "output"])
            .next()
            .map(|v| v.into())
            .unwrap_or_default();
        let assets_path_prefix = args
            .consume_many(&["p", "prefix"])
            .last()
            .unwrap_or_default();
        Some(Self {
            input_skeletons,
            output,
            assets_path_prefix,
        })
    }
}

fn main() -> Result<(), Error> {
    let (source, destination, params) = AssetPipelineInput::<Params>::consume().unwrap();
    let output = destination.join(&params.output);
    create_dir_all(&output)?;

    for path in &params.input_skeletons {
        let mut assets_set = String::default();
        let skeleton_path = source.join(path);
        let meta_path = skeleton_path.with_extension("meta.json");
        let atlas_path = skeleton_path.with_extension("atlas");
        if !atlas_path.exists() {
            println!(
                "Atlas file: {:?} for skeleton: {:?} not found",
                atlas_path, skeleton_path
            );
            continue;
        }

        let name = skeleton_path.with_extension("");
        let name = name
            .file_name()
            .unwrap_or_else(|| panic!("Could not get skeleton name: {:?}", skeleton_path))
            .to_str()
            .unwrap();
        let source_dir = skeleton_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let target_dir = output.join(&name);
        create_dir_all(&target_dir)?;
        let meta = if meta_path.exists() {
            serde_json::from_str::<AnimationMeta>(&read_to_string(meta_path)?)?
        } else {
            Default::default()
        };

        let atlas = Atlas::parse(&read_to_string(atlas_path)?);
        let from = source_dir.join(&atlas.atlas);
        let to = target_dir.join("image.png");
        copy(&from, &to)
            .unwrap_or_else(|_| panic!("Could not copy image file from {:?} to {:?}", from, to));
        let asset = ImageAssetSource::Png {
            bytes_paths: vec![format!("{}{}/image.png", params.assets_path_prefix, name)],
            descriptor: Default::default(),
        };
        let path = target_dir.join("image.yaml");
        write(
            &path,
            serde_yaml::to_string(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize atlas image asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write atlas image asset to file: {:?}", path));

        let asset = convert_atlas(&atlas, &params.assets_path_prefix, name);
        let path = target_dir.join("atlas.yaml");
        write(
            &path,
            serde_yaml::to_string(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize atlas asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write atlas asset to file: {:?}", path));
        writeln!(
            assets_set,
            "atlas://{}{}/atlas.yaml",
            params.assets_path_prefix, name
        )
        .unwrap();

        let document = serde_json::from_str::<Document>(&read_to_string(skeleton_path)?)?;
        let path = target_dir.join("skeleton.yaml");
        let asset = match convert_skeleton_to_skeleton(&document) {
            Ok(skeleton) => skeleton,
            Err(error) => panic!("Could not convert to skeleton asset: {:?}. {}", path, error),
        };
        write(
            &path,
            serde_yaml::to_string(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize skeleton asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write skeleton asset to file: {:?}", path));
        writeln!(
            assets_set,
            "skeleton://{}{}/skeleton.yaml",
            params.assets_path_prefix, name
        )
        .unwrap();

        let path = target_dir.join("animation.yaml");
        let asset = convert_skeleton_to_animation(&document, &meta);
        write(
            &path,
            serde_yaml::to_string(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize animation asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write animation asset to file: {:?}", path));
        writeln!(
            assets_set,
            "skelanim://{}{}/animation.yaml",
            params.assets_path_prefix, name
        )
        .unwrap();

        let path = target_dir.join("mesh.yaml");
        let asset = match convert_skeleton_to_mesh(
            &document,
            &atlas,
            format!("{}{}/skeleton.yaml", params.assets_path_prefix, name),
            &meta,
        ) {
            Ok(mesh) => mesh,
            Err(error) => panic!(
                "Could not convert to skeleton mesh asset: {:?}. {}",
                path, error
            ),
        };
        write(
            &path,
            serde_yaml::to_string(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize skeleton mesh asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write skeleton mesh asset to file: {:?}", path));
        writeln!(
            assets_set,
            "mesh://{}{}/mesh.yaml",
            params.assets_path_prefix, name
        )
        .unwrap();

        let path = target_dir.join("assets.txt");
        write(&path, assets_set)
            .unwrap_or_else(|_| panic!("Could not write assets set to file: {:?}", path));
    }
    Ok(())
}

fn convert_skeleton_to_skeleton(document: &Document) -> Result<SkeletonAsset, String> {
    for bone in &document.bones {
        if bone.transform != TransformMode::Normal {
            return Err(format!(
                "Wrong bone: {:?} transform mode: {:?}",
                bone.name, bone.transform
            ));
        }
    }
    let mut hierarchy = document
        .bones
        .iter()
        .map(|bone| {
            let hierarchy = SkeletonHierarchy::new(&bone.name)
                .target(vec3(bone.length, 0.0, 0.0))
                .transform(HaTransform::new(
                    vec3(bone.x, -bone.y, 0.0),
                    Eulers::yaw(-bone.rotation),
                    vec3(bone.scale_x, bone.scale_y, 1.0),
                ));
            (bone.name.to_owned(), hierarchy)
        })
        .collect::<HashMap<_, _>>();
    for bone in document.bones.iter().rev() {
        if let Some(bone_parent) = &bone.parent {
            let bone = hierarchy.remove(&bone.name);
            let parent = hierarchy.remove(bone_parent);
            if let (Some(bone), Some(parent)) = (bone, parent) {
                hierarchy.insert(bone_parent.to_owned(), parent.child(bone));
            }
        }
    }
    match hierarchy.into_iter().next() {
        Some((_, v)) => Ok(SkeletonAsset::new(v)),
        None => Err("Could not find root bone!".to_owned()),
    }
}

struct PhaseExtractMeta {
    time: Scalar,
    value: Scalar,
    curve: Option<[Scalar; 4]>,
}

fn extract_phase(data: Vec<PhaseExtractMeta>) -> Phase {
    let mut points = Vec::with_capacity(data.len());
    for index in 0..data.len() {
        let prev = index.checked_sub(1).and_then(|index| data.get(index));
        let current = data.get(index).unwrap();
        let direction_in = prev
            .map(|prev| {
                if let Some(curve) = &prev.curve {
                    (current.time - curve[2], current.value - curve[3])
                } else {
                    Default::default()
                }
            })
            .unwrap_or_default();
        let direction_out = if let Some(curve) = &current.curve {
            (curve[0] - current.time, curve[1] - current.value)
        } else {
            Default::default()
        };
        let direction = SplinePointDirection::InOut(direction_in, direction_out);
        points.push(SplinePoint::new((current.time, current.value), direction));
    }
    Phase::new(points).unwrap()
}

macro_rules! extract_bone_animation {
    ( $component_list: expr, $value: ident, $offset: expr, $base_value: expr, $multiplier: expr) => {
        if $component_list.is_empty() {
            None
        } else {
            Some(
                $component_list
                    .iter()
                    .map(|item| PhaseExtractMeta {
                        time: item.time,
                        value: item.$value * $multiplier + $base_value * $multiplier,
                        curve: match &item.curve {
                            AnimationCurve::Curve(values) => Some([
                                values[$offset],
                                values[$offset + 1] * $multiplier + $base_value * $multiplier,
                                values[$offset + 2],
                                values[$offset + 3] * $multiplier + $base_value * $multiplier,
                            ]),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>(),
            )
        }
    };
}

fn convert_skeleton_to_animation(
    document: &Document,
    meta: &AnimationMeta,
) -> SkeletalAnimationAsset {
    let sequences = document
        .animations
        .iter()
        .map(|(name, animation)| {
            let bone_sheets = animation
                .bones
                .iter()
                .filter_map(|(name, sheet)| {
                    document
                        .bones
                        .iter()
                        .find(|bone| &bone.name == name)
                        .map(|bone| (name, sheet, bone))
                })
                .map(|(name, sheet, bone)| {
                    let translation_x = extract_bone_animation!(sheet.translate, x, 0, bone.x, 1.0)
                        .map(extract_phase);
                    let translation_y =
                        extract_bone_animation!(sheet.translate, y, 4, bone.y, -1.0)
                            .map(extract_phase);
                    let rotation_yaw =
                        extract_bone_animation!(sheet.rotate, value, 0, bone.rotation, -1.0)
                            .map(extract_phase);
                    let scale_x = extract_bone_animation!(sheet.scale, x, 0, bone.scale_x, 1.0)
                        .map(extract_phase);
                    let scale_y = extract_bone_animation!(sheet.scale, y, 4, bone.scale_y, 1.0)
                        .map(extract_phase);
                    let sheet = SkeletalAnimationSequenceBoneSheet {
                        translation_x,
                        translation_y,
                        translation_z: None,
                        rotation_yaw,
                        rotation_pitch: None,
                        rotation_roll: None,
                        scale_x,
                        scale_y,
                        scale_z: None,
                    };
                    (name.to_owned(), sheet)
                })
                .collect();
            let signals = animation
                .events
                .iter()
                .map(|event| {
                    let mut params = HashMap::default();
                    if let Some(v) = event.int_value {
                        params.insert("int".to_owned(), SkeletalAnimationValue::Integer(v as _));
                    }
                    if let Some(v) = event.float_value {
                        params.insert("float".to_owned(), SkeletalAnimationValue::Scalar(v));
                    }
                    if let Some(v) = &event.string_value {
                        params.insert(
                            "string".to_owned(),
                            SkeletalAnimationValue::String(v.to_owned()),
                        );
                    }
                    if let Some(v) = event.volume {
                        params.insert("volume".to_owned(), SkeletalAnimationValue::Scalar(v));
                    }
                    if let Some(v) = event.balance {
                        params.insert("balance".to_owned(), SkeletalAnimationValue::Scalar(v));
                    }
                    SkeletalAnimationSignal {
                        time: event.time,
                        id: event.name.to_owned(),
                        params,
                    }
                })
                .collect();
            let sequence = SkeletalAnimationSequence {
                speed: 1.0,
                looping: meta.loop_sequences.iter().any(|n| n == name),
                bounce: meta.bounce_sequences.iter().any(|n| n == name),
                bone_sheets,
                signals,
            };
            (name.to_owned(), sequence)
        })
        .collect::<HashMap<_, _>>();
    let mut states = meta.states.to_owned();
    let mut rules = meta.rules.to_owned();
    if let Some(meta) = &meta.make_state_per_sequence {
        for name in sequences.keys() {
            let target_state = format!("{}{}", meta.prefix, name);
            states.insert(
                target_state.to_owned(),
                SkeletalAnimationState {
                    sequences: SkeletalAnimationStateSequences::Single(name.to_owned()),
                    rules: Default::default(),
                },
            );
            let mut conditions = HashMap::default();
            conditions.insert(
                meta.value_name.to_owned(),
                SkeletalAnimationCondition::StringEquals(name.to_owned()),
            );
            rules.push(SkeletalAnimationRule::Single {
                target_state,
                conditions,
                change_time: meta.change_time,
            });
        }
    }
    SkeletalAnimationAsset {
        default_state: meta.default_state.to_owned(),
        speed: 1.0,
        sequences,
        states,
        rules,
    }
}

fn convert_skeleton_to_mesh(
    document: &Document,
    atlas: &Atlas,
    skeleton_path: String,
    meta: &AnimationMeta,
) -> Result<MeshAsset, String> {
    let skin = meta.skin.as_deref().unwrap_or("default");
    let attachments = match document.skins.iter().find(|item| item.name == skin) {
        Some(skin) => &skin.attachments,
        None => return Err(format!("Skin not found: {}", skin)),
    };
    let sprites = document
        .slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| {
            let attachment = slot.attachment.as_ref()?;
            let slot_attachments = attachments.get(&slot.name)?;
            let region = slot_attachments.get(attachment)?;
            Some(
                SurfaceSkinnedSprite::new(
                    &slot.bone,
                    vek::Vec2::new(region.width as _, region.height as _),
                )
                .depth(index as _)
                .pivot(vek::Vec2::new(0.5, 0.5))
                .uvs_rect(atlas.uvs(attachment)?)
                .attachment_transform(HaTransform::new(
                    vec3(region.x, -region.y, 0.0),
                    Eulers::yaw(-region.rotation),
                    vec3(region.scale_x, region.scale_y, 1.0),
                )),
            )
        })
        .collect();
    Ok(MeshAsset::SkinnedSurface(SkinnedSurfaceMeshAsset {
        vertex_data: SurfaceVertexData {
            normal: false,
            texture: true,
            color: false,
        },
        factory: SkinnedSurfaceFactory::Sprite {
            skeleton: AssetVariant::Path(skeleton_path),
            factory: SurfaceSkinnedSpriteFactory { sprites },
        },
    }))
}

fn convert_atlas(atlas: &Atlas, assets_path_prefix: &str, name: &str) -> AtlasAssetSource {
    let frames = atlas
        .regions
        .iter()
        .map(|(id, region)| {
            let id = id.to_owned();
            let region = AtlasRegion {
                rect: region.rect(),
                layer: 0,
            };
            (id, region)
        })
        .collect::<HashMap<_, _>>();
    let mut pages = HashMap::with_capacity(1);
    pages.insert(format!("{}{}/image.yaml", assets_path_prefix, name), frames);
    AtlasAssetSource::Raw(pages)
}
