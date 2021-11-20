use oxygengine_build_tools::AssetPipelineInput;
use oxygengine_composite_renderer::{math::*, sprite_sheet_asset_protocol::*};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{read, read_dir, write},
    io::Error,
    path::{Path, PathBuf},
};
use texture_packer::{
    exporter::ImageExporter, importer::ImageImporter, texture::Texture, MultiTexturePacker,
    TexturePackerConfig,
};

#[derive(Debug, Clone, Deserialize)]
struct Params {
    #[serde(default)]
    pub paths: Vec<PathBuf>,
    #[serde(default)]
    pub output_image: PathBuf,
    #[serde(default)]
    pub output_atlas: PathBuf,
    #[serde(default = "Params::default_max_width")]
    pub max_width: u32,
    #[serde(default = "Params::default_max_height")]
    pub max_height: u32,
    #[serde(default = "Params::default_padding")]
    pub padding: u32,
    #[serde(default)]
    pub pretty: bool,
    #[serde(default)]
    pub full_names: bool,
}

impl Params {
    fn default_max_width() -> u32 {
        2048
    }

    fn default_max_height() -> u32 {
        2048
    }

    fn default_padding() -> u32 {
        2
    }
}

fn main() -> Result<(), Error> {
    let (source, destination, params) = AssetPipelineInput::<Params>::consume().unwrap();
    let mut files = HashMap::new();
    for path in &params.paths {
        let path = source.join(path);
        if path.is_file() {
            if let Ok(contents) = read(&path) {
                if let Some(path) = path.to_str() {
                    println!("* Include file: {:?}", path);
                    files.insert(path.to_owned(), contents);
                } else {
                    println!("* Cannot parse path: {:?}", path);
                }
            } else {
                println!("* Cannot read file: {:?}", path);
            }
        } else {
            scan_dir(&path, &path, &mut files)?;
        }
    }
    let config = TexturePackerConfig {
        max_width: params.max_width,
        max_height: params.max_height,
        allow_rotation: false,
        border_padding: 0,
        texture_padding: params.padding,
        trim: false,
        texture_outlines: false,
        ..Default::default()
    };
    let mut packer = MultiTexturePacker::new_skyline(config);
    for (path, bytes) in files {
        match ImageImporter::import_from_memory(&bytes) {
            Ok(image) => {
                let name = Path::new(&path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                if let Err(error) = packer.pack_own(name, image) {
                    println!(
                        "* Cannot pack image from file: {:?}. Error: {:?}",
                        path, error
                    );
                }
            }
            Err(error) => {
                println!(
                    "* Cannot read image from file: {:?}. Error: {}",
                    path, error
                );
            }
        }
    }
    for (i, page) in packer.get_pages().iter().enumerate() {
        let image_path = destination
            .join(&params.output_image)
            .with_extension(&format!("{}.png", i));
        match ImageExporter::export(page) {
            Ok(exporter) => match exporter.save_with_format(&image_path, image::ImageFormat::Png) {
                Ok(_) => {
                    let path = image_path.file_name().unwrap().to_str().unwrap().to_owned();
                    let info = SpriteSheetInfo {
                        meta: SpriteSheetInfoMeta {
                            image: format!("png://{}", path),
                            size: SpriteSheetInfoMetaSize {
                                w: page.width() as _,
                                h: page.height() as _,
                            },
                            scale: Default::default(),
                        },
                        frames: page
                            .get_frames()
                            .iter()
                            .map(|(n, f)| {
                                let frame = SpriteSheetInfoFrame {
                                    frame: Rect::new(
                                        [f.frame.x as _, f.frame.y as _].into(),
                                        [f.frame.w as _, f.frame.h as _].into(),
                                    ),
                                    ..Default::default()
                                };
                                let name = if params.full_names {
                                    n.clone()
                                } else {
                                    Path::new(n)
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap()
                                        .to_owned()
                                };
                                (name, frame)
                            })
                            .collect::<HashMap<_, _>>(),
                    };
                    let json_result = if params.pretty {
                        serde_json::to_string_pretty(&info)
                    } else {
                        serde_json::to_string(&info)
                    };
                    match json_result {
                        Ok(atlas) => {
                            let atlas_path = destination
                                .join(&params.output_atlas)
                                .with_extension(&format!("{}.json", i));
                            write(&atlas_path, atlas)?;
                            println!(
                                "  Atlas #{} is done! packed to files: {:?} and {:?}",
                                i, image_path, atlas_path
                            );
                        }
                        Err(error) => {
                            println!("* Cannot serialize atlas description. Error: {}", error);
                        }
                    }
                }
                Err(error) => {
                    println!(
                        "* Cannot save packed image to file: {:?}. Error: {}",
                        params.output_image, error
                    );
                }
            },
            Err(error) => {
                println!("* Cannot export packed image. Error: {}", error);
            }
        }
    }
    Ok(())
}

fn scan_dir(from: &Path, root: &Path, map: &mut HashMap<String, Vec<u8>>) -> Result<(), Error> {
    if from.is_dir() {
        for entry in read_dir(from)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, root, map)?;
            } else if path.is_file() {
                if let Ok(contents) = read(&path) {
                    if let Some(path) = pathdiff::diff_paths(&path, root) {
                        if let Some(path) = path.to_str() {
                            println!("* Include file: {:?} as: {:?}", root.join(path), path);
                            let name = path.to_owned().replace("\\\\", "/").replace("\\", "/");
                            map.insert(name, contents);
                        } else {
                            println!("* Cannot parse path: {:?}", root.join(path));
                        }
                    } else {
                        println!("* Cannot diff path: {:?} from root: {:?}", path, root);
                    }
                } else {
                    println!("* Cannot read file: {:?}", path);
                }
            }
        }
    }
    Ok(())
}
