use crate::{math::*, Color, Material, PrimitiveHitResult, Ray, ScatterResult};

pub trait SurfaceMapper: Send + Sync + std::fmt::Debug {
    fn process_hit_result(&self, hit_result: PrimitiveHitResult) -> PrimitiveHitResult;
    fn process_scatter_result(&self, scatter_result: ScatterResult) -> ScatterResult {
        scatter_result
    }
}

#[derive(Debug)]
pub struct SurfaceMappingMaterial<T: SurfaceMapper, M: Material>(T, M);

impl<T: SurfaceMapper + Clone, M: Material + Clone> Clone for SurfaceMappingMaterial<T, M> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T: SurfaceMapper, M: Material> Material for SurfaceMappingMaterial<T, M> {
    fn scatter(&self, ray_in: &Ray, hit_record: PrimitiveHitResult) -> Option<ScatterResult> {
        let mapped_hit_record = self.0.process_hit_result(hit_record);

        if let Some(scatter_result) = self.1.scatter(ray_in, mapped_hit_record) {
            Some(self.0.process_scatter_result(scatter_result))
        } else {
            None
        }
    }

    fn emitted(&self, p: Point3, uv: (FloatType, FloatType)) -> Color {
        self.1.emitted(p, uv)
    }
}

pub mod factories {
    use super::*;

    pub fn map_surface<T: SurfaceMapper, M: Material>(
        surface_mapper: T,
        material: M,
    ) -> SurfaceMappingMaterial<T, M> {
        SurfaceMappingMaterial(surface_mapper, material)
    }
}
