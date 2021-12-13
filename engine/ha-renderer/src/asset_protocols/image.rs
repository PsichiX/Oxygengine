use crate::image::{ImageDescriptor, ImageMode};
use core::{
    assets::{
        asset::{Asset, AssetId},
        protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
        protocols::binary::BinaryAsset,
    },
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

fn default_depth() -> usize {
    1
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct ImageAssetSourceRawPixels {
    pub width: usize,
    pub height: usize,
    #[serde(default = "default_depth")]
    pub depth: usize,
    pub bytes: Vec<u8>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum ImageAssetSource {
    Color {
        descriptor: ImageDescriptor,
        width: usize,
        height: usize,
        #[serde(default = "default_depth")]
        depth: usize,
        color: [u8; 4],
    },
    RawPixels {
        descriptor: ImageDescriptor,
        bytes_path: String,
    },
    Png {
        descriptor: ImageDescriptor,
        bytes_paths: Vec<String>,
    },
    Jpeg {
        descriptor: ImageDescriptor,
        bytes_paths: Vec<String>,
    },
    Ktx2 {
        descriptor: ImageDescriptor,
        bytes_path: String,
    },
}

impl Default for ImageAssetSource {
    fn default() -> Self {
        Self::Color {
            descriptor: Default::default(),
            width: 1,
            height: 1,
            depth: 1,
            color: [255, 255, 255, 255],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub descriptor: ImageDescriptor,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub bytes: Vec<u8>,
    pub content_assets: Vec<AssetId>,
}

pub struct ImageAssetProtocol;

impl AssetProtocol for ImageAssetProtocol {
    fn name(&self) -> &str {
        "image"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match serde_yaml::from_str(data).unwrap() {
            ImageAssetSource::Color {
                descriptor,
                width,
                height,
                depth,
                color,
            } => {
                let mut bytes = vec![0; width * height * depth * 4];
                for chunk in bytes.chunks_mut(4) {
                    chunk[0] = color[0];
                    chunk[1] = color[1];
                    chunk[2] = color[2];
                    chunk[3] = color[3];
                }
                AssetLoadResult::Data(Box::new(ImageAsset {
                    descriptor,
                    width,
                    height,
                    depth,
                    bytes,
                    content_assets: vec![],
                }))
            }
            ImageAssetSource::RawPixels {
                descriptor,
                bytes_path,
            } => AssetLoadResult::Yield(
                Some(Box::new((DataType::Raw, descriptor))),
                vec![("".to_owned(), format!("bin://{}", bytes_path))],
            ),
            ImageAssetSource::Png {
                descriptor,
                bytes_paths,
            } => {
                if descriptor.mode == ImageMode::Image2d && bytes_paths.len() > 1 {
                    return AssetLoadResult::Error(
                        "PNG Image2d mode can read data from only one binary asset!".to_owned(),
                    );
                }
                AssetLoadResult::Yield(
                    Some(Box::new((DataType::Png, descriptor))),
                    bytes_paths
                        .into_iter()
                        .map(|path| (path.to_owned(), format!("bin://{}", path)))
                        .collect(),
                )
            }
            ImageAssetSource::Jpeg {
                descriptor,
                bytes_paths,
            } => {
                if descriptor.mode == ImageMode::Image2d && bytes_paths.len() > 1 {
                    return AssetLoadResult::Error(
                        "JPEG Image2d mode can read data from only one binary asset!".to_owned(),
                    );
                }
                AssetLoadResult::Yield(
                    Some(Box::new((DataType::Jpeg, descriptor))),
                    bytes_paths
                        .into_iter()
                        .map(|path| (path.to_owned(), format!("bin://{}", path)))
                        .collect(),
                )
            }
            ImageAssetSource::Ktx2 {
                descriptor,
                bytes_path,
            } => AssetLoadResult::Yield(
                Some(Box::new((DataType::Ktx2, descriptor))),
                vec![("".to_owned(), format!("bin://{}", bytes_path))],
            ),
        }
    }

    fn on_resume(&mut self, meta: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let (data_type, descriptor) = *meta
            .unwrap()
            .downcast::<(DataType, ImageDescriptor)>()
            .unwrap();
        match data_type {
            DataType::Raw => {
                let asset = match list.first() {
                    Some(asset) => asset.1,
                    None => {
                        return AssetLoadResult::Error("No image binary data loaded".to_owned())
                    }
                };
                if let Some(bytes) = asset.get::<BinaryAsset>() {
                    let ImageAssetSourceRawPixels {
                        width,
                        height,
                        depth,
                        bytes,
                    } = bincode::deserialize(bytes.get()).unwrap();
                    AssetLoadResult::Data(Box::new(ImageAsset {
                        descriptor,
                        width,
                        height,
                        depth,
                        bytes,
                        content_assets: vec![asset.id()],
                    }))
                } else {
                    AssetLoadResult::Error(format!(
                        "Could not read binary asset: {:?}",
                        asset.to_full_path()
                    ))
                }
            }
            DataType::Png => {
                let mut width = 0;
                let mut height = 0;
                let mut bytes = vec![];
                let mut offset = 0;
                let mut content_assets = Vec::with_capacity(list.len());
                for (_, asset) in list {
                    if let Some(data) = asset.get::<BinaryAsset>() {
                        let decoder = png::Decoder::new(data.get());
                        let mut reader = decoder.read_info().unwrap();
                        let info = reader.info();
                        if info.interlaced {
                            return AssetLoadResult::Error(format!(
                                "Trying to load interlaced PNG: {:?}",
                                asset.to_full_path()
                            ));
                        }
                        let w = info.width as usize;
                        let h = info.height as usize;
                        let size = reader.output_buffer_size();
                        if width == 0 || height == 0 {
                            width = w;
                            height = h;
                            bytes = vec![0; size * list.len()];
                        } else if w != width || h != height {
                            return AssetLoadResult::Error(format!(
                                "PNG: {:?} image doesn't have expected resolution: {} x {}",
                                asset.to_full_path(),
                                width,
                                height,
                            ));
                        }
                        reader
                            .next_frame(&mut bytes[offset..(offset + size)])
                            .unwrap();
                        offset += size;
                        content_assets.push(asset.id());
                    } else {
                        return AssetLoadResult::Error(format!(
                            "Trying to read non-binary asset as PNG: {:?}",
                            asset.to_full_path()
                        ));
                    }
                }
                AssetLoadResult::Data(Box::new(ImageAsset {
                    descriptor,
                    width,
                    height,
                    depth: list.len(),
                    bytes,
                    content_assets,
                }))
            }
            DataType::Jpeg => {
                let mut width = 0;
                let mut height = 0;
                let mut bytes = vec![];
                let mut content_assets = Vec::with_capacity(list.len());
                for (_, asset) in list {
                    if let Some(data) = asset.get::<BinaryAsset>() {
                        let mut decoder = jpeg_decoder::Decoder::new(data.get());
                        decoder.read_info().unwrap();
                        let info = decoder.info().unwrap();
                        let w = info.width as usize;
                        let h = info.height as usize;
                        let data = decoder.decode().unwrap();
                        let size = bytes.len();
                        if width == 0 || height == 0 {
                            width = w;
                            height = h;
                            bytes = vec![0; size * list.len()];
                        } else if w != width || h != height {
                            return AssetLoadResult::Error(format!(
                                "JPEG: {:?} image doesn't have expected resolution: {} x {}",
                                asset.to_full_path(),
                                width,
                                height,
                            ));
                        }
                        bytes.extend(data);
                        content_assets.push(asset.id());
                    } else {
                        return AssetLoadResult::Error(format!(
                            "Trying to read non-binary asset as JPEG: {:?}",
                            asset.to_full_path()
                        ));
                    }
                }
                AssetLoadResult::Data(Box::new(ImageAsset {
                    descriptor,
                    width,
                    height,
                    depth: list.len(),
                    bytes,
                    content_assets,
                }))
            }
            DataType::Ktx2 => {
                let asset = match list.first() {
                    Some(asset) => asset.1,
                    None => {
                        return AssetLoadResult::Error("No image binary data loaded".to_owned())
                    }
                };
                if let Some(bytes) = asset.get::<BinaryAsset>() {
                    let reader =
                        ktx2::Reader::new(bytes.get()).expect("Could not create KTX2 decoder");
                    let header = reader.header();
                    let bytes = reader.levels().next().map(|b| b.to_owned());
                    if let Some(bytes) = bytes {
                        AssetLoadResult::Data(Box::new(ImageAsset {
                            descriptor,
                            width: header.pixel_width as _,
                            height: header.pixel_height as _,
                            depth: header.pixel_depth as _,
                            bytes,
                            content_assets: vec![asset.id()],
                        }))
                    } else {
                        AssetLoadResult::Error(format!(
                            "Could not decode KTX2 image: {:?}",
                            asset.to_full_path()
                        ))
                    }
                } else {
                    AssetLoadResult::Error(format!(
                        "Could not read binary asset: {:?}",
                        asset.to_full_path()
                    ))
                }
            }
        }
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        Some(
            asset
                .get::<ImageAsset>()
                .unwrap()
                .content_assets
                .iter()
                .map(|id| AssetVariant::Id(*id))
                .collect(),
        )
    }
}

enum DataType {
    Raw,
    Png,
    Jpeg,
    Ktx2,
}
