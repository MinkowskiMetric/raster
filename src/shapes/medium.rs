use crate::{
    math::*, utils::*, BoundingBox, DefaultVisible, GeometryHitResult, IntersectResult,
    Intersectable, Material, PartialScatterResult, Primitive, Ray, ScatterResult, SkinnedHitResult,
    Texture, TimeDependentBounded,
};
use std::sync::Arc;

pub trait MediumDensity: Send + Sync + std::fmt::Debug {
    fn does_scatter(&self, ray: Ray, ray_length: FloatType) -> Option<FloatType>;
}

#[derive(Debug)]
pub struct Medium<Density: MediumDensity, Phase: Material, Child: Primitive> {
    density: Density,
    phase: Arc<Phase>,
    child: Child,
}

impl<Density: MediumDensity, Phase: Material, Child: Primitive> Medium<Density, Phase, Child> {
    pub fn new(density: Density, child: Child, phase: Phase) -> Self {
        Medium {
            density,
            child,
            phase: Arc::new(phase),
        }
    }

    fn double_intersect(&self, ray: &Ray) -> Option<(GeometryHitResult, GeometryHitResult)> {
        if let Some(hit_1) = self
            .child
            .intersect(ray, -constants::INFINITY, constants::INFINITY)
        {
            self.child
                .intersect(ray, hit_1.distance() + 0.0001, constants::INFINITY)
                .map(|hit_2| (hit_1, hit_2))
        } else {
            None
        }
    }
}

impl<Density: 'static + MediumDensity, Phase: 'static + Material, Child: Primitive> Intersectable
    for Medium<Density, Phase, Child>
{
    type Result = SkinnedHitResult;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<SkinnedHitResult> {
        if let Some((hit_result_1, hit_result_2)) = self.double_intersect(ray) {
            let distance_1 = hit_result_1.distance().max(t_min).max(0.0);
            let distance_2 = hit_result_2.distance().min(t_max);

            if distance_1 < distance_2 {
                let internal_ray_origin = ray.origin() + (ray.direction() * distance_1);
                let internal_ray = Ray::new(internal_ray_origin, ray.direction(), ray.time());
                let internal_ray_length = distance_2 - distance_1;

                self.density
                    .does_scatter(internal_ray, internal_ray_length)
                    .map(|scatter_distance| {
                        SkinnedHitResult::new(
                            GeometryHitResult::new(
                                ray,
                                scatter_distance + distance_1,
                                vec3(1.0, 0.0, 0.0), // arbitrary
                                vec3(0.0, 1.0, 0.0), // arbitrary
                                vec3(0.0, 0.0, 1.0), // arbitrary
                                true,                // also arbitrary
                                point2(0.0, 0.0),
                            ),
                            self.phase.clone(),
                        )
                    })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<Density: 'static + MediumDensity, Phase: 'static + Material, Child: Primitive>
    TimeDependentBounded for Medium<Density, Phase, Child>
{
    fn time_dependent_bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.child.time_dependent_bounding_box(t0, t1)
    }
}

impl<Density: 'static + MediumDensity, Phase: 'static + Material, Child: Primitive> DefaultVisible
    for Medium<Density, Phase, Child>
{
}

#[derive(Debug, Clone)]
pub struct ConstantDensity {
    negative_inverse_density: FloatType,
}

impl ConstantDensity {
    pub fn new(density: FloatType) -> ConstantDensity {
        ConstantDensity {
            negative_inverse_density: -1.0 / density,
        }
    }
}

impl MediumDensity for ConstantDensity {
    fn does_scatter(&self, _ray: Ray, ray_length: FloatType) -> Option<FloatType> {
        let hit_distance = self.negative_inverse_density * random_in_range(0.0, 1.0).ln();
        if hit_distance <= ray_length {
            Some(hit_distance)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Isotropic<Albedo: Texture>(Albedo);

impl<Albedo: Texture + Clone> Material for Isotropic<Albedo> {
    fn scatter(&self, ray_in: &Ray, hit_record: GeometryHitResult) -> Option<ScatterResult> {
        let attenuation =
            cgmath::Vector4::from(self.0.value(hit_record.hit_point(), hit_record.uv())).truncate();

        Some(ScatterResult {
            partial: PartialScatterResult { attenuation },
            scattered: Ray::new(
                hit_record.hit_point(),
                random_in_unit_sphere(),
                ray_in.time(),
            ),
        })
    }
}

pub mod factories {
    use super::*;

    pub fn medium<Density: MediumDensity, Phase: Material, Child: Primitive>(
        density: Density,
        child: Child,
        phase: Phase,
    ) -> Medium<Density, Phase, Child> {
        Medium::new(density, child, phase)
    }

    pub fn constant_medium<Phase: Material, Child: Primitive>(
        density: FloatType,
        child: Child,
        phase: Phase,
    ) -> Medium<ConstantDensity, Phase, Child> {
        medium(ConstantDensity::new(density), child, phase)
    }

    pub fn isotropic<Albedo: Texture + Clone>(albedo: Albedo) -> Isotropic<Albedo> {
        Isotropic(albedo)
    }
}

#[test]
fn test_constant_medium_hit_points() {
    use crate::factories::*;
    use crate::Color;

    let medium = constant_medium(
        0.5,
        sphere(Point3::new(0.0, 0.0, 0.0), 1.0),
        isotropic(solid_texture(Color([1.0, 1.0, 1.0, 1.0]))),
    );

    let ray = Ray::new(Point3::new(0.0, 0.0, -10.0), vec3(0.0, 0.0, 1.0), 0.0);

    let hit_count = (0..1000000)
        .into_iter()
        .filter_map(|_| medium.intersect(&ray, 0.0001, constants::INFINITY))
        .map(|intersect| {
            let hit_point = intersect.hit_point();
            let distance = intersect.distance();

            assert_eq!(hit_point.x, 0.0);
            assert_eq!(hit_point.y, 0.0);

            assert!(
                hit_point.z >= -1.0 && hit_point.z <= 1.0,
                "hit_point.z {:?} not in range",
                hit_point.z,
            );
            assert!(distance >= 9.0 && distance < 11.0);
            assert_eq!(hit_point.z, distance - 10.0);
        })
        .count();
    assert!(hit_count > 100000);
}
