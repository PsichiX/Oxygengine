use crate::core::assets::protocol::{AssetLoadResult, AssetProtocol};
use std::io::Cursor;

pub struct PngImageAsset {
    bytes: Vec<u8>,
    // pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl PngImageAsset {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    // pub fn pixels(&self) -> &[u8] {
    //     &self.pixels
    // }

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
        // let (info, mut reader) = decoder.read_info().unwrap();
        let (info, _) = decoder.read_info().unwrap();
        let width = info.width as usize;
        let height = info.height as usize;
        // let mut pixels = vec![0; info.buffer_size()];
        // reader.next_frame(&mut pixels).unwrap();
        // let pixels = match info.color_type {
        //     png::ColorType::RGB => pixels
        //         .windows(3)
        //         .flat_map(|ch| vec![ch[0], ch[1], ch[2], 255])
        //         .collect(),
        //     png::ColorType::RGBA => pixels,
        //     png::ColorType::Grayscale => pixels
        //         .into_iter()
        //         .flat_map(|ch| vec![ch, ch, ch, 255])
        //         .collect(),
        //     png::ColorType::GrayscaleAlpha => pixels
        //         .windows(2)
        //         .flat_map(|ch| vec![ch[0], ch[0], ch[0], ch[1]])
        //         .collect(),
        //     _ => std::iter::repeat(0).take(width * height * 4).collect(),
        // };
        AssetLoadResult::Data(Box::new(PngImageAsset {
            bytes: data,
            // pixels,
            width,
            height,
        }))
    }
}
