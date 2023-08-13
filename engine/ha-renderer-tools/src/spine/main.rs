mod atlas;
mod document;
mod meta;

use crate::{atlas::*, document::*, meta::*};
use oxygengine_animation::{phase::*, spline::*};
use oxygengine_build_tools::*;
use oxygengine_core::{assets::protocol::*, Scalar};
use oxygengine_ha_renderer::{
    asset_protocols::{atlas::*, image::*, mesh::*, rig::*, rig_animation::*},
    components::transform::*,
    material::domains::surface::rig2d::*,
    math::*,
    mesh::rig::skeleton::*,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{copy, create_dir_all, read_to_string, write},
    io::Error,
};

#[derive(Debug, Clone, Deserialize)]
struct Params {}

impl ParamsFromArgs for Params {}

fn main() -> Result<(), Error> {
    AssetPipelinePlugin::run::<Params, _>(|input| {
        let AssetPipelineInput {
            source,
            target,
            assets,
            ..
        } = input;
        create_dir_all(&target)?;

        let source = match source.first() {
            Some(source) => source,
            None => return Ok(vec![]),
        };
        let mut assets_used = Vec::default();
        let meta_path = source.with_extension("meta.json");
        let atlas_path = source.with_extension("atlas");
        if !atlas_path.exists() {
            println!(
                "Atlas file: {:?} for document: {:?} not found",
                atlas_path, source
            );
            return Ok(vec![]);
        }

        let mut source_dir = source.to_owned();
        source_dir.pop();
        let meta = if meta_path.exists() {
            serde_json::from_str::<AnimationMeta>(&read_to_string(meta_path)?)?
        } else {
            Default::default()
        };

        let atlas = Atlas::parse(&read_to_string(atlas_path)?);
        let from = source_dir.join(&atlas.atlas);
        let to = target.join("image.png");
        copy(&from, &to)
            .unwrap_or_else(|_| panic!("Could not copy image file from {:?} to {:?}", from, to));
        let asset = ImageAssetSource::Png {
            bytes_paths: vec![format!("{}/image.png", assets)],
            descriptor: Default::default(),
        };
        let path = target.join("image.json");
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize atlas image asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write atlas image asset to file: {:?}", path));

        let asset = convert_atlas(&atlas, &assets);
        let path = target.join("atlas.json");
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize atlas asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write atlas asset to file: {:?}", path));
        assets_used.push(format!("atlas://{}/atlas.json", assets));

        let document = serde_json::from_str::<Document>(&read_to_string(source)?)?;
        let path = target.join("rig.json");
        let asset = match convert_document_to_rig(&document) {
            Ok(rig) => rig,
            Err(error) => panic!("Could not convert to rig asset: {:?}. {}", path, error),
        };
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize rig asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write rig asset to file: {:?}", path));
        assets_used.push(format!("rig://{}/rig.json", assets));

        let path = target.join("animation.json");
        let asset = convert_document_to_animation(&document, &meta);
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize animation asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write animation asset to file: {:?}", path));
        assets_used.push(format!("riganim://{}/animation.json", assets));

        let path = target.join("mesh.json");
        let asset = match convert_document_to_mesh(
            &document,
            &atlas,
            format!("{}/rig.json", assets),
            &meta,
        ) {
            Ok(mesh) => mesh,
            Err(error) => panic!("Could not convert to rig mesh asset: {:?}. {}", path, error),
        };
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize rig mesh asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write rig mesh asset to file: {:?}", path));
        assets_used.push(format!("mesh://{}/mesh.json", assets));
        Ok(assets_used)
    })
}

fn convert_document_to_rig(document: &Document) -> Result<RigAsset, String> {
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
        Some((_, v)) => Ok(RigAsset::new(v, Default::default(), Default::default())),
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

fn convert_document_to_animation(document: &Document, meta: &AnimationMeta) -> RigAnimationAsset {
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
                    let sheet = RigAnimationSequenceBoneSheet {
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
                        params.insert("int".to_owned(), RigAnimationValue::Integer(v as _));
                    }
                    if let Some(v) = event.float_value {
                        params.insert("float".to_owned(), RigAnimationValue::Scalar(v));
                    }
                    if let Some(v) = &event.string_value {
                        params.insert("string".to_owned(), RigAnimationValue::String(v.to_owned()));
                    }
                    if let Some(v) = event.volume {
                        params.insert("volume".to_owned(), RigAnimationValue::Scalar(v));
                    }
                    if let Some(v) = event.balance {
                        params.insert("balance".to_owned(), RigAnimationValue::Scalar(v));
                    }
                    RigAnimationSignal {
                        time: event.time,
                        id: event.name.to_owned(),
                        params,
                    }
                })
                .collect();
            let sequence = RigAnimationSequence {
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
                RigAnimationState {
                    sequences: RigAnimationStateSequences::Single(name.to_owned()),
                    rules: Default::default(),
                },
            );
            let mut conditions = HashMap::default();
            conditions.insert(
                meta.value_name.to_owned(),
                RigAnimationCondition::StringEquals(name.to_owned()),
            );
            rules.push(RigAnimationRule::Single {
                target_state,
                conditions,
                change_time: meta.change_time,
            });
        }
    }
    RigAnimationAsset {
        default_state: meta.default_state.to_owned(),
        speed: 1.0,
        sequences,
        states,
        rules,
    }
}

fn convert_document_to_mesh(
    document: &Document,
    atlas: &Atlas,
    rig_path: String,
    meta: &AnimationMeta,
) -> Result<MeshAsset, String> {
    let skin = meta.skin.as_deref().unwrap_or("default");
    let attachments = match document.skins.iter().find(|item| item.name == skin) {
        Some(skin) => &skin.attachments,
        None => return Err(format!("Skin not found: {}", skin)),
    };
    let nodes = document
        .slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| {
            let attachment = slot.attachment.as_ref()?;
            let slot_attachments = attachments.get(&slot.name)?;
            let region = slot_attachments.get(attachment)?;
            Some(
                SurfaceRig2dNode::new(
                    &slot.bone,
                    SurfaceRig2dSprite::new(vek::Vec2::new(region.width as _, region.height as _))
                        .pivot(vek::Vec2::new(0.5, 0.5))
                        .uvs_rect(atlas.uvs(attachment)?),
                )
                .depth(index as _)
                .attachment_transform(HaTransform::new(
                    vec3(region.x, -region.y, 0.0),
                    Eulers::yaw(-region.rotation),
                    vec3(region.scale_x, region.scale_y, 1.0),
                )),
            )
        })
        .collect();
    Ok(MeshAsset::Surface(SurfaceMeshAsset {
        vertex_data: MeshVertexData {
            deforming: false,
            skinning: true,
            texture: true,
            color: false,
        },
        factory: SurfaceFactory::Rig {
            asset: AssetVariant::Path(rig_path),
            factory: SurfaceRig2dFactory { nodes },
        },
    }))
}

fn convert_atlas(atlas: &Atlas, assets_path_prefix: &str) -> AtlasAssetSource {
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
    pages.insert(format!("{}/image.json", assets_path_prefix), frames);
    AtlasAssetSource::Raw(pages)
}
