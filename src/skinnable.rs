use crate::{
    math::*, BaseMaterial, CompoundVisible, DynVisible, GeometryHitResult, IntersectResult,
    Intersectable, Primitive, TimeDependentBounded, Transformable, Visible, WrappedIntersectResult,
};
use std::sync::Arc;

pub trait Skinnable: Sized {
    type Target;

    fn apply_material<M: 'static + BaseMaterial + Send + Sync>(self, material: M) -> Self::Target {
        self.apply_shared_material(Arc::new(material))
    }

    fn apply_shared_material(self, material: Arc<(dyn BaseMaterial + Send + Sync)>) -> Self::Target;
}

pub trait DefaultSkinnable {}

impl<S: DefaultSkinnable> Skinnable for S {
    type Target = Skinned<S>;

    fn apply_shared_material(self, material: Arc<(dyn BaseMaterial + Send + Sync)>) -> Self::Target {
        Skinned::new(self, material)
    }
}

pub struct Skinned<P> {
    primitive: P,
    material: Arc<(dyn BaseMaterial + Send + Sync)>,
}

impl<P: Clone> Clone for Skinned<P> {
    fn clone(&self) -> Self {
        Self {
            primitive: self.primitive.clone(),
            material: self.material.clone(),
        }
    }
}

impl<P> Skinned<P> {
    pub fn new(primitive: P, material: Arc<(dyn BaseMaterial + Send + Sync)>) -> Self {
        Self {
            primitive,
            material,
        }
    }

    pub fn split(self) -> (P, Arc<dyn BaseMaterial>) {
        (self.primitive, self.material)
    }
}

impl<P: Intersectable<Result = GeometryHitResult>> Intersectable for Skinned<P> {
    type Result = <GeometryHitResult as Skinnable>::Target;

    fn intersect(
        &self,
        ray: &crate::Ray,
        t_min: crate::math::FloatType,
        t_max: crate::math::FloatType,
    ) -> Option<Self::Result> {
        self.primitive
            .intersect(ray, t_min, t_max)
            .map(|hit_result| hit_result.apply_shared_material(self.material.clone()))
    }
}

impl<P: TimeDependentBounded> TimeDependentBounded for Skinned<P> {
    fn time_dependent_bounding_box(
        &self,
        t0: crate::math::FloatType,
        t1: crate::math::FloatType,
    ) -> crate::BoundingBox {
        self.primitive.time_dependent_bounding_box(t0, t1)
    }
}

impl<P: Transformable> Transformable for Skinned<P> {
    type Target = Skinned<<P as Transformable>::Target>;

    fn core_transform(self, transform: &Matrix4, inverse_transform: &Matrix4) -> Self::Target {
        Skinned {
            primitive: self.primitive.core_transform(transform, inverse_transform),
            material: self.material,
        }
    }
}

impl<P> Skinnable for Skinned<P> {
    type Target = Self;

    fn apply_shared_material(self, material: Arc<(dyn BaseMaterial + Send + Sync)>) -> Self::Target {
        Self {
            primitive: self.primitive,
            material,
        }
    }
}

impl<P: IntersectResult> WrappedIntersectResult for Skinned<P> {
    type Wrapped = P;

    fn intersect_result(&self) -> &Self::Wrapped {
        &self.primitive
    }
}

impl<P: 'static + Intersectable + TimeDependentBounded + Primitive + Send + Sync> Visible for Skinned<P> {
    fn to_dyn_visible(self) -> DynVisible {
        DynVisible::new(self)
    }

    fn decompose_box(self: Box<Self>) -> CompoundVisible {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundVisible {
        let compound = self.primitive.decompose();
        let material = self.material;

        compound
            .into_iter()
            .map(|primitive| Skinned {
                primitive,
                material: material.clone(),
            })
            .collect()
    }
}
