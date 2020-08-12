use crate::math::*;
use crate::{Color, SharedTexture, Texture};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CheckerTexture(SharedTexture, SharedTexture);

impl CheckerTexture {
    pub fn new(texture1: SharedTexture, texture2: SharedTexture) -> Self {
        Self(texture1, texture2)
    }

    pub fn texture1(&self) -> &dyn Texture {
        self.0.as_ref()
    }

    pub fn texture2(&self) -> &dyn Texture {
        self.1.as_ref()
    }
}

impl Texture for CheckerTexture {
    fn value(&self, p: Point3, u: FloatType, v: FloatType) -> Color {
        let sines = (10.0 * p.x).sin() * (10.0 * p.y).sin() * (10.0 * p.z).sin();

        if sines < 0.0 {
            self.texture1().value(p, u, v)
        } else {
            self.texture2().value(p, u, v)
        }
    }
}

pub mod factories {
    use super::*;

    pub fn checker_texture(
        texture1: SharedTexture,
        texture2: SharedTexture,
    ) -> Arc<CheckerTexture> {
        Arc::new(CheckerTexture::new(texture1, texture2))
    }
}
