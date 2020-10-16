use super::{HitResult, Primitive, PrimitiveHitResult, Shape, SimpleShape};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::utils::*;
use crate::RenderStatsCollector;
use crate::{BoundingBox, Material, PartialScatterResult, ScatterResult, Texture};

pub trait MediumDensity: Send + Sync + std::fmt::Debug {
    fn does_scatter(&self, ray: Ray, ray_length: FloatType) -> Option<FloatType>;
}

#[derive(Debug)]
pub struct Medium<Density: MediumDensity + Clone, Phase: Material + Clone, Child: Primitive + Clone>
{
    density: Density,
    phase: Phase,

    // We specifically do not want to decompose whatever geometry is passed in here because,
    // unlike in other places, we want to use it as a complete volume which means we care about it
    // being one thing. So, instead of smashing it up, we put it in a shape list
    child: Child,
}

impl<Density: MediumDensity + Clone, Phase: Material + Clone, Child: Primitive + Clone>
    Medium<Density, Phase, Child>
{
    pub fn new(density: Density, child: Child, phase: Phase) -> Self {
        Medium {
            density,
            child,
            phase,
        }
    }

    fn double_intersect(
        &self,
        ray: &Ray,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<(PrimitiveHitResult, PrimitiveHitResult)> {
        if let Some(hit_1) =
            self.child
                .intersect(ray, -constants::INFINITY, constants::INFINITY, stats)
        {
            if let Some(hit_2) =
                self.child
                    .intersect(ray, hit_1.distance() + 0.0001, constants::INFINITY, stats)
            {
                Some((hit_1, hit_2))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<
        Density: 'static + MediumDensity + Clone,
        Phase: 'static + Material + Clone,
        Child: Primitive + Clone,
    > Shape for Medium<Density, Phase, Child>
{
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        if let Some((hit_result_1, hit_result_2)) = self.double_intersect(ray, stats) {
            let distance_1 = hit_result_1.distance().max(t_min).max(0.0);
            let distance_2 = hit_result_2.distance().min(t_max);

            if distance_1 < distance_2 {
                let internal_ray_origin =
                    ray.origin.into_point() + (ray.direction.into_vector() * distance_1);
                let internal_ray =
                    Ray::new(internal_ray_origin, ray.direction.into_vector(), ray.time);
                let internal_ray_length = distance_2 - distance_1;

                if let Some(scatter_distance) =
                    self.density.does_scatter(internal_ray, internal_ray_length)
                {
                    Some(HitResult::new(
                        PrimitiveHitResult::new(
                            scatter_distance + distance_1,
                            ray.origin.into_point()
                                + (ray.direction.into_vector() * (scatter_distance + distance_1)),
                            vec3(1.0, 0.0, 0.0), // arbitrary
                            vec3(0.0, 1.0, 0.0), // arbitrary
                            vec3(0.0, 0.0, 1.0), // arbitrary
                            true,                // also arbitrary
                            (0.0, 0.0),
                        ),
                        &self.phase,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.child.bounding_box(t0, t1)
    }
}

impl<
        Density: 'static + MediumDensity + Clone,
        Phase: 'static + Material + Clone,
        Child: Primitive + Clone,
    > SimpleShape for Medium<Density, Phase, Child>
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

#[derive(Debug, Clone)]
pub struct Isotropic<Albedo: Texture + Clone>(Albedo);

impl<Albedo: Texture + Clone> Material for Isotropic<Albedo> {
    fn scatter(&self, ray_in: &Ray, hit_record: PrimitiveHitResult) -> Option<ScatterResult> {
        let attenuation =
            cgmath::Vector4::from(self.0.value(hit_record.hit_point(), hit_record.uv())).truncate();

        let ret = Some(ScatterResult {
            partial: PartialScatterResult { attenuation },
            scattered: Ray::new(hit_record.hit_point(), random_in_unit_sphere(), ray_in.time),
        });

        ret
    }
}

pub mod factories {
    use super::*;

    pub fn medium<
        Density: MediumDensity + Clone,
        Phase: Material + Clone,
        Child: Primitive + Clone,
    >(
        density: Density,
        child: Child,
        phase: Phase,
    ) -> Medium<Density, Phase, Child> {
        Medium::new(density, child, phase)
    }

    pub fn constant_medium<Phase: Material + Clone, Child: Primitive + Clone>(
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

    let medium: Box<dyn Shape> = Box::new(constant_medium(
        0.5,
        sphere(Point3::new(0.0, 0.0, 0.0), 1.0),
        isotropic(solid_texture(Color([1.0, 1.0, 1.0, 1.0]))),
    ));

    let ray = Ray::new(Point3::new(0.0, 0.0, -10.0), vec3(0.0, 0.0, 1.0), 0.0);

    let mut stats = crate::TracingStats::new();

    let hit_count = (0..1000000)
        .into_iter()
        .filter_map(|_| medium.intersect(&ray, 0.0001, constants::INFINITY, &mut stats))
        .map(|intersect| {
            let hit_point = intersect.hit_point();
            let distance = intersect.distance();

            assert_eq!(hit_point.x, 0.0);
            assert_eq!(hit_point.y, 0.0);

            assert!(
                hit_point.z >= -1.0 && hit_point.z <= 1.0,
                format!(
                    "hit_point.z {:?} not in range\nHIT_RESULT: {:#?}",
                    hit_point.z, intersect
                )
            );
            assert!(distance >= 9.0 && distance < 11.0);
            assert_eq!(hit_point.z, distance - 10.0);
        })
        .count();
    assert!(hit_count > 100000);
}
