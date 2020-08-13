use super::{factories::*, GeometryModifier, GeometryObject, GeometryWrapper, HitResult};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;

macro_rules! generate_rotate {

    ($name:ident, $a1:ident, $a2:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name(FloatType, FloatType);

        impl $name {
            pub fn new(angle: Rad<FloatType>) -> Self {
                Self(angle.sin(), angle.cos())
            }

            fn sin_theta(&self) -> FloatType {
                self.0
            }

            fn cos_theta(&self) -> FloatType {
                self.1
            }

            fn rotate_point(&self, p: Point3) -> Point3 {
                let a1 = (self.cos_theta() * p.$a1) - (self.sin_theta() * p.$a2);
                let a2 = (self.sin_theta() * p.$a1) + (self.cos_theta() * p.$a2);
                generate_rotate!(make_triple $a1 $a2 Point3 a1, a2, p)
            }

            fn rotate_vector(&self, p: Vector3) -> Vector3 {
                let a1 = (self.cos_theta() * p.$a1) - (self.sin_theta() * p.$a2);
                let a2 = (self.sin_theta() * p.$a1) + (self.cos_theta() * p.$a2);
                generate_rotate!(make_triple $a1 $a2 Vector3 a1, a2, p)
            }

            fn unrotate_point(&self, p: Point3) -> Point3 {
                let a1 = (self.cos_theta() * p.$a1) + (self.sin_theta() * p.$a2);
                let a2 = (-self.sin_theta() * p.$a1) + (self.cos_theta() * p.$a2);
                generate_rotate!(make_triple $a1 $a2 Point3 a1, a2, p)
            }

            fn unrotate_vector(&self, p: Vector3) -> Vector3 {
                let a1 = (self.cos_theta() * p.$a1) + (self.sin_theta() * p.$a2);
                let a2 = (-self.sin_theta() * p.$a1) + (self.cos_theta() * p.$a2);
                generate_rotate!(make_triple $a1 $a2 Vector3 a1, a2, p)
            }
        }

        impl GeometryModifier for $name {
            fn process_input_ray(&self, ray: &Ray) -> Ray {
                let origin = self.rotate_point(ray.origin.into_point());
                let direction = self.rotate_vector(ray.direction.into_vector());

                Ray::new(origin, direction, ray.time)
            }

            fn process_hit_result<'a>(&self, _original_ray: &Ray, _modified_ray: &Ray, mut hit_result: HitResult<'a>) -> HitResult<'a> {
                hit_result.hit_point = self.unrotate_point(hit_result.hit_point);
                hit_result.surface_normal = self.unrotate_vector(hit_result.surface_normal);

                hit_result
            }

            fn translate_bounding_box(&self, child_bounding_box: BoundingBox) -> BoundingBox {
                let mut pt_min = Point3::new(
                    constants::INFINITY,
                    constants::INFINITY,
                    constants::INFINITY,
                );
                let mut pt_max = Point3::new(
                    -constants::INFINITY,
                    -constants::INFINITY,
                    -constants::INFINITY,
                );

                generate_rotate!(box_axis_worker $a1 $a2 self, pt_min, pt_max, child_bounding_box);

                BoundingBox::new(pt_min, pt_max)
            }
        }
    };

    (make_triple x z $type:ident $a1:expr, $a2:expr, $p:expr) => { $type::new($a1, $p.y, $a2) };
    (make_triple x y $type:ident $a1:expr, $a2:expr, $p:expr) => { $type::new($a1, $a2, $p.z) };
    (make_triple y z $type:ident $a1:expr, $a2:expr, $p:expr) => { $type::new($p.x, $a1, $a2) };

    (box_axis_worker $a1:ident $a2:ident $self:ident, $pt_min:ident, $pt_max:ident, $bound:ident) => {
        generate_rotate!(box_axis_component $a1 $a2 $self, $pt_min, $pt_max, $bound, 0, 0, 0);
        generate_rotate!(box_axis_component $a1 $a2 $self, $pt_min, $pt_max, $bound, 0, 0, 1);
        generate_rotate!(box_axis_component $a1 $a2 $self, $pt_min, $pt_max, $bound, 0, 1, 0);
        generate_rotate!(box_axis_component $a1 $a2 $self, $pt_min, $pt_max, $bound, 0, 1, 1);
        generate_rotate!(box_axis_component $a1 $a2 $self, $pt_min, $pt_max, $bound, 1, 0, 0);
        generate_rotate!(box_axis_component $a1 $a2 $self, $pt_min, $pt_max, $bound, 1, 0, 1);
        generate_rotate!(box_axis_component $a1 $a2 $self, $pt_min, $pt_max, $bound, 1, 1, 0);
        generate_rotate!(box_axis_component $a1 $a2 $self, $pt_min, $pt_max, $bound, 1, 1, 1);
    };

    (box_axis_component $a1:ident $a2:ident $self:ident, $pt_min:ident, $pt_max:ident, $bound:ident, $xsel:tt, $ysel:tt, $zsel:tt) => {
        let pt = Point3::new(generate_rotate!(select_box_axis $bound, x, $xsel),
                             generate_rotate!(select_box_axis $bound, y, $ysel),
                             generate_rotate!(select_box_axis $bound, z, $zsel));

        let pt = $self.unrotate_point(pt);

        generate_rotate!(update_box_axis $pt_min, $pt_max, pt, x);
        generate_rotate!(update_box_axis $pt_min, $pt_max, pt, y);
        generate_rotate!(update_box_axis $pt_min, $pt_max, pt, z);
    };

    (select_box_axis $bound:ident, $axis:ident, 0) => { $bound.min_point().into_point().$axis };
    (select_box_axis $bound:ident, $axis:ident, 1) => { $bound.max_point().into_point().$axis };

    (update_box_axis $pt_min:ident, $pt_max:ident, $pt:ident, $axis:ident) => {
        $pt_min.$axis = $pt_min.$axis.min($pt.$axis);
        $pt_max.$axis = $pt_max.$axis.max($pt.$axis);
    };
}

generate_rotate!(RotateY, x, z);
generate_rotate!(RotateZ, x, y);
generate_rotate!(RotateX, y, z);

pub mod factories {
    use super::*;

    macro_rules! generate_rotate_func {
        ($fn_name:ident, $name:ident) => {
            pub fn $fn_name<T: 'static + GeometryObject + Clone>(
                angle: Rad<FloatType>,
                child: T,
            ) -> GeometryWrapper<$name, T> {
                geometry_wrapper($name::new(angle), child)
            }
        };
    }

    generate_rotate_func!(rotate_y, RotateY);
    generate_rotate_func!(rotate_z, RotateZ);
    generate_rotate_func!(rotate_x, RotateX);
}
