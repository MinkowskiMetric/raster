use crate::{
    math::*, Bounded, BoundingBox, BoundingBoxIntersectionTester, CompoundPrimitive,
    CompoundVisible, DefaultSkinnable, DefaultTransformable, DynPrimitive, DynVisible,
    GeometryHitResult, IntersectResultIteratorOps, Intersectable, Primitive, Ray,
    TimeDependentBounded,
};
use core::ops::Range;
use std::mem::MaybeUninit;
// How do we want the kd tree to work. KDTree is only concerned with bounding. It does not require that it's contents be intersectable

// KDTree snapshots the times when it is created and as such does not implement TimeDependentBounded. If you want to be time dependent
// then you can create a new snapshot.
pub trait KDTreeItemSource {
    type Item: TimeDependentBounded;

    fn into_boxed_slice(self) -> Box<[Self::Item]>;
}

impl<P: TimeDependentBounded> KDTreeItemSource for Vec<P> {
    type Item = P;

    fn into_boxed_slice(self) -> Box<[Self::Item]> {
        self.into_boxed_slice()
    }
}

impl KDTreeItemSource for CompoundPrimitive {
    type Item = DynPrimitive;

    fn into_boxed_slice(self) -> Box<[Self::Item]> {
        self.into_inner().into_boxed_slice()
    }
}

impl KDTreeItemSource for CompoundVisible {
    type Item = DynVisible;

    fn into_boxed_slice(self) -> Box<[Self::Item]> {
        self.into_inner().into_boxed_slice()
    }
}

impl<P: TimeDependentBounded> KDTreeItemSource for KDTree<P> {
    type Item = P;

    fn into_boxed_slice(self) -> Box<[Self::Item]> {
        self.items
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SortAxis {
    X,
    Y,
    Z,
}

impl SortAxis {
    pub fn read_axis(&self, p: Point3) -> FloatType {
        match *self {
            SortAxis::X => p.x,
            SortAxis::Y => p.y,
            SortAxis::Z => p.z,
        }
    }

    pub fn read_midpoint(&self, bb: &BoundingBox) -> FloatType {
        (self.read_axis(bb.min_point()) + self.read_axis(bb.min_point())) / 2.0
    }

    pub fn compare(&self, l: &BoundingBox, r: &BoundingBox) -> std::cmp::Ordering {
        self.read_midpoint(l)
            .partial_cmp(&(self.read_midpoint(r)))
            .unwrap()
    }

    pub fn next_axis(&self) -> Self {
        match self {
            SortAxis::X => SortAxis::Y,
            SortAxis::Y => SortAxis::Z,
            SortAxis::Z => SortAxis::X,
        }
    }
}

struct KDTreeEntry {
    bounding_box: BoundingBox,
    range: std::ops::Range<usize>,
    children: Option<Box<(KDTreeEntry, KDTreeEntry)>>,
}

fn sort_and_divide<P: TimeDependentBounded>(
    items: &mut [P],
    range: &Range<usize>,
    axis: SortAxis,
    group_size_hint: usize,
    t0: FloatType,
    t1: FloatType,
) -> KDTreeEntry {
    // Start by sorting the range
    items[range.clone()].sort_by(|l, r| {
        let l = l.time_dependent_bounding_box(t0, t1);
        let r = r.time_dependent_bounding_box(t0, t1);

        axis.compare(&l, &r)
    });

    // We add one here so we round up if it is odd. We want the left half to always
    // be the bigger half
    let bounding_box = items[range.clone()]
        .iter()
        .map(|p| p.time_dependent_bounding_box(t0, t1))
        .collect();

    let children = if range.len() <= group_size_hint.min(1) {
        // We have fewer than the group size hint, so we want to stop looking here.
        None
    } else {
        let left_range = range.start..(range.start + ((range.len() + 1) / 2));
        let left = sort_and_divide(
            items,
            &left_range,
            axis.next_axis(),
            group_size_hint,
            t0,
            t1,
        );

        let right_range = left_range.end..range.end;
        let right = sort_and_divide(
            items,
            &right_range,
            axis.next_axis(),
            group_size_hint,
            t0,
            t1,
        );

        Some(Box::new((left, right)))
    };

    KDTreeEntry {
        bounding_box,
        range: range.clone(),
        children,
    }
}

// Since the KDTree is always perfectly balanced, a limit of 32 deep is enough
// for 4 billion objects. This should be big enough, and avoids the need to allocate
// on every ray intersection. If we exceed the size of this stack, we will panic
const KDTREE_MAX_DEPTH: usize = 32;

pub struct KDTreeBlockIntersectIterator<'a, P: TimeDependentBounded> {
    items: &'a [P],
    work_stack: [MaybeUninit<&'a KDTreeEntry>; KDTREE_MAX_DEPTH],
    work_stack_top: usize,
    intersection_tester: BoundingBoxIntersectionTester,
    t_min: FloatType,
    t_max: FloatType,
}

impl<'a, P: TimeDependentBounded> KDTreeBlockIntersectIterator<'a, P> {
    fn new(
        items: &'a [P],
        root: Option<&'a KDTreeEntry>,
        ray: &'a Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Self {
        let mut work_stack = MaybeUninit::uninit_array();
        let mut work_stack_top = 0;

        if let Some(root) = root {
            work_stack[0].write(root);
            work_stack_top = 1;
        }

        Self {
            items,
            work_stack,
            work_stack_top,
            intersection_tester: BoundingBoxIntersectionTester::new(ray),
            t_min,
            t_max,
        }
    }

    fn push_work_stack(&mut self, entry: &'a KDTreeEntry) {
        assert!(
            self.work_stack_top < self.work_stack.len(),
            "Work stack exceeds depth of {}",
            KDTREE_MAX_DEPTH
        );

        self.work_stack[self.work_stack_top].write(entry);
        self.work_stack_top += 1;
    }

    fn pop_work_stack(&mut self) -> Option<&'a KDTreeEntry> {
        if self.work_stack_top > 0 {
            self.work_stack_top -= 1;
            Some(unsafe { self.work_stack[self.work_stack_top].assume_init() })
        } else {
            None
        }
    }
}

impl<'a, P: TimeDependentBounded> Drop for KDTreeBlockIntersectIterator<'a, P> {
    fn drop(&mut self) {
        for idx in 0..self.work_stack_top {
            unsafe { self.work_stack[idx].assume_init_drop() };
        }
    }
}

impl<'a, P: TimeDependentBounded> Iterator for KDTreeBlockIntersectIterator<'a, P> {
    type Item = &'a [P];

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(top) = self.pop_work_stack() {
            // Check if the ray intersects the node
            if top.bounding_box.intersect_with_tester(
                &self.intersection_tester,
                self.t_min,
                self.t_max,
            ) {
                if let Some((left, right)) = top.children.as_deref() {
                    // This is a branch node, so push its children
                    self.push_work_stack(right);
                    self.push_work_stack(left);
                } else {
                    // This is a leaf node, so return it
                    return Some(&self.items[top.range.clone()]);
                }
            }
        }

        // No intersects found
        None
    }
}

pub struct KDTree<P: TimeDependentBounded> {
    items: Box<[P]>,
    time_range: (FloatType, FloatType),
    kdtree: Option<KDTreeEntry>,
}

impl<P: TimeDependentBounded> KDTree<P> {
    pub fn snapshot_into_groups<Items: KDTreeItemSource<Item = P>>(
        items: Items,
        group_size_hint: usize,
        t0: FloatType,
        t1: FloatType,
    ) -> Self {
        let mut items = items.into_boxed_slice();
        let kdtree = if items.is_empty() {
            None
        } else {
            let range = 0..items.len();
            Some(sort_and_divide(
                items.as_mut(),
                &range,
                SortAxis::X,
                group_size_hint,
                t0,
                t1,
            ))
        };

        Self {
            items,
            time_range: (t0, t1),
            kdtree,
        }
    }
    pub fn snapshot<Items: KDTreeItemSource<Item = P>>(
        items: Items,
        t0: FloatType,
        t1: FloatType,
    ) -> Self {
        Self::snapshot_into_groups(items, 1, t0, t1)
    }

    pub fn min_time(&self) -> FloatType {
        self.time_range.0
    }

    pub fn max_time(&self) -> FloatType {
        self.time_range.1
    }
}

// We do not implement TimeDependentBounded for KDTree. If you want a different time then you can always snapshot again
impl<P: TimeDependentBounded> Bounded for KDTree<P> {
    fn bounding_box(&self) -> BoundingBox {
        self.kdtree
            .as_ref()
            .map(|o| o.bounding_box)
            .unwrap_or_else(BoundingBox::empty_box)
    }
}

impl<P: TimeDependentBounded> KDTree<P> {
    pub fn intersecting_blocks<'a>(
        &'a self,
        ray: &'a Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> KDTreeBlockIntersectIterator<'a, P> {
        // If the ray time is out of range then we panic
        assert!(
            ray.time() >= self.min_time() && ray.time() <= self.max_time(),
            "Ray time is out of range for kdtree"
        );

        KDTreeBlockIntersectIterator::new(
            self.items.as_ref(),
            self.kdtree.as_ref(),
            ray,
            t_min,
            t_max,
        )
    }
}

impl<P: TimeDependentBounded + Intersectable> Intersectable for KDTree<P> {
    type Result = P::Result;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result> {
        self.intersecting_blocks(ray, t_min, t_max)
            .flat_map(|f| f.iter())
            .filter_map(|i| i.intersect(ray, t_min, t_max))
            .nearest()
    }
}

impl<P: TimeDependentBounded + Intersectable> DefaultTransformable for KDTree<P> {}
impl<P: TimeDependentBounded + Intersectable> DefaultSkinnable for KDTree<P> {}

impl<P: 'static + TimeDependentBounded + Intersectable<Result = GeometryHitResult> + Primitive>
    Primitive for KDTree<P>
{
    fn to_dyn_primitive(self) -> DynPrimitive {
        DynPrimitive::new(self)
    }

    fn decompose_box(self: Box<Self>) -> CompoundPrimitive {
        (*self).decompose()
    }

    fn decompose(self) -> CompoundPrimitive {
        self.items
            .into_vec()
            .into_iter()
            .map(|p| p.to_dyn_primitive())
            .collect()
    }
}
