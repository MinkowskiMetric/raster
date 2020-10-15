use super::{
    CompoundShape, HitResult, Primitive, PrimitiveHitResult, Shape, SkinnablePrimitive,
    SkinnedPrimitive, UntransformedPrimitive, UntransformedShape,
};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::Material;
use crate::RenderStatsCollector;

fn transform_bounding_box(bounding_box: BoundingBox, transform: &Matrix4) -> BoundingBox {
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
        let corner = transform.transform_point(*corner);

        pt_min.x = pt_min.x.min(corner.x);
        pt_min.y = pt_min.y.min(corner.y);
        pt_min.z = pt_min.z.min(corner.z);

        pt_max.x = pt_max.x.max(corner.x);
        pt_max.y = pt_max.y.max(corner.y);
        pt_max.z = pt_max.z.max(corner.z);
    }

    BoundingBox::new(pt_min, pt_max)
}

#[derive(Clone, Debug)]
pub struct TransformedPrimitive<P: Primitive> {
    transform: Matrix4,
    inverse: Matrix4,
    primitive: P,
}

impl<P: Primitive> Primitive for TransformedPrimitive<P> {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
        let transformed_origin = self.inverse.transform_point(ray.origin.into_point());
        let transformed_direction = self.inverse.transform_vector(ray.direction.into_vector());
        let transformed_ray = Ray::new(transformed_origin, transformed_direction, ray.time);

        self.primitive
            .intersect(&transformed_ray, t_min, t_max, stats)
            .map(|mut hit_result| {
                hit_result.hit_point = self.transform.transform_point(hit_result.hit_point);
                hit_result.surface_normal = self
                    .transform
                    .transform_vector(hit_result.surface_normal)
                    .normalize();
                hit_result.tangent = self
                    .transform
                    .transform_vector(hit_result.tangent)
                    .normalize();
                hit_result.bitangent = self
                    .transform
                    .transform_vector(hit_result.bitangent)
                    .normalize();
                hit_result.distance = (hit_result.hit_point - ray.origin.into_point()).magnitude();

                hit_result
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        transform_bounding_box(self.primitive.bounding_box(t0, t1), &self.transform)
    }
}

impl<M: Material, P: Primitive> SkinnablePrimitive<M> for TransformedPrimitive<P> {
    type Target = SkinnedPrimitive<TransformedPrimitive<P>, M>;

    fn apply_material(self, material: M) -> Self::Target {
        SkinnedPrimitive::new(material, vec![self])
    }
}

#[derive(Clone, Debug)]
pub struct TransformedShape<S: Shape> {
    transform: Matrix4,
    inverse: Matrix4,
    shape: S,
}

impl<S: Shape> Shape for TransformedShape<S> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        let transformed_origin = self.inverse.transform_point(ray.origin.into_point());
        let transformed_direction = self.inverse.transform_vector(ray.direction.into_vector());
        let transformed_ray = Ray::new(transformed_origin, transformed_direction, ray.time);

        self.shape
            .intersect(&transformed_ray, t_min, t_max, stats)
            .map(|mut hit_result| {
                hit_result.hit_point = self.transform.transform_point(hit_result.hit_point);
                hit_result.surface_normal = self
                    .transform
                    .transform_vector(hit_result.surface_normal)
                    .normalize();
                hit_result.tangent = self
                    .transform
                    .transform_vector(hit_result.tangent)
                    .normalize();
                hit_result.bitangent = self
                    .transform
                    .transform_vector(hit_result.bitangent)
                    .normalize();
                hit_result.distance = (hit_result.hit_point - ray.origin.into_point()).magnitude();

                hit_result
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        transform_bounding_box(self.shape.bounding_box(t0, t1), &self.transform)
    }
}

pub struct TransformedShapeIterator<Iter: Iterator<Item = Box<dyn Shape>>> {
    transform: Matrix4,
    inverse: Matrix4,
    iter: Iter,
}

impl<Iter: Iterator<Item = Box<dyn Shape>>> Iterator for TransformedShapeIterator<Iter> {
    type Item = Box<dyn Shape>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|child| {
            let b: Box<dyn Shape> = Box::new(TransformedShape {
                transform: self.transform.clone(),
                inverse: self.inverse.clone(),
                shape: child,
            });
            b
        })
    }
}

impl<S: CompoundShape> CompoundShape for TransformedShape<S> {
    type GeometryIterator = TransformedShapeIterator<S::GeometryIterator>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        TransformedShapeIterator {
            transform: self.transform,
            inverse: self.inverse,
            iter: self.shape.into_geometry_iterator(),
        }
    }
}

macro_rules! defined_transforms {
    () => {
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
    };
}

pub trait TransformableShape: Shape + Sized {
    type Target: Shape;

    fn transform(self, transform: Matrix4) -> Self::Target;

    defined_transforms!();
}

pub trait TransformablePrimitive: Primitive + Sized {
    type Target: Primitive;

    fn transform(self, transform: Matrix4) -> Self::Target;

    defined_transforms!();
}

impl<S: UntransformedShape + Sized> TransformableShape for S {
    type Target = TransformedShape<S>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        let inverse = transform.inverse_transform().unwrap();
        Self::Target {
            transform,
            inverse,
            shape: self,
        }
    }
}

impl<P: UntransformedPrimitive + Sized> TransformablePrimitive for P {
    type Target = TransformedPrimitive<P>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        let inverse = transform.inverse_transform().unwrap();
        Self::Target {
            transform,
            inverse,
            primitive: self,
        }
    }
}

impl<P: Primitive> TransformablePrimitive for TransformedPrimitive<P> {
    type Target = Self;

    fn transform(self, transform: Matrix4) -> Self::Target {
        let transform = transform.concat(&self.transform);
        let inverse = transform.inverse_transform().unwrap();
        Self::Target {
            transform,
            inverse,
            primitive: self.primitive,
        }
    }
}

impl<S: Shape> TransformableShape for TransformedShape<S> {
    type Target = Self;

    fn transform(self, transform: Matrix4) -> Self::Target {
        let transform = transform.concat(&self.transform);
        let inverse = transform.inverse_transform().unwrap();
        Self::Target {
            transform,
            inverse,
            shape: self.shape,
        }
    }
}
