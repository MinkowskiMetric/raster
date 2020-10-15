use super::{HitResult, Shape, SimpleShape, TransformablePrimitive, TransformableShape};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::Material;
use crate::RenderStatsCollector;

#[derive(Debug, Clone)]
pub struct SkinnedPrimitive<P: Primitive, M: Material> {
    primitive: P,
    material: M,
}

impl<P: Primitive, M: Material> Shape for SkinnedPrimitive<P, M> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        self.primitive
            .intersect(ray, t_min, t_max, stats)
            .map(|hit_result| HitResult {
                distance: hit_result.distance,
                hit_point: hit_result.hit_point,
                surface_normal: hit_result.surface_normal,
                tangent: hit_result.tangent,
                bitangent: hit_result.bitangent,
                front_face: hit_result.front_face,
                material: &self.material,
                u: hit_result.u,
                v: hit_result.v,
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.primitive.bounding_box(t0, t1)
    }
}

impl<P: TransformablePrimitive, M: Material> TransformableShape for SkinnedPrimitive<P, M> {
    type Target = SkinnedPrimitive<P::Target, M>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        Self::Target {
            primitive: self.primitive.transform(transform),
            material: self.material,
        }
    }
}

// Not sure this is the right thing to do
impl<P: Primitive, M: Material> SimpleShape for SkinnedPrimitive<P, M> {}

#[derive(Debug)]
pub struct PrimitiveHitResult {
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub tangent: Vector3,
    pub bitangent: Vector3,
    pub front_face: bool,
    pub u: FloatType,
    pub v: FloatType,
}

pub trait Primitive: Send + Sync + std::fmt::Debug {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;

    fn apply_material<M: Material>(self, material: M) -> SkinnedPrimitive<Self, M>
    where
        Self: Sized,
    {
        SkinnedPrimitive {
            primitive: self,
            material,
        }
    }
}

pub trait UntransformedPrimitive: Primitive {}
