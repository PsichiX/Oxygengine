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
    path::PathBuf,
};
use texture_packer::{exporter::ImageExporter, MultiTexturePacker, TexturePackerConfig};

#[derive(Debug, Clone, Deserialize)]
struct Params {
    pub input_directories: Vec<PathBuf>,
    #[serde(default)]
    pub output: PathBuf,
    #[serde(default)]
    pub assets_path_prefix: String,
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
    let (source, destination, params) = AssetPipelineInput::<Params>::consume().unwrap();
    let output = destination.join(&params.output);
    create_dir_all(&output)?;

    for directory in &params.input_directories {
        let directory = source.join(directory);
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

        if !directory.is_dir() {
            panic!("Path is not a valid directory: {:?}", directory);
        }
        let entries = read_dir(&directory)
            .unwrap_or_else(|_| panic!("Directory could not be read: {:?}", directory));
        for entry in entries {
            let entry = entry
                .unwrap_or_else(|_| panic!("Could not read entry in directory: {:?}", directory,));
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
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
        let name = directory
            .file_name()
            .unwrap_or_else(|| panic!("Coult not get directory name: {:?}", directory))
            .to_str()
            .unwrap();
        let pages = packer
            .get_pages()
            .iter()
            .enumerate()
            .map(|(i, page)| {
                let path = output.join(&name).with_extension(&format!("{}.png", i));
                ImageExporter::export(page)
                    .unwrap_or_else(|_| panic!("Could not export atlas: {:?} page: {}", name, i))
                    .save_with_format(&path, image::ImageFormat::Png)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Could not save atlas: {:?} page: {} to file: {:?}",
                            name, i, path
                        )
                    });
                let asset = ImageAssetSource::Png {
                    bytes_paths: vec![format!("{}{}.{}.png", params.assets_path_prefix, name, i)],
                    descriptor: Default::default(),
                };
                let path = output.join(&name).with_extension(&format!("{}.yaml", i));
                write(
                    &path,
                    serde_yaml::to_string(&asset).unwrap_or_else(|_| {
                        panic!(
                            "Could not serialize atlas: {:?} page: {} image asset",
                            name, i
                        )
                    }),
                )
                .unwrap_or_else(|_| {
                    panic!(
                        "Could not write atlas: {:?} page: {} image asset to file: {:?}",
                        name, i, path
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
                (
                    format!("{}{}.{}.yaml", params.assets_path_prefix, name, i),
                    frames,
                )
            })
            .collect::<HashMap<_, _>>();
        let asset = AtlasAssetSource::Raw(pages);
        let path = output.join(&name).with_extension("yaml");
        write(
            &path,
            serde_yaml::to_string(&asset)
                .unwrap_or_else(|_| panic!("Could not serialize atlas: {:?} asset", name)),
        )
        .unwrap_or_else(|_| {
            panic!(
                "Could not write atlas: {:?} asset to file: {:?}",
                name, path
            )
        });
    }
    Ok(())
}
