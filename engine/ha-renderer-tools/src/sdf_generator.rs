use image::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteChannel {
    None,
    Distance,
    Density,
    Sharpness,
    Alpha,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct WriteChannels {
    #[serde(default = "WriteChannels::default_red")]
    pub red: WriteChannel,
    #[serde(default = "WriteChannels::default_green")]
    pub green: WriteChannel,
    #[serde(default = "WriteChannels::default_blue")]
    pub blue: WriteChannel,
    #[serde(default = "WriteChannels::default_alpha")]
    pub alpha: WriteChannel,
}

impl Default for WriteChannels {
    fn default() -> Self {
        Self {
            red: Self::default_red(),
            green: Self::default_green(),
            blue: Self::default_blue(),
            alpha: Self::default_alpha(),
        }
    }
}

impl WriteChannels {
    fn default_red() -> WriteChannel {
        WriteChannel::Distance
    }

    fn default_green() -> WriteChannel {
        WriteChannel::Density
    }

    fn default_blue() -> WriteChannel {
        WriteChannel::Sharpness
    }

    fn default_alpha() -> WriteChannel {
        WriteChannel::Alpha
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdfGenerator {
    #[serde(default = "SdfGenerator::default_resolution")]
    pub resolution: usize,
    #[serde(default = "SdfGenerator::default_threshold")]
    pub threshold: u8,
    #[serde(default)]
    pub blur: Option<f32>,
    #[serde(default)]
    pub write_channels: WriteChannels,
}

impl Default for SdfGenerator {
    fn default() -> Self {
        Self {
            resolution: Self::default_resolution(),
            threshold: Self::default_threshold(),
            blur: Default::default(),
            write_channels: Default::default(),
        }
    }
}

impl SdfGenerator {
    fn default_resolution() -> usize {
        8
    }

    fn default_threshold() -> u8 {
        127
    }

    pub fn process(&self, source: &GrayImage) -> RgbaImage {
        let resolution = self.resolution as u32;
        let sw = source.width();
        let sh = source.height();
        let r = resolution + resolution;
        let dw = sw + r;
        let dh = sh + r;
        let value_limit = self.resolution as f32;
        let r = 2 * resolution;
        let density_sharpness_limit = (r * r) as f32;
        let field = (0..(dw * dh)).map(|i| {
            let col = i % dw;
            let row = i / dw;
            let sx = col.checked_sub(resolution).filter(|v| *v < sw);
            let sy = row.checked_sub(resolution).filter(|v| *v < sh);
            let current = match (sx, sy) {
                (Some(sx), Some(sy)) => (source.get_pixel(sx, sy).0)[0],
                _ => 0,
            };
            let fx = col.saturating_sub(resolution);
            let fy = row.saturating_sub(resolution);
            let tx = (col + resolution).min(dw);
            let ty = (row + resolution).min(dh);
            let mut value_score = None;
            let mut density_score = 0;
            let mut sharpness_score = 0;
            let a = current > self.threshold;
            for x in fx..tx {
                let sx = x.checked_sub(resolution).filter(|v| *v < sw);
                for y in fy..ty {
                    let sy = y.checked_sub(resolution).filter(|v| *v < sh);
                    let item = match (sx, sy) {
                        (Some(sx), Some(sy)) => source.get_pixel(sx, sy).0[0],
                        _ => 0,
                    };
                    let b = item > self.threshold;
                    if a != b {
                        let dist = distance(col, row, x, y);
                        if dist < value_score.unwrap_or(f32::INFINITY).abs() {
                            if a {
                                value_score = Some(dist);
                            } else {
                                value_score = Some(-dist);
                            }
                        }
                    }
                    if b {
                        density_score += 1;
                    }
                    if a != b {
                        sharpness_score += 1;
                    }
                }
            }
            if value_score.is_none() && a {
                value_score = Some(value_limit);
            }
            (value_score, density_score, sharpness_score, current)
        });
        let mut result = RgbaImage::new(dw as _, dh as _);
        for (i, (v, d, s, alpha)) in field.enumerate() {
            let col = i as u32 % dw;
            let row = i as u32 / dw;
            let distance = if let Some(v) = v {
                let v = if v >= 0.0 {
                    lerp(0.5, 1.0, v / value_limit)
                } else {
                    lerp(0.5, 0.0, -v / value_limit)
                };
                (v * 255.0) as u8
            } else {
                0
            };
            let density = (255.0 * (d as f32 / density_sharpness_limit).max(0.0).min(1.0)) as u8;
            let sharpnes = (255.0 * (s as f32 / density_sharpness_limit).max(0.0).min(1.0)) as u8;
            let red = match self.write_channels.red {
                WriteChannel::None => 0,
                WriteChannel::Distance => distance,
                WriteChannel::Density => density,
                WriteChannel::Sharpness => sharpnes,
                WriteChannel::Alpha => alpha,
            };
            let green = match self.write_channels.green {
                WriteChannel::None => 0,
                WriteChannel::Distance => distance,
                WriteChannel::Density => density,
                WriteChannel::Sharpness => sharpnes,
                WriteChannel::Alpha => alpha,
            };
            let blue = match self.write_channels.blue {
                WriteChannel::None => 0,
                WriteChannel::Distance => distance,
                WriteChannel::Density => density,
                WriteChannel::Sharpness => sharpnes,
                WriteChannel::Alpha => alpha,
            };
            let alpha = match self.write_channels.alpha {
                WriteChannel::None => 0,
                WriteChannel::Distance => distance,
                WriteChannel::Density => density,
                WriteChannel::Sharpness => sharpnes,
                WriteChannel::Alpha => alpha,
            };
            result.put_pixel(col, row, Rgba([red, green, blue, alpha]));
        }
        if let Some(blur) = self.blur {
            image::imageops::blur(&result, blur)
        } else {
            result
        }
    }
}

fn distance(fx: u32, fy: u32, tx: u32, ty: u32) -> f32 {
    let dx = tx as f32 - fx as f32;
    let dy = ty as f32 - fy as f32;
    (dx * dx + dy * dy).sqrt()
}

fn lerp(from: f32, to: f32, factor: f32) -> f32 {
    from + (to - from) * factor.max(0.0).min(1.0)
}
