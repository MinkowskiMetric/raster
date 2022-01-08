use super::surface_mapper::SurfaceMappingMaterial;
use crate::{
    factories::*, math::*, utils::*, Color, GeometryHitResult, IntersectResult, Material,
    PartialScatterResult, Ray, ScatterResult, SurfaceMapper, Texture,
};
use std::convert::TryInto;

#[derive(Debug)]
pub struct BumpMapper<T: Texture>(T);

impl<T: Texture + Clone> Clone for BumpMapper<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Texture> SurfaceMapper for BumpMapper<T> {
    fn process_hit_result(&self, mut hit_result: GeometryHitResult) -> GeometryHitResult {
        let t = hit_result.tangent();
        let b = hit_result.bitangent();
        let n = hit_result.surface_normal();

        let tbn = cgmath::Matrix3 { x: t, y: b, z: n };

        let normal = (Vector3::from(self.0.value(hit_result.hit_point(), hit_result.uv())) * 2.0)
            - vec3(1.0, 1.0, 1.0);
        let normal = (tbn * normal).normalize();

        hit_result.surface_normal = normal;
        hit_result
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceNormalDebugMaterial();

impl Material for SurfaceNormalDebugMaterial {
    fn scatter(&self, ray_in: &Ray, hit_record: GeometryHitResult) -> Option<ScatterResult> {
        let target = hit_record.hit_point() + hit_record.surface_normal() + random_unit_vector();
        let color = (hit_record.surface_normal() * 2.0) + vec3(1.0, 1.0, 1.0);
        let color: Color = color.extend(1.0).try_into().unwrap();
        Some(ScatterResult {
            partial: PartialScatterResult {
                attenuation: cgmath::Vector4::from(color).truncate(),
            },
            scattered: Ray::new(
                hit_record.hit_point(),
                target - hit_record.hit_point(),
                ray_in.time(),
            ),
        })
    }
}

pub mod factories {
    use super::*;

    pub fn bump_mapper<T: Texture, M: Material>(
        texture: T,
        material: M,
    ) -> SurfaceMappingMaterial<BumpMapper<T>, M> {
        map_surface(BumpMapper(texture), material)
    }

    pub fn surface_normal_debug_material() -> SurfaceNormalDebugMaterial {
        SurfaceNormalDebugMaterial()
    }
}
