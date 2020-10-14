use crate::math::*;
use super::{Shape, UntransformedShape, HitResult, CompoundShape};
use crate::ray_scanner::Ray;
use crate::BoundingBox;
use crate::RenderStatsCollector;

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

        self.shape.intersect(&transformed_ray, t_min, t_max, stats).map(|mut hit_result| {
            hit_result.hit_point = self.transform.transform_point(hit_result.hit_point);
            hit_result.surface_normal = self.transform.transform_vector(hit_result.surface_normal).normalize();
            hit_result.tangent = self.transform.transform_vector(hit_result.tangent).normalize();
            hit_result.bitangent = self.transform.transform_vector(hit_result.bitangent).normalize();
            hit_result.distance = (hit_result.hit_point - ray.origin.into_point()).magnitude();
    
            hit_result
        })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        let shape_bounding_box = self.shape.bounding_box(t0, t1);
        BoundingBox::new(
            self.transform.transform_point(shape_bounding_box.min_point().into_point()),
            self.transform.transform_point(shape_bounding_box.max_point().into_point()),
        )
    }
}

/*pub struct TransformedShapeIterator<Iter: Iterator<Item = Box<dyn Shape>>> {
    transform: Matrix4,
    iter: Iter,
}

impl<Iter: Iterator<Item = Box<dyn Shape>>> Iterator for TransformedShapeIterator<Iter> {
    type Item = Box<dyn Shape>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|child| {
            let a: u8 = child;
            let shape = TransformedShape {
                transform: self.transform.clone(),
                shape: child,
            };
            todo!()
        })
    }
}

impl<S: CompoundShape> CompoundShape for TransformedShape<S> {
    type GeometryIterator = TransformedShapeIterator<S::GeometryIterator>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        TransformedShapeIterator {
            transform: self.transform,
            iter: self.shape.into_geometry_iterator(),
        }
    }
}*/

impl<S: 'static + Shape> CompoundShape for TransformedShape<S> {
    type GeometryIterator = std::iter::Once<Box<dyn Shape>>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        let b: Box<dyn Shape> = Box::new(self);
        std::iter::once(b)
    }
}

pub trait Transformable: Shape + Sized {
    type Target: Shape;

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

    fn nonuniform_scale(self, x_scale: FloatType, y_scale: FloatType, z_scale: FloatType) -> Self::Target {
        self.transform(Matrix4::from_nonuniform_scale(x_scale, y_scale, z_scale))
    }
}

impl<S: UntransformedShape + Sized> Transformable for S {
    type Target = TransformedShape<S>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        let inverse = transform.inverse_transform().unwrap();
        Self::Target { transform, inverse, shape: self }
    }
}

impl<S: Shape> Transformable for TransformedShape<S> {
    type Target = Self;

    fn transform(self, transform: Matrix4) -> Self::Target {
        let transform = transform.concat(&self.transform);
        let inverse = transform.inverse_transform().unwrap();
        Self::Target { transform, inverse, shape: self.shape }
    }
}