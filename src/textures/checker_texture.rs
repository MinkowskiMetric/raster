use crate::math::*;
use crate::{Color, Texture};

#[derive(Debug, Clone)]
pub struct CheckerTexture<Tex1: 'static + Texture + Clone, Tex2: 'static + Texture + Clone>(
    Tex1,
    Tex2,
);

impl<Tex1: 'static + Texture + Clone, Tex2: 'static + Texture + Clone> CheckerTexture<Tex1, Tex2> {
    pub fn new(texture1: Tex1, texture2: Tex2) -> Self {
        Self(texture1, texture2)
    }

    pub fn texture1(&self) -> &Tex1 {
        &self.0
    }

    pub fn texture2(&self) -> &Tex2 {
        &self.1
    }
}

impl<Tex1: 'static + Texture + Clone, Tex2: 'static + Texture + Clone> Texture
    for CheckerTexture<Tex1, Tex2>
{
    fn value(&self, p: Point3, uv: Point2) -> Color {
        let sines = (10.0 * p.x).sin() * (10.0 * p.y).sin() * (10.0 * p.z).sin();

        if sines < 0.0 {
            self.texture1().value(p, uv)
        } else {
            self.texture2().value(p, uv)
        }
    }
}

pub mod factories {
    use super::*;

    pub fn checker_texture<Tex1: 'static + Texture + Clone, Tex2: 'static + Texture + Clone>(
        texture1: Tex1,
        texture2: Tex2,
    ) -> CheckerTexture<Tex1, Tex2> {
        CheckerTexture::new(texture1, texture2)
    }
}
