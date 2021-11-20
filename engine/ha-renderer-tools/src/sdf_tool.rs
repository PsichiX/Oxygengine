mod sdf_generator;

use crate::sdf_generator::*;
use image::*;
use oxygengine_build_tools::AssetPipelineInput;
use oxygengine_ha_renderer::asset_protocols::image::*;
use serde::Deserialize;
use std::{
    fs::{create_dir_all, write},
    io::Error,
    path::PathBuf,
};

#[derive(Debug, Clone, Deserialize)]
enum ValueSource {
    Saturation,
    Red,
    Green,
    Blue,
    Alpha,
    Custom {
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    },
}

impl Default for ValueSource {
    fn default() -> Self {
        Self::Alpha
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Params {
    pub input_images: Vec<PathBuf>,
    #[serde(default)]
    pub output: PathBuf,
    #[serde(default)]
    pub assets_path_prefix: String,
    #[serde(default)]
    pub generator: SdfGenerator,
    #[serde(default)]
    pub value_source: ValueSource,
}

fn main() -> Result<(), Error> {
    let (source, destination, params) = AssetPipelineInput::<Params>::consume().unwrap();
    let output = destination.join(&params.output);
    create_dir_all(&output)?;

    for path in &params.input_images {
        let path = source.join(path);
        let filename = path.with_extension("");
        let filename = filename
            .file_name()
            .unwrap_or_else(|| panic!("Could not get image file name: {:?}", path));
        let image = image::open(&path)
            .unwrap_or_else(|_| panic!("Could not load image: {:?}", path))
            .into_rgba8();
        let image = match &params.value_source {
            ValueSource::Saturation => DynamicImage::ImageRgba8(image).into_luma8(),
            ValueSource::Red => preprocess_image(image, 1.0, 0.0, 0.0, 0.0),
            ValueSource::Green => preprocess_image(image, 0.0, 1.0, 0.0, 0.0),
            ValueSource::Blue => preprocess_image(image, 0.0, 0.0, 1.0, 0.0),
            ValueSource::Alpha => preprocess_image(image, 0.0, 0.0, 0.0, 1.0),
            ValueSource::Custom {
                red,
                green,
                blue,
                alpha,
            } => preprocess_image(image, *red, *green, *blue, *alpha),
        };
        let image = params.generator.process(&image);

        let image_name = filename.to_str().unwrap();
        let path = output.join(format!("{}.png", image_name));
        image
            .save_with_format(&path, ImageFormat::Png)
            .unwrap_or_else(|_| panic!("Could not save image: {:?}", path));

        let asset = ImageAssetSource::Png {
            bytes_path: format!("{}{}.png", params.assets_path_prefix, image_name),
            descriptor: Default::default(),
        };
        let path = output.join(format!("{}.yaml", image_name));
        write(
            &path,
            serde_yaml::to_string(&asset).expect("Could not serialize image asset"),
        )
        .unwrap_or_else(|_| panic!("Could not write image asset to file: {:?}", path));
    }
    Ok(())
}

fn preprocess_image(image: RgbaImage, red: f32, green: f32, blue: f32, alpha: f32) -> GrayImage {
    let mut result = GrayImage::new(image.width(), image.height());
    for col in 0..image.width() {
        for row in 0..image.height() {
            let [r, g, b, a] = image.get_pixel(col, row).0;
            let value = (r as f32 * red + g as f32 * green + b as f32 * blue + a as f32 * alpha)
                .max(0.0)
                .min(255.0) as u8;
            result.put_pixel(col, row, Luma([value]));
        }
    }
    result
}
