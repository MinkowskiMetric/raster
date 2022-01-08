use crate::{
    math::*, Bounded, BoundingBox, DefaultPrimitive, DefaultSkinnable, DefaultTransformable,
    GeometryHitResult, Intersectable, Ray, Transformable,
};

#[derive(Debug, Clone)]
pub struct UnitXyRectangle;

impl UnitXyRectangle {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnitXyRectangle {
    fn default() -> Self {
        Self::new()
    }
}

impl Intersectable for UnitXyRectangle {
    type Result = GeometryHitResult;

    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<GeometryHitResult> {
        let ray_origin = ray.origin();
        let ray_direction = ray.direction();

        let t = -ray_origin.z / ray_direction.z;
        if t < t_min || t > t_max {
            return None;
        }

        let x_intersect = ray_origin.x + (t * ray_direction.x);
        let y_intersect = ray_origin.y + (t * ray_direction.y);
        if x_intersect.abs() > 0.5 || y_intersect.abs() > 0.5 {
            return None;
        }

        let u = x_intersect + 0.5;
        let v = y_intersect + 0.5;
        let outward_normal = vec3(0.0, 0.0, 1.0);
        // How we define the tangent is up to us. It can be any vector in the plane.
        // I'm just going to point it along the specified axis
        let tangent = vec3(1.0, 0.0, 0.0);
        let front_face = ray_direction.dot(outward_normal) < 0.0;
        let surface_normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        let bitangent = surface_normal.cross(tangent);

        Some(GeometryHitResult::new(
            ray,
            t,
            surface_normal,
            tangent,
            bitangent,
            front_face,
            point2(u, v),
        ))
    }
}

impl Bounded for UnitXyRectangle {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(
            Point3::new(-0.5, -0.5, -0.0001),
            Point3::new(0.5, 0.5, 0.0001),
        )
    }
}

impl DefaultTransformable for UnitXyRectangle {}
impl DefaultSkinnable for UnitXyRectangle {}
impl DefaultPrimitive for UnitXyRectangle {}

pub mod factories {
    use super::*;

    pub fn unit_xy_rectangle() -> <UnitXyRectangle as Transformable>::Target {
        UnitXyRectangle::new().identity()
    }

    pub fn xy_rectangle(
        x_range: (FloatType, FloatType),
        y_range: (FloatType, FloatType),
        z_center: FloatType,
    ) -> <UnitXyRectangle as Transformable>::Target {
        let x_scale = x_range.1 - x_range.0;
        let y_scale = y_range.1 - y_range.0;
        let x_center = (x_range.1 + x_range.0) / 2.0;
        let y_center = (y_range.1 + y_range.0) / 2.0;

        unit_xy_rectangle()
            .nonuniform_scale(x_scale, y_scale, 1.0)
            .translate(vec3(x_center, y_center, z_center))
    }

    pub fn unit_xz_rectangle() -> <UnitXyRectangle as Transformable>::Target {
        unit_xy_rectangle().rotate_x(Deg(90.0).into())
    }

    pub fn xz_rectangle(
        x_range: (FloatType, FloatType),
        z_range: (FloatType, FloatType),
        y_center: FloatType,
    ) -> <UnitXyRectangle as Transformable>::Target {
        let x_scale = x_range.1 - x_range.0;
        let z_scale = z_range.1 - z_range.0;
        let x_center = (x_range.1 + x_range.0) / 2.0;
        let z_center = (z_range.1 + z_range.0) / 2.0;

        unit_xz_rectangle()
            .nonuniform_scale(x_scale, 1.0, z_scale)
            .translate(vec3(x_center, y_center, z_center))
    }

    pub fn unit_yz_rectangle() -> <UnitXyRectangle as Transformable>::Target {
        unit_xy_rectangle().rotate_y(Deg(90.0).into())
    }

    pub fn yz_rectangle(
        y_range: (FloatType, FloatType),
        z_range: (FloatType, FloatType),
        x_center: FloatType,
    ) -> <UnitXyRectangle as Transformable>::Target {
        let y_scale = y_range.1 - y_range.0;
        let z_scale = z_range.1 - z_range.0;
        let y_center = (y_range.1 + y_range.0) / 2.0;
        let z_center = (z_range.1 + z_range.0) / 2.0;

        unit_yz_rectangle()
            .nonuniform_scale(1.0, y_scale, z_scale)
            .translate(vec3(x_center, y_center, z_center))
    }
}
