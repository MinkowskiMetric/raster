use super::{
    CompoundShape, HitResult, Primitive, PrimitiveHitResult, Shape, SkinnablePrimitive,
    SkinnedPrimitive, UntransformedPrimitive,
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

pub trait GeometryTransform: Send + Sync + std::fmt::Debug {
    fn transform_ray(&self, ray: &Ray) -> Ray;
    fn transform_hit_result(
        &self,
        original_ray: &Ray,
        hit_result: PrimitiveHitResult,
    ) -> PrimitiveHitResult;
    fn transform_bounding_box(
        &self,
        t0: FloatType,
        t1: FloatType,
        bounding_box: BoundingBox,
    ) -> BoundingBox;
}

#[derive(Clone, Debug)]
pub struct StaticTransform {
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

impl GeometryTransform for StaticTransform {
    fn transform_ray(&self, ray: &Ray) -> Ray {
        let transformed_origin = self.inverse.transform_point(ray.origin.into_point());
        let transformed_direction = self.inverse.transform_vector(ray.direction.into_vector());
        let transformed_ray = Ray::new(transformed_origin, transformed_direction, ray.time);
        transformed_ray
    }

    fn transform_hit_result(
        &self,
        original_ray: &Ray,
        mut hit_result: PrimitiveHitResult,
    ) -> PrimitiveHitResult {
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
        hit_result
            .set_distance((hit_result.hit_point() - original_ray.origin.into_point()).magnitude());

        hit_result
    }

    fn transform_bounding_box(
        &self,
        _t0: FloatType,
        _t1: FloatType,
        bounding_box: BoundingBox,
    ) -> BoundingBox {
        transform_bounding_box(bounding_box, &self.transform)
    }
}

#[derive(Clone, Debug)]
pub struct TransformedPrimitive<P: Primitive, T: GeometryTransform> {
    primitive: P,
    transform: T,
}

impl<P: Primitive, T: GeometryTransform> Primitive for TransformedPrimitive<P, T> {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
        let transformed_ray = self.transform.transform_ray(ray);

        self.primitive
            .intersect(&transformed_ray, t_min, t_max, stats)
            .map(|hit_result| self.transform.transform_hit_result(ray, hit_result))
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.transform
            .transform_bounding_box(t0, t1, self.primitive.bounding_box(t0, t1))
    }
}

impl<M: Material, P: Primitive, T: GeometryTransform> SkinnablePrimitive<M>
    for TransformedPrimitive<P, T>
{
    type Target = SkinnedPrimitive<TransformedPrimitive<P, T>, M>;

    fn apply_material(self, material: M) -> Self::Target {
        SkinnedPrimitive::new(material, vec![self])
    }
}

#[derive(Clone, Debug)]
pub struct TransformedShape<S: Shape, T: GeometryTransform> {
    transform: T,
    shape: S,
}

impl<S: Shape, T: GeometryTransform> Shape for TransformedShape<S, T> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        let transformed_ray = self.transform.transform_ray(ray);

        self.shape
            .intersect(&transformed_ray, t_min, t_max, stats)
            .map(|hit_result| {
                let (hit_result, material) = hit_result.split();
                let hit_result = self.transform.transform_hit_result(ray, hit_result);
                HitResult::new(hit_result, material)
            })
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.transform
            .transform_bounding_box(t0, t1, self.shape.bounding_box(t0, t1))
    }
}

pub struct TransformedShapeIterator<
    T: GeometryTransform + Clone,
    Iter: Iterator<Item = Box<dyn Shape>>,
> {
    transform: T,
    iter: Iter,
}

impl<T: 'static + GeometryTransform + Clone, Iter: Iterator<Item = Box<dyn Shape>>> Iterator
    for TransformedShapeIterator<T, Iter>
{
    type Item = Box<dyn Shape>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|child| {
            let b: Box<dyn Shape> = Box::new(TransformedShape {
                transform: self.transform.clone(),
                shape: child,
            });
            b
        })
    }
}

impl<S: CompoundShape, T: 'static + GeometryTransform + Clone> CompoundShape
    for TransformedShape<S, T>
{
    type GeometryIterator = TransformedShapeIterator<T, S::GeometryIterator>;

    fn into_geometry_iterator(self) -> Self::GeometryIterator {
        TransformedShapeIterator {
            transform: self.transform,
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

impl TransformableShape for Box<dyn Shape> {
    // This is pretty unfortunate. What is happening here is that we're transforming the shape after
    // we've discarded it's type, so we are probably double transforming.
    type Target = TransformedShape<Self, StaticTransform>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        Self::Target {
            transform: StaticTransform::new(transform),
            shape: self,
        }
    }
}

impl<P: UntransformedPrimitive + Sized> TransformablePrimitive for P {
    type Target = TransformedPrimitive<P, StaticTransform>;

    fn transform(self, transform: Matrix4) -> Self::Target {
        Self::Target {
            transform: StaticTransform::new(transform),
            primitive: self,
        }
    }
}

impl<P: Primitive> TransformablePrimitive for TransformedPrimitive<P, StaticTransform> {
    type Target = Self;

    fn transform(self, transform: Matrix4) -> Self::Target {
        Self::Target {
            transform: self.transform.concat(transform),
            primitive: self.primitive,
        }
    }
}

impl<S: Shape> TransformableShape for TransformedShape<S, StaticTransform> {
    type Target = Self;

    fn transform(self, transform: Matrix4) -> Self::Target {
        Self::Target {
            transform: self.transform.concat(transform),
            shape: self.shape,
        }
    }
}
