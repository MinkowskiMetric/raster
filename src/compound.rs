use crate::{
    math::*, DefaultSkinnable, DefaultTransformable, GeometryHitResult, IntersectResultIteratorOps,
    Intersectable, Ray, SkinnedHitResult, TimeDependentBounded,
};
use core::iter::FromIterator;
use std::sync::Arc;

#[macro_export]
macro_rules! compound_visible [
    () => {
        $crate::CompoundVisible::default()
    };

    ($($x:expr),+ $(,)?) => { $crate::CompoundVisible::from(vec![$($crate::macro_helpers::convert_to_vec_for_macro($x)),+]) };
];

#[macro_export]
macro_rules! compound_primitive [
    () => {
        $crate::CompoundPrimitive::default()
    };

    ($($x:expr),+ $(,)?) => { $crate::CompoundPrimitive::from(vec![$($x.to_dyn_primitive()),+]) };
];

// Start off by defining dynamic types for primitives and visible objects
pub trait Primitive: Intersectable<Result = GeometryHitResult> + TimeDependentBounded {
    fn to_dyn_primitive(self) -> DynPrimitive;
    fn decompose_box(self: Box<Self>) -> CompoundPrimitive;
    fn decompose(self) -> CompoundPrimitive;
}

impl<
        P: 'static
            + Intersectable<Result = GeometryHitResult>
            + TimeDependentBounded
            + DefaultPrimitive,
    > Primitive for P
{
    fn to_dyn_primitive(self) -> DynPrimitive {
        DynPrimitive::new(self)
    }

    fn decompose_box(self: Box<Self>) -> CompoundPrimitive {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundPrimitive {
        compound_primitive![self]
    }
}

pub trait DefaultPrimitive:
    Intersectable<Result = GeometryHitResult> + TimeDependentBounded
{
}

pub struct DynPrimitive(Box<dyn Primitive>);

impl DynPrimitive {
    pub(crate) fn new<P: 'static + Primitive>(p: P) -> Self {
        Self(Box::new(p))
    }
}

impl Intersectable for DynPrimitive {
    type Result = GeometryHitResult;

    fn intersect(
        &self,
        ray: &crate::Ray,
        t_min: crate::math::FloatType,
        t_max: crate::math::FloatType,
    ) -> Option<Self::Result> {
        self.0.as_ref().intersect(ray, t_min, t_max)
    }
}

impl TimeDependentBounded for DynPrimitive {
    fn time_dependent_bounding_box(
        &self,
        t0: crate::math::FloatType,
        t1: crate::math::FloatType,
    ) -> crate::BoundingBox {
        self.0.as_ref().time_dependent_bounding_box(t0, t1)
    }
}

impl Primitive for DynPrimitive {
    fn to_dyn_primitive(self) -> DynPrimitive {
        self
    }

    fn decompose_box(self: Box<Self>) -> CompoundPrimitive {
        self.0.decompose_box()
    }

    fn decompose(self) -> CompoundPrimitive {
        self.0.decompose_box()
    }
}

impl DefaultTransformable for DynPrimitive {}

#[derive(Clone)]
pub struct SharedPrimitive(Arc<dyn Primitive>);

impl SharedPrimitive {
    pub(crate) fn new<P: 'static + Primitive>(p: P) -> Self {
        Self(Arc::new(p))
    }
}

impl Intersectable for SharedPrimitive {
    type Result = GeometryHitResult;

    fn intersect(
        &self,
        ray: &crate::Ray,
        t_min: crate::math::FloatType,
        t_max: crate::math::FloatType,
    ) -> Option<Self::Result> {
        self.0.intersect(ray, t_min, t_max)
    }
}

impl TimeDependentBounded for SharedPrimitive {
    fn time_dependent_bounding_box(
        &self,
        t0: crate::math::FloatType,
        t1: crate::math::FloatType,
    ) -> crate::BoundingBox {
        self.0.time_dependent_bounding_box(t0, t1)
    }
}

impl Primitive for SharedPrimitive {
    fn to_dyn_primitive(self) -> DynPrimitive {
        DynPrimitive::new(self)
    }

    fn decompose_box(self: Box<Self>) -> CompoundPrimitive {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundPrimitive {
        compound_primitive![self.to_dyn_primitive()]
    }
}

impl DefaultTransformable for SharedPrimitive {}
impl DefaultSkinnable for SharedPrimitive {}

pub trait Visible: Intersectable<Result = SkinnedHitResult> + TimeDependentBounded {
    fn to_dyn_visible(self) -> DynVisible;
    fn decompose_box(self: Box<Self>) -> CompoundVisible;
    fn decompose(self) -> CompoundVisible;
}

pub trait DefaultVisible: Intersectable<Result = SkinnedHitResult> + TimeDependentBounded {}

pub struct DynVisible(Box<dyn Visible>);

impl DynVisible {
    pub(crate) fn new<P: 'static + Visible>(p: P) -> Self {
        Self(Box::new(p))
    }
}

impl<
        P: 'static + Intersectable<Result = SkinnedHitResult> + TimeDependentBounded + DefaultVisible,
    > Visible for P
{
    fn to_dyn_visible(self) -> DynVisible {
        DynVisible::new(self)
    }

    fn decompose_box(self: Box<Self>) -> CompoundVisible {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundVisible {
        compound_visible![self]
    }
}

impl Intersectable for DynVisible {
    type Result = SkinnedHitResult;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result> {
        self.0.as_ref().intersect(ray, t_min, t_max)
    }
}

impl TimeDependentBounded for DynVisible {
    fn time_dependent_bounding_box(&self, t0: FloatType, t1: FloatType) -> crate::BoundingBox {
        self.0.as_ref().time_dependent_bounding_box(t0, t1)
    }
}

impl DefaultTransformable for DynVisible {}
impl DefaultSkinnable for DynVisible {}

impl Visible for DynVisible {
    fn to_dyn_visible(self) -> DynVisible {
        self
    }

    fn decompose_box(self: Box<Self>) -> CompoundVisible {
        self.0.decompose_box()
    }

    fn decompose(self) -> CompoundVisible {
        self.0.decompose_box()
    }
}

// Now we need a compound of each of those
#[derive(Default)]
pub struct CompoundPrimitive(Vec<DynPrimitive>);

impl From<Vec<DynPrimitive>> for CompoundPrimitive {
    fn from(v: Vec<DynPrimitive>) -> Self {
        Self(v)
    }
}

impl CompoundPrimitive {
    pub fn iter(&self) -> std::slice::Iter<'_, DynPrimitive> {
        self.0.iter()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn into_inner(self) -> Vec<DynPrimitive> {
        self.0
    }
}

impl Intersectable for CompoundPrimitive {
    type Result = GeometryHitResult;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result> {
        self.iter()
            .filter_map(|i| i.intersect(ray, t_min, t_max))
            .nearest()
    }
}

impl TimeDependentBounded for CompoundPrimitive {
    fn time_dependent_bounding_box(&self, t0: FloatType, t1: FloatType) -> crate::BoundingBox {
        self.iter()
            .map(|b| b.time_dependent_bounding_box(t0, t1))
            .collect()
    }
}

// Question: Is the default implementation of transformable what we want here?
impl DefaultTransformable for CompoundPrimitive {}
impl DefaultSkinnable for CompoundPrimitive {}

impl Primitive for CompoundPrimitive {
    fn to_dyn_primitive(self) -> DynPrimitive {
        DynPrimitive(Box::new(self))
    }

    fn decompose_box(self: Box<Self>) -> CompoundPrimitive {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundPrimitive {
        self.0.into_iter().flat_map(|f| f.decompose()).collect()
    }
}

impl IntoIterator for CompoundPrimitive {
    type Item = DynPrimitive;
    type IntoIter = std::vec::IntoIter<DynPrimitive>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<P: Primitive> Extend<P> for CompoundPrimitive {
    fn extend<T: IntoIterator<Item = P>>(&mut self, iter: T) {
        self.0
            .extend(iter.into_iter().map(Primitive::to_dyn_primitive))
    }
}

impl<P: Primitive> FromIterator<P> for CompoundPrimitive {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().map(Primitive::to_dyn_primitive).collect())
    }
}

#[derive(Default)]
pub struct CompoundVisible(Vec<DynVisible>);

impl From<Vec<DynVisible>> for CompoundVisible {
    fn from(v: Vec<DynVisible>) -> Self {
        Self(v)
    }
}

impl CompoundVisible {
    pub fn push<P: Visible>(&mut self, p: P) {
        self.0.push(p.to_dyn_visible())
    }

    pub fn iter(&self) -> std::slice::Iter<'_, DynVisible> {
        self.0.iter()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn into_inner(self) -> Vec<DynVisible> {
        self.0
    }
}

impl Intersectable for CompoundVisible {
    type Result = SkinnedHitResult;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result> {
        self.iter()
            .filter_map(|i| i.intersect(ray, t_min, t_max))
            .nearest()
    }
}

impl TimeDependentBounded for CompoundVisible {
    fn time_dependent_bounding_box(&self, t0: FloatType, t1: FloatType) -> crate::BoundingBox {
        self.iter()
            .map(|b| b.time_dependent_bounding_box(t0, t1))
            .collect()
    }
}

// Question: Is the default implementation of transformable what we want here?
impl DefaultTransformable for CompoundVisible {}
impl DefaultSkinnable for CompoundVisible {}

impl Visible for CompoundVisible {
    fn to_dyn_visible(self) -> DynVisible {
        DynVisible::new(self)
    }

    fn decompose_box(self: Box<Self>) -> CompoundVisible {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundVisible {
        self.0.into_iter().flat_map(|f| f.decompose()).collect()
    }
}

impl IntoIterator for CompoundVisible {
    type Item = DynVisible;
    type IntoIter = std::vec::IntoIter<DynVisible>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<P: Visible> Extend<P> for CompoundVisible {
    fn extend<T: IntoIterator<Item = P>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(Visible::to_dyn_visible))
    }
}

impl<P: Visible> FromIterator<P> for CompoundVisible {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().map(Visible::to_dyn_visible).collect())
    }
}

pub mod macro_helpers {

    use crate::{DynVisible, Visible};

    #[doc(hidden)]
    pub fn convert_to_vec_for_macro<V: Visible>(v: V) -> DynVisible {
        v.to_dyn_visible()
    }
}
