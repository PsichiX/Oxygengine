use crate::resources::{NavGrid, NavResult};
use bincode::deserialize;
use core::assets::protocol::{AssetLoadResult, AssetProtocol};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NavGridAsset {
    cols: usize,
    rows: usize,
    cells: Vec<bool>,
}

impl NavGridAsset {
    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cells(&self) -> &[bool] {
        &self.cells
    }

    pub fn build_nav_grid(&self) -> NavResult<NavGrid> {
        NavGrid::new(self.cols, self.rows, self.cells.clone())
    }
}

pub struct NavGridAssetProtocol;

impl AssetProtocol for NavGridAssetProtocol {
    fn name(&self) -> &str {
        "navgrid"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        match deserialize::<NavGridAsset>(&data) {
            Ok(asset) => AssetLoadResult::Data(Box::new(asset)),
            Err(error) => {
                AssetLoadResult::Error(format!("Error loading navgrid asset: {:?}", error))
            }
        }
    }
}
