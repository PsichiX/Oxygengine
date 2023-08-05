mod sdf_generator;

use crate::sdf_generator::*;
use image::*;
use oxygengine_build_tools::*;
use oxygengine_ha_renderer::{
    asset_protocols::{font::*, image::*},
    image::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir_all, read_to_string, write},
    io::Error,
    path::{Path, PathBuf},
};
use texture_packer::{exporter::ImageExporter, MultiTexturePacker, TexturePackerConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Font {
    pub common: Common,
    pub pages: Pages,
    pub chars: Chars,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Common(pub CommonAttributes);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonAttributes {
    pub line_height: usize,
    pub base: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pages {
    #[serde(rename = "$value")]
    pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page(pub PageAttributes);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageAttributes {
    pub id: usize,
    pub file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chars {
    #[serde(rename = "$value")]
    chars: Vec<Char>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Char(pub CharAttributes);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharAttributes {
    pub id: usize,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub xoffset: isize,
    pub yoffset: isize,
    pub xadvance: isize,
    pub page: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct Params {
    #[serde(default)]
    pub image_filtering: ImageFiltering,
    #[serde(default)]
    pub generator: SdfGenerator,
    #[serde(default = "Params::default_max_width")]
    pub max_width: u32,
    #[serde(default = "Params::default_max_height")]
    pub max_height: u32,
    #[serde(default)]
    pub padding: u32,
    #[serde(default)]
    pub force_line_height: Option<usize>,
}

impl Params {
    fn default_max_width() -> u32 {
        1024
    }

    fn default_max_height() -> u32 {
        1024
    }
}

impl ParamsFromArgs for Params {}

fn main() -> Result<(), Error> {
    AssetPipelinePlugin::run::<Params, _>(|input| {
        let AssetPipelineInput {
            source,
            target,
            assets,
            params,
        } = input;
        create_dir_all(&target)?;

        let source = match source.first() {
            Some(source) => source,
            None => return Ok(vec![]),
        };
        Ok(vec![generate_sdf_font(
            source,
            &target,
            &assets,
            params.image_filtering,
            &params.generator,
            params.max_width,
            params.max_height,
            params.padding,
            params.force_line_height,
        )?])
    })
}

#[allow(clippy::too_many_arguments)]
fn generate_sdf_font(
    source: &Path,
    target: &Path,
    assets: &str,
    image_filtering: ImageFiltering,
    generator: &SdfGenerator,
    max_width: u32,
    max_height: u32,
    padding: u32,
    force_line_height: Option<usize>,
) -> Result<String, Error> {
    let dirname = source.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let font = serde_xml_rs::from_str::<Font>(&read_to_string(source)?)
        .unwrap_or_else(|_| panic!("Could not load bmfont XML descriptor: {:?}", source));
    let glyphs = font
        .pages
        .pages
        .iter()
        .flat_map(|page| {
            let path = dirname.join(&page.0.file);
            let image = image::open(&path)
                .unwrap_or_else(|_| panic!("Could not load font: {:?} image: {:?}", source, path))
                .into_luma8();
            font.chars
                .chars
                .iter()
                .filter(move |c| c.0.page == page.0.id)
                .map(move |c| {
                    let image = image
                        .view(c.0.x as _, c.0.y as _, c.0.width as _, c.0.height as _)
                        .to_image();
                    let image = generator.process(&image);
                    (c.0.id, image)
                })
        })
        .collect::<HashMap<_, _>>();

    let config = TexturePackerConfig {
        max_width,
        max_height,
        allow_rotation: false,
        border_padding: 0,
        texture_padding: padding,
        texture_extrusion: 0,
        trim: false,
        texture_outlines: false,
    };
    let mut packer = MultiTexturePacker::new_skyline(config);
    for (id, image) in glyphs {
        packer
            .pack_own(id, image)
            .unwrap_or_else(|_| panic!("Could not pack font glyph: {}", id));
    }
    let diff_y = force_line_height
        .filter(|value| *value > 0)
        .map(|value| value as isize - font.common.0.line_height as isize)
        .unwrap_or_default();
    let pages = packer
        .get_pages()
        .iter()
        .enumerate()
        .map(|(i, page)| {
            let path = target.join(format!("page.{}.png", i));
            ImageExporter::export(page)
                .unwrap_or_else(|_| panic!("Could not export font: {:?} page: {}", source, i))
                .save_with_format(&path, image::ImageFormat::Png)
                .unwrap_or_else(|_| {
                    panic!(
                        "Could not save font: {:?} page: {} to file: {:?}",
                        source, i, path
                    )
                });

            let asset = ImageAssetSource::Png {
                descriptor: ImageDescriptor {
                    mode: ImageMode::Image2dArray,
                    ..Default::default()
                },
                bytes_paths: vec![format!("{}/page.{}.png", assets, i)],
            };
            let path = target.join(format!("page.{}.json", i));
            write(
                &path,
                serde_json::to_string_pretty(&asset).unwrap_or_else(|_| {
                    panic!(
                        "Could not serialize font: {:?} page: {} image asset",
                        source, i
                    )
                }),
            )
            .unwrap_or_else(|_| {
                panic!(
                    "Could not write font: {:?} page: {} image asset to file: {:?}",
                    source, i, path
                )
            });

            let characters = page
                .get_frames()
                .iter()
                .map(|(id, frame)| {
                    let character = &font
                        .chars
                        .chars
                        .iter()
                        .find(|c| c.0.id == *id)
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find font: {:?} page: {} character: {}",
                                source, i, id
                            )
                        })
                        .0;
                    let character = FontAssetSourceCharacter {
                        x: frame.frame.x as _,
                        y: frame.frame.y as _,
                        width: frame.frame.w as _,
                        height: frame.frame.h as _,
                        xoffset: character.xoffset,
                        yoffset: character.yoffset + diff_y,
                        xadvance: character.xadvance,
                    };
                    let c = std::char::from_u32(*id as _).unwrap_or_else(|| {
                        panic!("Could not convert character id: {} to character", id)
                    });
                    (c, character)
                })
                .collect();
            FontAssetSourcePage {
                image: format!("{}/page.{}.json", assets, i),
                characters,
            }
        })
        .collect();

    let asset = FontAssetSource {
        line_height: (font.common.0.line_height as isize + diff_y).max(0) as usize,
        line_base: (font.common.0.base as isize + diff_y).max(0) as usize,
        sdf_resolution: generator.resolution,
        pages,
        filtering: image_filtering,
    };
    let path = target.join("font.json");
    write(
        &path,
        serde_json::to_string_pretty(&asset)
            .unwrap_or_else(|_| panic!("Could not serialize font: {:?} asset", source,)),
    )
    .unwrap_or_else(|_| {
        panic!(
            "Could not write font: {:?} asset to file: {:?}",
            source, path
        )
    });
    Ok(format!("font://{}/font.json", assets))
}
