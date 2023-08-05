use oxygengine_build_tools::*;
use oxygengine_ha_renderer::{
    asset_protocols::{atlas::*, image::*},
    math::*,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{create_dir_all, read_dir, write},
    io::Error,
    path::{Path, PathBuf},
};
use texture_packer::{exporter::ImageExporter, MultiTexturePacker, TexturePackerConfig};

#[derive(Debug, Clone, Deserialize)]
struct Params {
    #[serde(default = "Params::default_max_width")]
    pub max_width: u32,
    #[serde(default = "Params::default_max_height")]
    pub max_height: u32,
    #[serde(default = "Params::default_padding")]
    pub padding: u32,
}

impl Params {
    fn default_max_width() -> u32 {
        1024
    }

    fn default_max_height() -> u32 {
        1024
    }

    fn default_padding() -> u32 {
        2
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

        let mut paths = vec![];
        for directory in &source {
            get_files_recursively(directory, &mut paths);
        }

        let config = TexturePackerConfig {
            max_width: params.max_width,
            max_height: params.max_height,
            allow_rotation: false,
            border_padding: 0,
            texture_padding: params.padding,
            texture_extrusion: 0,
            trim: false,
            texture_outlines: false,
        };
        let mut packer = MultiTexturePacker::new_skyline(config);

        for path in paths {
            let image =
                image::open(&path).unwrap_or_else(|_| panic!("Could not load image: {:?}", path));
            let id = path.with_extension("");
            let id = id
                .file_name()
                .unwrap_or_else(|| panic!("Could not get path file name: {:?}", path))
                .to_str()
                .unwrap();
            packer
                .pack_own(id.to_owned(), image)
                .unwrap_or_else(|_| panic!("Could not pack image: {} in path: {:?}", id, path));
        }
        let pages = packer
            .get_pages()
            .iter()
            .enumerate()
            .map(|(i, page)| {
                let path = target.join(format!("page.{}.png", i));
                ImageExporter::export(page)
                    .unwrap_or_else(|_| panic!("Could not export atlas page: {}", i))
                    .save_with_format(&path, image::ImageFormat::Png)
                    .unwrap_or_else(|_| {
                        panic!("Could not save atlas page: {} to file: {:?}", i, path)
                    });
                let asset = ImageAssetSource::Png {
                    bytes_paths: vec![format!("{}/page.{}.png", assets, i)],
                    descriptor: Default::default(),
                };
                let path = target.join(format!("page.{}.json", i));
                write(
                    &path,
                    serde_json::to_string_pretty(&asset).unwrap_or_else(|_| {
                        panic!("Could not serialize atlas page: {} image asset", i)
                    }),
                )
                .unwrap_or_else(|_| {
                    panic!(
                        "Could not write atlas page: {} image asset to file: {:?}",
                        i, path
                    )
                });
                let frames = page
                    .get_frames()
                    .iter()
                    .map(|(id, frame)| {
                        (
                            id.to_owned(),
                            AtlasRegion {
                                rect: Rect {
                                    x: frame.frame.x as _,
                                    y: frame.frame.y as _,
                                    w: frame.frame.w as _,
                                    h: frame.frame.h as _,
                                },
                                layer: 0,
                            },
                        )
                    })
                    .collect::<HashMap<_, _>>();
                (format!("{}/page.{}.json", assets, i), frames)
            })
            .collect::<HashMap<_, _>>();
        let asset = AtlasAssetSource::Raw(pages);
        let path = target.join("atlas.json");
        write(
            &path,
            serde_json::to_string_pretty(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize atlas asset")),
        )
        .unwrap_or_else(|_| panic!("Could not write atlas asset to file: {:?}", path));
        Ok(vec![format!("atlas://{}/atlas.json", assets)])
    })
}

fn get_files_recursively(directory: impl AsRef<Path>, result: &mut Vec<PathBuf>) {
    if let Ok(entries) = read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                result.push(path);
            } else if path.is_dir() {
                get_files_recursively(path, result);
            }
        }
    }
}
