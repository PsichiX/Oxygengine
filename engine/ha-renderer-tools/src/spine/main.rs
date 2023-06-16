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
    fmt::Write,
    fs::{copy, create_dir_all, read_to_string, write},
    io::Error,
    path::PathBuf,
};

#[derive(Debug, Clone, Deserialize)]
struct Params {
    pub input_documents: Vec<PathBuf>,
    #[serde(default)]
    pub output: PathBuf,
    #[serde(default)]
    pub assets_path_prefix: String,
}

impl ParamsFromArgs for Params {
    fn params_from_args(args: impl Iterator<Item = String>) -> Option<Self> {
        let mut args = StructuredArguments::new(args);
        let input_documents = args.consume_default().map(|v| v.into()).collect();
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
            input_documents,
            output,
            assets_path_prefix,
        })
    }
}

fn main() -> Result<(), Error> {
    let (source, destination, params) = AssetPipelineInput::<Params>::consume().unwrap();
    let output = destination.join(&params.output);
    create_dir_all(&output)?;

    for path in &params.input_documents {
        let mut assets_set = String::default();
        let document_path = source.join(path);
        let meta_path = document_path.with_extension("meta.json");
        let atlas_path = document_path.with_extension("atlas");
        if !atlas_path.exists() {
            println!(
                "Atlas file: {:?} for document: {:?} not found",
                atlas_path, document_path
            );
            continue;
        }

        let name = document_path.with_extension("");
        let name = name
            .file_name()
            .unwrap_or_else(|| panic!("Could not get document name: {:?}", document_path))
            .to_str()
            .unwrap();
        let source_dir = document_path
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
        let path = target_dir.join("image.json");
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize atlas image asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write atlas image asset to file: {:?}", path));

        let asset = convert_atlas(&atlas, &params.assets_path_prefix, name);
        let path = target_dir.join("atlas.json");
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize atlas asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write atlas asset to file: {:?}", path));
        writeln!(
            assets_set,
            "atlas://{}{}/atlas.json",
            params.assets_path_prefix, name
        )
        .unwrap();

        let document = serde_json::from_str::<Document>(&read_to_string(document_path)?)?;
        let path = target_dir.join("rig.json");
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
        writeln!(
            assets_set,
            "rig://{}{}/rig.json",
            params.assets_path_prefix, name
        )
        .unwrap();

        let path = target_dir.join("animation.json");
        let asset = convert_document_to_animation(&document, &meta);
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize animation asset: {:?}", path)),
        )
        .unwrap_or_else(|_| panic!("Could not write animation asset to file: {:?}", path));
        writeln!(
            assets_set,
            "riganim://{}{}/animation.json",
            params.assets_path_prefix, name
        )
        .unwrap();

        let path = target_dir.join("mesh.json");
        let asset = match convert_document_to_mesh(
            &document,
            &atlas,
            format!("{}{}/rig.json", params.assets_path_prefix, name),
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
        writeln!(
            assets_set,
            "mesh://{}{}/mesh.json",
            params.assets_path_prefix, name
        )
        .unwrap();

        let path = target_dir.join("assets.txt");
        write(&path, assets_set)
            .unwrap_or_else(|_| panic!("Could not write assets set to file: {:?}", path));
    }
    Ok(())
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
        Some((_, v)) => Ok(RigAsset::new(v, Default::default())),
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
    pages.insert(format!("{}{}/image.json", assets_path_prefix, name), frames);
    AtlasAssetSource::Raw(pages)
}
