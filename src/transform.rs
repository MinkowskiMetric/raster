use crate::{
    math::*, BoundingBox, CompoundPrimitive, CompoundVisible, DynPrimitive, DynVisible,
    GeometryHitResult, IntersectResult, Intersectable, Primitive, Ray, Skinnable, SkinnedHitResult,
    TimeDependentBounded, Visible,
};

trait InstantGeometryTransform {
    fn transform_ray(&self, ray: &Ray) -> Ray;
    fn transform_hit_result<I: IntersectResult>(&self, original_ray: &Ray, hit_result: I) -> I;
    fn transform_bounding_box(&self, bounding_box: BoundingBox) -> BoundingBox;
}

trait GeometryTransform {
    type Instant: InstantGeometryTransform;

    fn transform_at_t(&self, t: FloatType) -> Self::Instant;
}

#[derive(Clone, Debug)]
struct StaticTransform {
    transform: Matrix4,
    inverse: Matrix4,
}

impl StaticTransform {
    pub fn new(transform: Matrix4) -> Self {
        let inverse = transform.inverse_transform().unwrap();
        Self { transform, inverse }
    }

    pub fn concat(self, transform: Matrix4) -> Self {
        let transform = transform.concat(&self.transform);
        let inverse = transform.inverse_transform().unwrap();
        Self { transform, inverse }
    }
}

impl InstantGeometryTransform for StaticTransform {
    fn transform_ray(&self, ray: &Ray) -> Ray {
        let transformed_origin = self.inverse.transform_point(ray.origin);
        let transformed_direction = self.inverse.transform_vector(ray.direction);
        Ray::new(transformed_origin, transformed_direction, ray.time)
    }

    fn transform_hit_result<I: IntersectResult>(&self, original_ray: &Ray, mut hit_result: I) -> I {
        hit_result.set_hit_point(self.transform.transform_point(hit_result.hit_point()));
        hit_result.set_surface_normal(
            self.transform
                .transform_vector(hit_result.surface_normal())
                .normalize(),
        );
        hit_result.set_tangent(
            self.transform
                .transform_vector(hit_result.tangent())
                .normalize(),
        );
        hit_result.set_bitangent(
            self.transform
                .transform_vector(hit_result.bitangent())
                .normalize(),
        );
        hit_result.set_distance((hit_result.hit_point() - original_ray.origin).magnitude());

        hit_result
    }

    fn transform_bounding_box(&self, bounding_box: BoundingBox) -> BoundingBox {
        let shape_bounding_box_corners = bounding_box.all_corners();

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

        for corner in shape_bounding_box_corners.iter() {
            let corner = self.transform.transform_point(*corner);

            pt_min.x = pt_min.x.min(corner.x);
            pt_min.y = pt_min.y.min(corner.y);
            pt_min.z = pt_min.z.min(corner.z);

            pt_max.x = pt_max.x.max(corner.x);
            pt_max.y = pt_max.y.max(corner.y);
            pt_max.z = pt_max.z.max(corner.z);
        }

        BoundingBox::new(pt_min, pt_max)
    }
}

impl GeometryTransform for StaticTransform {
    type Instant = Self;

    fn transform_at_t(&self, _t: FloatType) -> Self::Instant {
        self.clone()
    }
}

pub trait Transformable: Intersectable + Sized {
    type Target: Intersectable<Result = Self::Result> + Skinnable + Transformable;

    fn transform(self, transform: Matrix4) -> Self::Target;

    fn translate(self, v: Vector3) -> Self::Target {
        self.transform(Matrix4::from_translation(v))
    }

    fn rotate_x(self, angle: Rad<FloatType>) -> Self::Target {
        self.transform(Matrix4::from_angle_x(angle))
    }

    fn rotate_y(self, angle: Rad<FloatType>) -> Self::Target {
        self.transform(Matrix4::from_angle_y(angle))
    }

    fn rotate_z(self, angle: Rad<FloatType>) -> Self::Target {
        self.transform(Matrix4::from_angle_z(angle))
    }

    fn nonuniform_scale(
        self,
        x_scale: FloatType,
        y_scale: FloatType,
        z_scale: FloatType,
    ) -> Self::Target {
        self.transform(Matrix4::from_nonuniform_scale(x_scale, y_scale, z_scale))
    }

    fn scale(self, scale: FloatType) -> Self::Target {
        self.transform(Matrix4::from_scale(scale))
    }

    fn identity(self) -> Self::Target {
        self.transform(Matrix4::identity())
    }
}

pub trait DefaultTransformable: Intersectable {}

pub struct Transformed<P: Intersectable> {
    primitive: P,
    transform: StaticTransform,
}

impl<P: Intersectable + Clone> Clone for Transformed<P> {
    fn clone(&self) -> Self {
        Self {
            primitive: self.primitive.clone(),
            transform: self.transform.clone(),
        }
    }
}

impl<P: Intersectable> Intersectable for Transformed<P> {
    type Result = P::Result;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result> {
        let instant = self.transform.transform_at_t(ray.time);
        let transformed_ray = instant.transform_ray(ray);

        self.primitive
            .intersect(&transformed_ray, t_min, t_max)
            .map(|hit_result| instant.transform_hit_result(ray, hit_result))
    }
}

impl<P: Intersectable + TimeDependentBounded> TimeDependentBounded for Transformed<P> {
    fn time_dependent_bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let primitive_box = self.primitive.time_dependent_bounding_box(t0, t1);

        let box0 = self
            .transform
            .transform_at_t(t0)
            .transform_bounding_box(primitive_box);
        let box1 = self
            .transform
            .transform_at_t(t1)
            .transform_bounding_box(primitive_box);

        BoundingBox::surrounding_box(&box0, &box1)
    }
}

impl<P: Intersectable + Skinnable> Transformable for Transformed<P> {
    type Target = Self;

    fn transform(self, transform: Matrix4) -> Self::Target {
        Self::Target {
            transform: self.transform.concat(transform),
            primitive: self.primitive,
        }
    }
}

impl<P: Intersectable + Skinnable> Skinnable for Transformed<P> {
    type Target = Transformed<P::Target>;

    fn apply_material<M: 'static + crate::BaseMaterial>(self, material: M) -> Self::Target {
        Transformed {
            primitive: self.primitive.apply_material(material),
            transform: self.transform,
        }
    }
}

impl<P: 'static + Intersectable<Result = GeometryHitResult> + TimeDependentBounded + Primitive>
    Primitive for Transformed<P>
{
    fn to_dyn_primitive(self) -> DynPrimitive {
        DynPrimitive::new(self)
    }

    fn decompose_box(self: Box<Self>) -> CompoundPrimitive {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundPrimitive {
        let compound = self.primitive.decompose();
        let transform = self.transform;

        compound
            .into_iter()
            .map(|primitive| Transformed {
                primitive,
                transform: transform.clone(),
            })
            .collect()
    }
}

impl<P: 'static + Intersectable<Result = SkinnedHitResult> + TimeDependentBounded + Visible> Visible
    for Transformed<P>
{
    fn to_dyn_visible(self) -> DynVisible {
        DynVisible::new(self)
    }

    fn decompose_box(self: Box<Self>) -> CompoundVisible {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundVisible {
        let compound = self.primitive.decompose();
        let transform = self.transform;

        compound
            .into_iter()
            .map(|primitive| Transformed {
                primitive,
                transform: transform.clone(),
            })
            .collect()
    }
}

pub struct TransformableIterator<P: Intersectable, I: Iterator<Item = P>> {
    iter: I,
    transform: StaticTransform,
}

impl<P: Intersectable, I: Iterator<Item = P>> Iterator for TransformableIterator<P, I> {
    type Item = Transformed<P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|primitive| Transformed {
            primitive,
            transform: self.transform.clone(),
        })
    }
}

impl<P: Intersectable, I: IntoIterator<Item = P> + Intersectable> IntoIterator for Transformed<I> {
    type Item = Transformed<P>;
    type IntoIter = TransformableIterator<P, I::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        TransformableIterator {
            iter: self.primitive.into_iter(),
            transform: self.transform,
        }
    }
}

impl<DT: DefaultTransformable + Skinnable> Transformable for DT {
    type Target = Transformed<DT>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        Self::Target {
            primitive: self,
            transform: StaticTransform::new(transform),
        }
    }
}
