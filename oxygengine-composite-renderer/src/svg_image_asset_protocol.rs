use crate::core::assets::protocol::{AssetLoadResult, AssetProtocol};
use std::str::from_utf8;
use svg::{
    node::element::tag::{Type, SVG},
    parser::Event,
};

pub struct SvgImageAsset {
    bytes: Vec<u8>,
    width: usize,
    height: usize,
}

impl SvgImageAsset {
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

pub struct SvgImageAssetProtocol;

impl AssetProtocol for SvgImageAssetProtocol {
    fn name(&self) -> &str {
        "svg"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let content = from_utf8(&data).unwrap();
        let mut width = 0;
        let mut height = 0;
        for event in svg::read(&content).unwrap() {
            if let Event::Tag(SVG, Type::Start, attributes) = event {
                let mut iter = attributes.get("viewBox").unwrap().split_whitespace();
                let left = iter.next().unwrap().parse::<isize>().unwrap();
                let top = iter.next().unwrap().parse::<isize>().unwrap();
                let right = iter.next().unwrap().parse::<isize>().unwrap();
                let bottom = iter.next().unwrap().parse::<isize>().unwrap();
                width = (right - left) as usize;
                height = (bottom - top) as usize;
                break;
            }
        }
        let content = content.replace("width=\"100%\"", &format!("width=\"{}\"", width));
        let content = content.replace("height=\"100%\"", &format!("height=\"{}\"", height));
        AssetLoadResult::Data(Box::new(SvgImageAsset {
            bytes: content.into_bytes(),
            width,
            height,
        }))
    }
}
