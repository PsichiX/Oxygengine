mod sdf_generator;

use crate::sdf_generator::*;
use image::*;
use oxygengine_build_tools::AssetPipelineInput;
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
    pub descriptors: Vec<PathBuf>,
    #[serde(default)]
    pub output: PathBuf,
    #[serde(default)]
    pub assets_path_prefix: String,
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
}

impl Params {
    fn default_max_width() -> u32 {
        1024
    }

    fn default_max_height() -> u32 {
        1024
    }
}

fn main() -> Result<(), Error> {
    let (source, destination, params) = AssetPipelineInput::<Params>::consume().unwrap();
    let output = destination.join(&params.output);
    create_dir_all(&output)?;

    for descriptor in &params.descriptors {
        let descriptor = source.join(descriptor);
        generate_sdf_font(
            &descriptor,
            &output,
            &params.assets_path_prefix,
            params.image_filtering,
            &params.generator,
            params.max_width,
            params.max_height,
            params.padding,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_sdf_font(
    descriptor: &Path,
    output: &Path,
    assets_path_prefix: &str,
    image_filtering: ImageFiltering,
    generator: &SdfGenerator,
    max_width: u32,
    max_height: u32,
    padding: u32,
) -> Result<(), Error> {
    let dirname = descriptor
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let filename = descriptor.with_extension("");
    let filename = filename
        .file_name()
        .unwrap_or_else(|| panic!("Could not get descriptor file name: {:?}", descriptor));
    let font = serde_xml_rs::from_str::<Font>(&read_to_string(descriptor)?)
        .unwrap_or_else(|_| panic!("Could not load bmfont XML descriptor: {:?}", descriptor));
    let glyphs = font
        .pages
        .pages
        .iter()
        .flat_map(|page| {
            let path = dirname.join(&page.0.file);
            let image = image::open(&path)
                .unwrap_or_else(|_| {
                    panic!("Could not load font: {:?} image: {:?}", descriptor, path)
                })
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
    let pages = packer
        .get_pages()
        .iter()
        .enumerate()
        .map(|(i, page)| {
            let path = output.join(&filename).with_extension(&format!("{}.png", i));
            ImageExporter::export(page)
                .unwrap_or_else(|_| panic!("Could not export font: {:?} page: {}", descriptor, i))
                .save_with_format(&path, image::ImageFormat::Png)
                .unwrap_or_else(|_| {
                    panic!(
                        "Could not save font: {:?} page: {} to file: {:?}",
                        descriptor, i, path
                    )
                });

            let image_name = filename.to_str().unwrap();
            let asset = ImageAssetSource::Png {
                descriptor: ImageDescriptor {
                    mode: ImageMode::Image2dArray,
                    ..Default::default()
                },
                bytes_paths: vec![format!("{}{}.{}.png", assets_path_prefix, image_name, i)],
            };
            let path = output
                .join(&filename)
                .with_extension(&format!("{}.yaml", i));
            write(
                &path,
                serde_yaml::to_string(&asset).unwrap_or_else(|_| {
                    panic!(
                        "Could not serialize font: {:?} page: {} image asset",
                        descriptor, i
                    )
                }),
            )
            .unwrap_or_else(|_| {
                panic!(
                    "Could not write font: {:?} page: {} image asset to file: {:?}",
                    descriptor, i, path
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
                                descriptor, i, id
                            )
                        })
                        .0;
                    let character = FontAssetSourceCharacter {
                        x: frame.frame.x as _,
                        y: frame.frame.y as _,
                        width: frame.frame.w as _,
                        height: frame.frame.h as _,
                        xoffset: character.xoffset,
                        yoffset: character.yoffset,
                        xadvance: character.xadvance,
                    };
                    let c = std::char::from_u32(*id as _).unwrap_or_else(|| {
                        panic!("Could not convert character id: {} to character", id)
                    });
                    (c, character)
                })
                .collect();
            FontAssetSourcePage {
                image: format!("{}{}.{}.yaml", assets_path_prefix, image_name, i),
                characters,
            }
        })
        .collect();

    let asset = FontAssetSource {
        line_height: font.common.0.line_height,
        line_base: font.common.0.base,
        sdf_resolution: generator.resolution,
        pages,
        filtering: image_filtering,
    };
    let path = output.join(&filename).with_extension("yaml");
    write(
        &path,
        serde_yaml::to_string(&asset)
            .unwrap_or_else(|_| panic!("Could not serialize font: {:?} asset", descriptor,)),
    )
    .unwrap_or_else(|_| {
        panic!(
            "Could not write font: {:?} asset to file: {:?}",
            descriptor, path
        )
    });
    Ok(())
}
