mod map;
mod tiled;
mod tileset;

use crate::tiled::*;
use oxygengine_build_tools::*;
use serde::Deserialize;
use std::{fs::write, io::Error, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct Params {
    #[serde(default)]
    pub input: PathBuf,
    #[serde(default)]
    pub output: PathBuf,
    #[serde(default)]
    pub spritesheets: Vec<PathBuf>,
    #[serde(default)]
    pub full_names: bool,
}

impl ParamsFromArgs for Params {}

fn main() -> Result<(), Error> {
    let (source, destination, params) = AssetPipelineInput::<Params>::consume().unwrap();
    let input = source.join(&params.input);
    let output = destination.join(&params.output);
    let spritesheets = params
        .spritesheets
        .iter()
        .map(|p| source.join(p))
        .collect::<Vec<_>>();
    let contents = build_map(input, &spritesheets, params.full_names)?;
    write(output, contents)?;
    println!("Done! map built to file: {:?}", params.output);
    Ok(())
}
