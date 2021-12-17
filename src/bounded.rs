use crate::{math::*, BoundingBox};

pub trait TimeDependentBounded {
    fn time_dependent_bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

pub trait Bounded {
    fn bounding_box(&self) -> BoundingBox;
}

impl<B: Bounded> TimeDependentBounded for B {
    fn time_dependent_bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        self.bounding_box()
    }
}

pub trait TimeDependentBoundedIteratorOps {
    fn time_dependent_bounding_box(self, t0: FloatType, t1: FloatType) -> BoundingBox;
}

impl<'a, B: 'a + TimeDependentBounded, Iter: Iterator<Item = &'a B>> TimeDependentBoundedIteratorOps
    for Iter
{
    fn time_dependent_bounding_box(self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.map(|b| b.time_dependent_bounding_box(t0, t1))
            .collect()
    }
}

pub trait BoundedIteratorOps {
    fn bounding_box(self) -> BoundingBox;
}

impl<B: Bounded, Iter: Iterator<Item = B>> BoundedIteratorOps for Iter {
    fn bounding_box(self) -> BoundingBox {
        self.map(|b| b.bounding_box()).collect()
    }
}
