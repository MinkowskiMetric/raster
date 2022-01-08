use std::sync::Arc;

use crate::{
    math::*, BaseMaterial, BoundingBox, CompoundPrimitive, CompoundVisible, DynPrimitive,
    DynVisible, GeometryHitResult, IntersectResult, Intersectable, Primitive, Ray, Skinnable,
    SkinnedHitResult, TimeDependentBounded, Visible,
};

trait GeometryTransform {
    fn transform_at_t(&self, t: FloatType) -> StaticTransform;
}

#[derive(Clone, Debug)]
struct StaticTransform {
    transform: Matrix4,
    inverse: Matrix4,
}

impl StaticTransform {
    pub fn concat(self, transform: &Matrix4) -> Self {
        let transform = transform.concat(&self.transform);
        let inverse = transform.inverse_transform().unwrap();
        Self { transform, inverse }
    }
}

impl GeometryTransform for StaticTransform {
    fn transform_at_t(&self, _t: FloatType) -> StaticTransform {
        self.clone()
    }
}

pub trait Transformable: Sized {
    type Target: Transformable;

    fn core_transform(self, transform: &Matrix4, inverse_transform: &Matrix4) -> Self::Target;

    fn transform(self, transform: Matrix4) -> Self::Target {
        let inverse_transform = transform.inverse_transform().unwrap();
        self.core_transform(&transform, &inverse_transform)
    }

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

pub trait DefaultTransformable {}

pub struct Transformed<P> {
    primitive: P,
    transform: StaticTransform,
}

impl<P: Clone> Clone for Transformed<P> {
    fn clone(&self) -> Self {
        Self {
            primitive: self.primitive.clone(),
            transform: self.transform.clone(),
        }
    }
}

impl<R: IntersectResult + Transformable, P: Intersectable<Result = R>> Intersectable
    for Transformed<P>
where
    <R as Transformable>::Target: IntersectResult,
{
    type Result = <R as Transformable>::Target;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result> {
        let instant = self.transform.transform_at_t(ray.time());
        let transformed_ray = ray.core_transform(&instant.transform, &instant.inverse);

        self.primitive
            .intersect(&transformed_ray, t_min, t_max)
            .map(|hit_result| hit_result.core_transform(&instant.transform, &instant.inverse))
    }
}

impl<P: TimeDependentBounded> TimeDependentBounded for Transformed<P> {
    fn time_dependent_bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let primitive_box = self.primitive.time_dependent_bounding_box(t0, t1);

        [t0, t1]
            .iter()
            .flat_map(|t| {
                primitive_box
                    .transform(self.transform.transform_at_t(*t).transform)
                    .all_corners()
            })
            .collect()
    }
}

impl<P> Transformable for Transformed<P> {
    type Target = Self;

    fn core_transform(self, transform: &Matrix4, _inverse_transform: &Matrix4) -> Self::Target {
        Self::Target {
            transform: self.transform.concat(transform),
            primitive: self.primitive,
        }
    }
}

impl<P: Skinnable> Skinnable for Transformed<P> {
    type Target = Transformed<P::Target>;

    fn apply_shared_material(self, material: Arc<dyn BaseMaterial>) -> Self::Target {
        Transformed {
            primitive: self.primitive.apply_shared_material(material),
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

pub struct TransformableIterator<P, I: Iterator<Item = P>> {
    iter: I,
    transform: StaticTransform,
}

impl<P, I: Iterator<Item = P>> Iterator for TransformableIterator<P, I> {
    type Item = Transformed<P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|primitive| Transformed {
            primitive,
            transform: self.transform.clone(),
        })
    }
}

impl<P, I: IntoIterator<Item = P>> IntoIterator for Transformed<I> {
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

    fn core_transform(self, transform: &Matrix4, inverse_transform: &Matrix4) -> Self::Target {
        Self::Target {
            primitive: self,
            transform: StaticTransform {
                transform: *transform,
                inverse: *inverse_transform,
            },
        }
    }
}
