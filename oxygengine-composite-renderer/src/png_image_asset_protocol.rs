use crate::core::assets::protocol::{AssetLoadResult, AssetProtocol};
use std::io::Cursor;

pub struct PngImageAsset {
    bytes: Vec<u8>,
    width: usize,
    height: usize,
}

impl PngImageAsset {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

pub struct PngImageAssetProtocol;

impl AssetProtocol for PngImageAssetProtocol {
    fn name(&self) -> &str {
        "png"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let stream = Cursor::new(&data);
        let decoder = png::Decoder::new(stream);
        let (info, _) = decoder.read_info().unwrap();
        let width = info.width as usize;
        let height = info.height as usize;
        AssetLoadResult::Data(Box::new(PngImageAsset {
            bytes: data,
            width,
            height,
        }))
    }
}
