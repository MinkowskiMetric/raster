use crate::{
    BaseMaterial, CompoundVisible, DynVisible, IntersectResult, Intersectable, Primitive,
    SkinnedHitResult, TimeDependentBounded, Transformable, Visible,
};
use std::sync::Arc;

pub trait Skinnable: Intersectable {
    type Target: Intersectable<Result = SkinnedHitResult>;

    fn apply_material<M: 'static + BaseMaterial>(self, material: M) -> Self::Target;
}

pub trait DefaultSkinnable {}

impl<S: Intersectable + DefaultSkinnable> Skinnable for S {
    type Target = Skinned<S>;

    fn apply_material<M: 'static + BaseMaterial>(self, material: M) -> Self::Target {
        Skinned::new(self, material)
    }
}

pub struct Skinned<P: Intersectable> {
    primitive: P,
    material: Arc<dyn BaseMaterial>,
}

impl<P: Intersectable + Clone> Clone for Skinned<P> {
    fn clone(&self) -> Self {
        Self {
            primitive: self.primitive.clone(),
            material: self.material.clone(),
        }
    }
}

impl<P: Intersectable> Skinned<P> {
    pub fn new<M: 'static + BaseMaterial>(primitive: P, material: M) -> Self {
        Self {
            primitive,
            material: Arc::new(material),
        }
    }
}

impl<P: Intersectable> Intersectable for Skinned<P> {
    type Result = SkinnedHitResult;

    fn intersect(
        &self,
        ray: &crate::Ray,
        t_min: crate::math::FloatType,
        t_max: crate::math::FloatType,
    ) -> Option<Self::Result> {
        self.primitive
            .intersect(ray, t_min, t_max)
            .map(|hit_result| {
                SkinnedHitResult::new(hit_result.as_geometry_hit_result(), self.material.clone())
            })
    }
}

impl<P: Intersectable + TimeDependentBounded> TimeDependentBounded for Skinned<P> {
    fn time_dependent_bounding_box(
        &self,
        t0: crate::math::FloatType,
        t1: crate::math::FloatType,
    ) -> crate::BoundingBox {
        self.primitive.time_dependent_bounding_box(t0, t1)
    }
}

impl<P: Intersectable + Transformable> Transformable for Skinned<P> {
    type Target = Skinned<P::Target>;

    fn transform(self, transform: crate::math::Matrix4) -> Self::Target {
        Skinned {
            primitive: self.primitive.transform(transform),
            material: self.material,
        }
    }
}

impl<P: Intersectable> Skinnable for Skinned<P> {
    type Target = Self;

    fn apply_material<M: 'static + BaseMaterial>(self, material: M) -> Self::Target {
        Self {
            primitive: self.primitive,
            material: Arc::new(material),
        }
    }
}

impl<P: 'static + Intersectable + TimeDependentBounded + Primitive> Visible for Skinned<P> {
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
