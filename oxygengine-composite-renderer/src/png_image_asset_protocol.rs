use crate::core::assets::protocol::{AssetLoadResult, AssetProtocol};
use std::io::Cursor;

pub struct PngImageAsset {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl PngImageAsset {
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
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
        let (info, mut reader) = decoder.read_info().unwrap();
        let width = info.width as usize;
        let height = info.height as usize;
        let mut pixels = vec![0; info.buffer_size()];
        reader.next_frame(&mut pixels).unwrap();
        AssetLoadResult::Data(Box::new(PngImageAsset {
            pixels,
            width,
            height,
        }))
    }
}
