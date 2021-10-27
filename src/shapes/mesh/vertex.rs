use crate::math::*;

#[derive(Debug, Clone)]
pub struct TriangleVertex {
    pos: Point3,
    uv: Point2,
    surface_normal: Vector3,
    tangent: Vector3,
}

impl TriangleVertex {
    pub fn new(pos: Point3, uv: Point2, surface_normal: Vector3, tangent: Vector3) -> Self {
        Self {
            pos,
            uv,
            surface_normal,
            tangent,
        }
    }

    pub fn pos(&self) -> Point3 {
        self.pos
    }

    pub fn uv(&self) -> Point2 {
        self.uv
    }

    pub fn surface_normal(&self) -> Vector3 {
        self.surface_normal
    }

    pub fn tangent(&self) -> Vector3 {
        self.tangent
    }
}
