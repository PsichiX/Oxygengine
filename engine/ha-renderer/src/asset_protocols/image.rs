use crate::image::ImageDescriptor;
use core::{
    assets::{
        asset::{Asset, AssetId},
        protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
        protocols::binary::BinaryAsset,
    },
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, str::from_utf8};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct ImageAssetSourceRawPixels {
    pub width: usize,
    pub height: usize,
    pub bytes: Vec<u8>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum ImageAssetSource {
    Color {
        #[serde(default)]
        descriptor: ImageDescriptor,
        width: usize,
        height: usize,
        color: [u8; 4],
    },
    RawPixels {
        #[serde(default)]
        descriptor: ImageDescriptor,
        bytes_path: String,
    },
    Png {
        #[serde(default)]
        descriptor: ImageDescriptor,
        bytes_path: String,
    },
    Jpeg {
        #[serde(default)]
        descriptor: ImageDescriptor,
        bytes_path: String,
    },
    Ktx2 {
        #[serde(default)]
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
            color: [255, 255, 255, 255],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub descriptor: ImageDescriptor,
    pub width: usize,
    pub height: usize,
    pub bytes: Vec<u8>,
    pub content_asset: Option<AssetId>,
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
                color,
            } => {
                let mut bytes = vec![0; width * height * 4];
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
                    bytes,
                    content_asset: None,
                }))
            }
            ImageAssetSource::RawPixels {
                descriptor,
                bytes_path,
            }
            | ImageAssetSource::Png {
                descriptor,
                bytes_path,
            }
            | ImageAssetSource::Jpeg {
                descriptor,
                bytes_path,
            }
            | ImageAssetSource::Ktx2 {
                descriptor,
                bytes_path,
            } => AssetLoadResult::Yield(
                Some(Box::new(descriptor)),
                vec![("".to_owned(), format!("bin://{}", bytes_path))],
            ),
        }
    }

    fn on_resume(&mut self, meta: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let asset = match list.first() {
            Some(asset) => asset.1,
            None => return AssetLoadResult::Error("No image binary data loaded".to_owned()),
        };
        let ext = asset.path().rsplit_once('.').map(|ext| ext.1);
        match ext {
            Some("bin") => {
                if let Some(bytes) = asset.get::<BinaryAsset>() {
                    let ImageAssetSourceRawPixels {
                        width,
                        height,
                        bytes,
                    } = bincode::deserialize(bytes.get()).unwrap();
                    AssetLoadResult::Data(Box::new(ImageAsset {
                        descriptor: *meta.unwrap().downcast::<ImageDescriptor>().unwrap(),
                        width,
                        height,
                        bytes,
                        content_asset: Some(asset.id()),
                    }))
                } else {
                    AssetLoadResult::Error(format!(
                        "Could not read binary asset: {:?}",
                        asset.to_full_path()
                    ))
                }
            }
            Some("png") => {
                if let Some(bytes) = asset.get::<BinaryAsset>() {
                    let stream = Cursor::new(bytes.get());
                    let decoder = png::Decoder::new(stream);
                    let mut reader = decoder.read_info().unwrap();
                    let mut bytes = vec![0; reader.output_buffer_size()];
                    let info = reader.next_frame(&mut bytes).unwrap();
                    let width = info.width as usize;
                    let height = info.height as usize;
                    AssetLoadResult::Data(Box::new(ImageAsset {
                        descriptor: *meta.unwrap().downcast::<ImageDescriptor>().unwrap(),
                        width,
                        height,
                        bytes,
                        content_asset: Some(asset.id()),
                    }))
                } else {
                    AssetLoadResult::Error(format!(
                        "Could not read binary asset: {:?}",
                        asset.to_full_path()
                    ))
                }
            }
            Some("jpg") | Some("jpeg") => {
                if let Some(bytes) = asset.get::<BinaryAsset>() {
                    let stream = Cursor::new(bytes.get());
                    let mut decoder = jpeg_decoder::Decoder::new(stream);
                    let bytes = decoder.decode().unwrap();
                    let info = decoder.info().unwrap();
                    let width = info.width as usize;
                    let height = info.height as usize;
                    AssetLoadResult::Data(Box::new(ImageAsset {
                        descriptor: *meta.unwrap().downcast::<ImageDescriptor>().unwrap(),
                        width,
                        height,
                        bytes,
                        content_asset: Some(asset.id()),
                    }))
                } else {
                    AssetLoadResult::Error(format!(
                        "Could not read binary asset: {:?}",
                        asset.to_full_path()
                    ))
                }
            }
            Some("ktx2") => {
                if let Some(bytes) = asset.get::<BinaryAsset>() {
                    let reader =
                        ktx2::Reader::new(bytes.get()).expect("Could not create KTX2 decoder");
                    let header = reader.header();
                    let bytes = reader.levels().next().map(|bytes| bytes.to_owned());
                    if let Some(bytes) = bytes {
                        AssetLoadResult::Data(Box::new(ImageAsset {
                            descriptor: *meta.unwrap().downcast::<ImageDescriptor>().unwrap(),
                            width: header.pixel_width as _,
                            height: header.pixel_height as _,
                            bytes,
                            content_asset: Some(asset.id()),
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
            _ => AssetLoadResult::Error(format!("Unsupported extension: {:?}", ext)),
        }
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        asset
            .get::<ImageAsset>()
            .unwrap()
            .content_asset
            .map(|id| vec![AssetVariant::Id(id)])
    }
}
