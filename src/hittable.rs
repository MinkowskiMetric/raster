use crate::aabb::BoundingBox;
use crate::material::SharedMaterial;
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::shape_list::ShapeList;
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
            normal: Vector3,
            material: SharedMaterial,
        }

        impl $name {
            pub fn new(
                $faxis0: (FloatType, FloatType),
                $faxis1: (FloatType, FloatType),
                $oaxis: FloatType,
                flip_normal: bool,
                material: SharedMaterial,
            ) -> Self {
                let normal = generate_rectangle!(make_normal $oaxis);
                let normal = if flip_normal {
                    -normal
                } else {
                    normal
                };

                Self {
                    $faxis0,
                    $faxis1,
                    $oaxis,
                    normal,
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
                let outward_normal = self.normal;
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

#[derive(Debug)]
pub struct BoxShape {
    pt_min: Point3,
    pt_max: Point3,
    shapes: ShapeList,
}

impl BoxShape {
    pub fn new(pt_min: Point3, pt_max: Point3, material: SharedMaterial) -> Self {
        let sides: Vec<SharedHittable> = vec![
            shapes::xy_rectangle(
                (pt_min.x, pt_max.x),
                (pt_min.y, pt_max.y),
                pt_max.z,
                false,
                material.clone(),
            ),
            shapes::xy_rectangle(
                (pt_min.x, pt_max.x),
                (pt_min.y, pt_max.y),
                pt_min.z,
                true,
                material.clone(),
            ),
            shapes::xz_rectangle(
                (pt_min.x, pt_max.x),
                (pt_min.z, pt_max.z),
                pt_max.y,
                false,
                material.clone(),
            ),
            shapes::xz_rectangle(
                (pt_min.x, pt_max.x),
                (pt_min.z, pt_max.z),
                pt_min.y,
                true,
                material.clone(),
            ),
            shapes::yz_rectangle(
                (pt_min.y, pt_max.y),
                (pt_min.z, pt_max.z),
                pt_max.x,
                false,
                material.clone(),
            ),
            shapes::yz_rectangle(
                (pt_min.y, pt_max.y),
                (pt_min.z, pt_max.z),
                pt_min.x,
                true,
                material.clone(),
            ),
        ];

        Self {
            pt_min,
            pt_max,
            shapes: ShapeList::from_shapes(sides),
        }
    }
}

impl Hittable for BoxShape {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        self.shapes.intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        BoundingBox::new(self.pt_min, self.pt_max)
    }
}

#[derive(Debug, Clone)]
pub struct Translation(Vector3, SharedHittable);

impl Translation {
    pub fn new(offset: Vector3, child: SharedHittable) -> Self {
        Self(offset, child)
    }

    fn offset(&self) -> Vector3 {
        self.0
    }

    fn child(&self) -> &dyn Hittable {
        self.1.as_ref()
    }
}

impl Hittable for Translation {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        let moved_ray = Ray::new(
            ray.origin.into_point() - self.offset(),
            ray.direction.into_vector(),
            ray.time,
        );
        if let Some(mut hit_result) = self.child().intersect(&moved_ray, t_min, t_max, stats) {
            hit_result.hit_point = hit_result.hit_point + self.offset();
            Some(hit_result)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let child_bounding_box = self.child().bounding_box(t0, t1);
        BoundingBox::new(
            (child_bounding_box.min_point().into_point() + self.offset()).into(),
            (child_bounding_box.max_point().into_point() + self.offset()).into(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RotateY(FloatType, FloatType, SharedHittable);

impl RotateY {
    pub fn new(angle: Rad<FloatType>, child: SharedHittable) -> Self {
        Self(angle.sin(), angle.cos(), child)
    }

    fn sin_theta(&self) -> FloatType {
        self.0
    }

    fn cos_theta(&self) -> FloatType {
        self.1
    }

    fn child(&self) -> &dyn Hittable {
        self.2.as_ref()
    }

    fn rotate_point(&self, p: Point3) -> Point3 {
        let x = (self.cos_theta() * p.x) - (self.sin_theta() * p.z);
        let z = (self.sin_theta() * p.x) + (self.cos_theta() * p.z);
        Point3::new(x, p.y, z)
    }

    fn rotate_vector(&self, p: Vector3) -> Vector3 {
        let x = (self.cos_theta() * p.x) - (self.sin_theta() * p.z);
        let z = (self.sin_theta() * p.x) + (self.cos_theta() * p.z);
        Vector3::new(x, p.y, z)
    }

    fn unrotate_point(&self, p: Point3) -> Point3 {
        let x = (self.cos_theta() * p.x) + (self.sin_theta() * p.z);
        let z = (-self.sin_theta() * p.x) + (self.cos_theta() * p.z);
        Point3::new(x, p.y, z)
    }

    fn unrotate_vector(&self, p: Vector3) -> Vector3 {
        let x = (self.cos_theta() * p.x) + (self.sin_theta() * p.z);
        let z = (-self.sin_theta() * p.x) + (self.cos_theta() * p.z);
        Vector3::new(x, p.y, z)
    }
}

macro_rules! box_axis_worker {
    ($self:ident, $pt_min:ident, $pt_max:ident, $bound:ident) => {
        box_axis_worker!(component $self, $pt_min, $pt_max, $bound, 0, 0, 0);
        box_axis_worker!(component $self, $pt_min, $pt_max, $bound, 0, 0, 1);
        box_axis_worker!(component $self, $pt_min, $pt_max, $bound, 0, 1, 0);
        box_axis_worker!(component $self, $pt_min, $pt_max, $bound, 0, 1, 1);
        box_axis_worker!(component $self, $pt_min, $pt_max, $bound, 1, 0, 0);
        box_axis_worker!(component $self, $pt_min, $pt_max, $bound, 1, 0, 1);
        box_axis_worker!(component $self, $pt_min, $pt_max, $bound, 1, 1, 0);
        box_axis_worker!(component $self, $pt_min, $pt_max, $bound, 1, 1, 1);
    };

    (component $self:ident, $pt_min:ident, $pt_max:ident, $bound:ident, $xsel:tt, $ysel:tt, $zsel:tt) => {
        let x = box_axis_worker!(select_axis $bound, x, $xsel);
        let y = box_axis_worker!(select_axis $bound, y, $ysel);
        let z = box_axis_worker!(select_axis $bound, z, $zsel);

        let x = ($self.cos_theta() * x) + ($self.sin_theta() * z);
        let z = (-$self.sin_theta() * x) + ($self.cos_theta() * z);

        box_axis_worker!(update_axis $pt_min, $pt_max, x);
        box_axis_worker!(update_axis $pt_min, $pt_max, y);
        box_axis_worker!(update_axis $pt_min, $pt_max, z);
    };

    (select_axis $bound:ident, $axis:ident, 0) => { $bound.min_point().into_point().$axis };
    (select_axis $bound:ident, $axis:ident, 1) => { $bound.max_point().into_point().$axis };

    (update_axis $pt_min:ident, $pt_max:ident, $axis:ident) => {
        $pt_min.$axis = $pt_min.$axis.min($axis);
        $pt_max.$axis = $pt_max.$axis.max($axis);
    }
}

impl Hittable for RotateY {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut TracingStats,
    ) -> Option<HitResult<'a>> {
        let origin = self.rotate_point(ray.origin.into_point());
        let direction = self.rotate_vector(ray.direction.into_vector());

        let moved_ray = Ray::new(origin, direction, ray.time);
        if let Some(mut hit_result) = self.child().intersect(&moved_ray, t_min, t_max, stats) {
            hit_result.hit_point = self.unrotate_point(hit_result.hit_point);
            hit_result.surface_normal = self.unrotate_vector(hit_result.surface_normal);

            Some(hit_result)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let child_bounding_box = self.child().bounding_box(t0, t1);

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

        box_axis_worker!(self, pt_min, pt_max, child_bounding_box);

        BoundingBox::new(pt_min, pt_max)
    }
}

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
                flip_normal: bool,
                material: SharedMaterial,
            ) -> Arc<$name> {
                Arc::new($name::new($faxis0, $faxis1, $oaxis, flip_normal, material))
            }
        };
    }

    generate_rectangle_func!(xy_rectangle, XyRectangle, x, y, z);
    generate_rectangle_func!(xz_rectangle, XzRectangle, x, z, y);
    generate_rectangle_func!(yz_rectangle, YzRectangle, y, z, x);

    pub fn box_shape(pt_min: Point3, pt_max: Point3, material: SharedMaterial) -> Arc<BoxShape> {
        Arc::new(BoxShape::new(pt_min, pt_max, material))
    }

    pub fn translate(offset: Vector3, child: SharedHittable) -> Arc<Translation> {
        Arc::new(Translation::new(offset, child))
    }

    pub fn rotate_y(angle: Rad<FloatType>, child: SharedHittable) -> Arc<RotateY> {
        Arc::new(RotateY::new(angle, child))
    }
}