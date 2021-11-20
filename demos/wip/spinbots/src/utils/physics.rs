use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, PI};

pub const GRAVITY: Scalar = 9.81 * 100.0;

#[derive(Debug, Copy, Clone)]
pub enum ArenaError {
    ComplexCurvature,
}

pub enum ArenaShape {
    Circle {
        /// Circle radius.
        radius: Scalar,
    },
    Sphere {
        /// Cross-section radius.
        radius: Scalar,
        /// Cross-section height.
        height: Scalar,
    },
}

impl ArenaShape {
    pub fn new(
        cross_section_radius: Scalar,
        cross_section_angle: Scalar,
    ) -> Result<Self, ArenaError> {
        if cross_section_angle.abs() < 1.0e-4 {
            return Ok(Self::Circle {
                radius: cross_section_radius,
            });
        }
        if cross_section_angle >= FRAC_PI_2 {
            return Err(ArenaError::ComplexCurvature);
        }
        let r = cross_section_radius / cross_section_angle.sin();
        let height = (r * r - cross_section_radius * cross_section_radius).sqrt();
        Ok(Self::Sphere {
            radius: cross_section_radius,
            height,
        })
    }

    pub fn radius(&self) -> Scalar {
        match self {
            Self::Circle { radius } => *radius,
            Self::Sphere { radius, .. } => *radius,
        }
    }

    pub fn normal_2d(&self, position: Vec2) -> Option<Vec2> {
        let normal = self.normal_3d(position)?;
        Some(Vec2::new(normal.x, normal.y))
    }

    pub fn normal_3d(&self, position: Vec2) -> Option<vek::Vec3<Scalar>> {
        let dist = position.magnitude();
        match self {
            Self::Circle { radius } => {
                if dist > *radius {
                    return None;
                }
                Some(vek::Vec3::new(0.0, 0.0, 1.0))
            }
            Self::Sphere { radius, height } => {
                if dist > *radius {
                    return None;
                }
                let from = vek::Vec3::new(position.x, position.y, 0.0);
                let to = vek::Vec3::new(0.0, 0.0, *height);
                Some((to - from).normalized())
            }
        }
    }
}

pub struct Arena {
    pub shape: ArenaShape,
}
