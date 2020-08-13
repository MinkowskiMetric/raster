use super::{CoreHittable, HitResult};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::TracingStats;
use crate::{BoundingBox, Material};

macro_rules! generate_rectangle {
    ($name:ident, $faxis0:ident, $faxis1:ident, $oaxis:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name<T: 'static + Material + Clone> {
            $faxis0: (FloatType, FloatType),
            $faxis1: (FloatType, FloatType),
            $oaxis: FloatType,
            material: T,
        }

        impl<T: 'static + Material + Clone> $name<T> {
            pub fn new(
                $faxis0: (FloatType, FloatType),
                $faxis1: (FloatType, FloatType),
                $oaxis: FloatType,
                material: T,
            ) -> Self {
                Self {
                    $faxis0,
                    $faxis1,
                    $oaxis,
                    material,
                }
            }
        }

        impl<T: 'static + Material + Clone> CoreHittable for $name<T> {
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

pub mod factories {
    use super::*;

    macro_rules! generate_rectangle_func {
        ($fn_name:ident, $name:ident, $faxis0:ident, $faxis1:ident, $oaxis:ident) => {
            pub fn $fn_name<T: 'static + Material + Clone>(
                $faxis0: (FloatType, FloatType),
                $faxis1: (FloatType, FloatType),
                $oaxis: FloatType,
                material: T,
            ) -> $name<T> {
                $name::new($faxis0, $faxis1, $oaxis, material)
            }
        };
    }

    generate_rectangle_func!(xy_rectangle, XyRectangle, x, y, z);
    generate_rectangle_func!(xz_rectangle, XzRectangle, x, z, y);
    generate_rectangle_func!(yz_rectangle, YzRectangle, y, z, x);
}
