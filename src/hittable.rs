use crate::aabb::BoundingBox;
use crate::material::SharedMaterial;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::stats::TracingStats;
use std::sync::Arc;

#[derive(Debug)]
pub struct HitResult<'a> {
    pub distance: FloatType,
    pub hit_point: Point3,
    pub surface_normal: Vector3,
    pub front_face: bool,
    pub material: &'a SharedMaterial,
    pub u: FloatType,
    pub v: FloatType,
}

pub trait Hittable: Sync + Send + std::fmt::Debug {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>>;

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

pub type SharedHittable = Arc<dyn Hittable>;

macro_rules! generate_rectangle {
    ($name:ident, $faxis0:ident, $faxis1:ident, $oaxis:ident) => {
        #[derive(Debug)]
        pub struct $name {
            $faxis0: (FloatType, FloatType),
            $faxis1: (FloatType, FloatType),
            $oaxis: FloatType,
            material: SharedMaterial,
        }

        impl $name {
            pub fn new(
                $faxis0: (FloatType, FloatType),
                $faxis1: (FloatType, FloatType),
                $oaxis: FloatType,
                material: SharedMaterial,
            ) -> Self {
                Self {
                    $faxis0,
                    $faxis1,
                    $oaxis,
                    material,
                }
            }
        }

        impl Hittable for $name {
            fn intersect<'a>(
                &'a self,
                ray: &Ray,
                t_min: FloatType,
                t_max: FloatType,
                _stats: &mut TracingStats,
            ) -> Option<HitResult<'a>> {
                let ray_origin = ray.origin.into_point();
                let ray_direction = ray.direction.into_vector();

                let t = (self.$oaxis - ray_origin.$oaxis) / ray_direction.$oaxis;
                if t < t_min || t > t_max {
                    return None;
                }

                let $faxis0 = ray_origin.$faxis0 + (t * ray_direction.$faxis0);
                let $faxis1 = ray_origin.$faxis1 + (t * ray_direction.$faxis1);
                if $faxis0 < self.$faxis0.0 || $faxis0 > self.$faxis0.1 || $faxis1 < self.$faxis1.0 || $faxis1 > self.$faxis1.1 {
                    return None;
                }

                let u = ($faxis0 - self.$faxis0.0) / (self.$faxis0.1 - self.$faxis0.0);
                let v = ($faxis1 - self.$faxis1.0) / (self.$faxis1.1 - self.$faxis1.0);
                let outward_normal = generate_rectangle!(make_normal $oaxis);
                let front_face = ray_direction.dot(outward_normal) < 0.0;
                let surface_normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };

                let hit_point = ray_origin + (t * ray_direction);

                Some(HitResult {
                    distance: t,
                    hit_point: hit_point.into(),
                    surface_normal: surface_normal.into(),
                    front_face,
                    material: &self.material,
                    u,
                    v,
                })
            }

            fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
                BoundingBox::new(
                    generate_rectangle!(make_bb_point self $oaxis 0 -1.0),
                    generate_rectangle!(make_bb_point self $oaxis 1 1.0),
                )
            }
        }
    };

    (make_normal x) => { Vector3::new(1.0, 0.0, 0.0) };
    (make_normal y) => { Vector3::new(0.0, 1.0, 0.0) };
    (make_normal z) => { Vector3::new(0.0, 0.0, 1.0) };

    (make_bb_point $self:ident x $idx:tt $sgn:expr) => { Point3::new($self.x + ($sgn * 0.0001), $self.y.$idx, $self.z.$idx) };
    (make_bb_point $self:ident y $idx:tt $sgn:expr) => { Point3::new($self.x.$idx, $self.y + ($sgn * 0.0001), $self.z.$idx) };
    (make_bb_point $self:ident z $idx:tt $sgn:expr) => { Point3::new($self.x.$idx, $self.y.$idx, $self.z + ($sgn * 0.0001)) };
}

generate_rectangle!(XyRectangle, x, y, z);
generate_rectangle!(XzRectangle, x, z, y);
generate_rectangle!(YzRectangle, y, z, x);

pub mod shapes {
    use super::*;
    use crate::material::SharedMaterial;
    use crate::sphere::{MovingSphere, Sphere};

    pub fn sphere(center: Point3, radius: FloatType, material: SharedMaterial) -> Arc<Sphere> {
        Arc::new(Sphere::new(center, radius, material))
    }

    pub fn moving_sphere(
        center0: (Point3, FloatType),
        center1: (Point3, FloatType),
        radius: FloatType,
        material: SharedMaterial,
    ) -> Arc<MovingSphere> {
        Arc::new(MovingSphere::new(center0, center1, radius, material))
    }

    macro_rules! generate_rectangle_func {
        ($fn_name:ident, $name:ident, $faxis0:ident, $faxis1:ident, $oaxis:ident) => {
            pub fn $fn_name(
                $faxis0: (FloatType, FloatType),
                $faxis1: (FloatType, FloatType),
                $oaxis: FloatType,
                material: SharedMaterial,
            ) -> Arc<$name> {
                Arc::new($name::new($faxis0, $faxis1, $oaxis, material))
            }
        };
    }

    generate_rectangle_func!(xy_rectangle, XyRectangle, x, y, z);
    generate_rectangle_func!(xz_rectangle, XzRectangle, x, z, y);
    generate_rectangle_func!(yz_rectangle, YzRectangle, y, z, x);
}
