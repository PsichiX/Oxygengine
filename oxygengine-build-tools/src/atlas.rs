use crate::utils::scan_dir;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{read, write},
    io::Error,
    path::Path,
};
use texture_packer::{
    exporter::ImageExporter, importer::ImageImporter, texture::Texture, MultiTexturePacker,
    TexturePackerConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpriteSheetInfo {
    pub meta: SpriteSheetInfoMeta,
    pub frames: HashMap<String, SpriteSheetInfoFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpriteSheetInfoMeta {
    pub image: String,
    pub size: SpriteSheetInfoMetaSize,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct SpriteSheetInfoMetaSize {
    pub w: usize,
    pub h: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpriteSheetInfoFrame {
    pub frame: Rect,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn pack_sprites_and_write_to_files<P: AsRef<Path>>(
    paths: &[P],
    output_image: P,
    output_atlas: P,
    max_width: usize,
    max_height: usize,
    padding: usize,
    pretty: bool,
    full_names: bool,
    quiet: bool,
) -> Result<(), Error> {
    let mut files = HashMap::new();
    for path in paths {
        let path = path.as_ref();
        if path.is_file() {
            if let Ok(contents) = read(&path) {
                if let Some(path) = path.to_str() {
                    if !quiet {
                        println!("* Include file: {:?}", path);
                    }
                    files.insert(path.to_owned(), contents);
                } else if !quiet {
                    println!("* Cannot parse path: {:?}", path);
                }
            } else if !quiet {
                println!("* Cannot read file: {:?}", path);
            }
        } else {
            scan_dir(path, path, &mut files, quiet)?;
        }
    }
    let config = TexturePackerConfig {
        max_width: max_width as u32,
        max_height: max_height as u32,
        allow_rotation: false,
        border_padding: 0,
        texture_padding: padding as u32,
        trim: false,
        texture_outlines: false,
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
                    if !quiet {
                        println!(
                            "* Cannot pack image from file: {:?}. Error: {:?}",
                            path, error
                        );
                    }
                }
            }
            Err(error) => {
                if !quiet {
                    println!(
                        "* Cannot read image from file: {:?}. Error: {}",
                        path, error
                    );
                }
            }
        }
    }
    for (i, page) in packer.get_pages().iter().enumerate() {
        let image_path = output_image.as_ref().with_extension(&format!("{}.png", i));
        match ImageExporter::export(page) {
            Ok(exporter) => match exporter.save_with_format(&image_path, image::ImageFormat::Png) {
                Ok(_) => {
                    let path = image_path.file_name().unwrap().to_str().unwrap().to_owned();
                    let info = SpriteSheetInfo {
                        meta: SpriteSheetInfoMeta {
                            image: format!("png://{}", path),
                            size: SpriteSheetInfoMetaSize {
                                w: page.width() as usize,
                                h: page.height() as usize,
                            },
                        },
                        frames: page
                            .get_frames()
                            .iter()
                            .map(|(n, f)| {
                                let frame = SpriteSheetInfoFrame {
                                    frame: Rect::new(
                                        f.frame.x as usize,
                                        f.frame.y as usize,
                                        f.frame.w as usize,
                                        f.frame.h as usize,
                                    ),
                                };
                                let name = if full_names {
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
                    let json_result = if pretty {
                        serde_json::to_string_pretty(&info)
                    } else {
                        serde_json::to_string(&info)
                    };
                    match json_result {
                        Ok(atlas) => {
                            let atlas_path =
                                output_atlas.as_ref().with_extension(&format!("{}.json", i));
                            write(&atlas_path, atlas)?;
                            if !quiet {
                                println!(
                                    "  Atlas #{} is done! packed to files: {:?} and {:?}",
                                    i, image_path, atlas_path
                                );
                            }
                        }
                        Err(error) => {
                            if !quiet {
                                println!("* Cannot serialize atlas description. Error: {}", error);
                            }
                        }
                    }
                }
                Err(error) => {
                    if !quiet {
                        println!(
                            "* Cannot save packed image to file: {:?}. Error: {}",
                            output_image.as_ref(),
                            error
                        );
                    }
                }
            },
            Err(error) => {
                if !quiet {
                    println!("* Cannot export packed image. Error: {}", error);
                }
            }
        }
    }
    Ok(())
}
