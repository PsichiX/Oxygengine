use crate::core::assets::protocol::{AssetLoadResult, AssetProtocol};
use std::io::Cursor;

pub struct JpgImageAsset {
    bytes: Vec<u8>,
    width: usize,
    height: usize,
}

impl JpgImageAsset {
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

pub struct JpgImageAssetProtocol;

impl AssetProtocol for JpgImageAssetProtocol {
    fn name(&self) -> &str {
        "jpg"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let stream = Cursor::new(&data);
        let mut decoder = jpeg_decoder::Decoder::new(stream);
        decoder.read_info().unwrap();
        let info = decoder.info().unwrap();
        let width = info.width as usize;
        let height = info.height as usize;
        AssetLoadResult::Data(Box::new(JpgImageAsset {
            bytes: data,
            width,
            height,
        }))
    }
}
