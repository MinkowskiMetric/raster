use crate::{
    math::*, Bounded, BoundingBox, BoundingBoxIntersectionTester, CompoundPrimitive,
    CompoundVisible, DefaultSkinnable, DefaultTransformable, DynPrimitive, DynVisible,
    GeometryHitResult, IntersectResultIteratorOps, Intersectable, Primitive, Ray,
    TimeDependentBounded,
};
use core::ops::Range;
// How do we want the octree to work. Octree is only concerned with bounding. It does not require that it's contents be intersectable

// Octree snapshots the times when it is created and as such does not implement TimeDependentBounded. If you want to be time dependent
// then you can create a new snapshot.
pub trait OctreeItemSource {
    type Item: TimeDependentBounded;

    fn into_boxed_slice(self) -> Box<[Self::Item]>;
}

impl<P: TimeDependentBounded> OctreeItemSource for Vec<P> {
    type Item = P;

    fn into_boxed_slice(self) -> Box<[Self::Item]> {
        self.into_boxed_slice()
    }
}

impl OctreeItemSource for CompoundPrimitive {
    type Item = DynPrimitive;

    fn into_boxed_slice(self) -> Box<[Self::Item]> {
        self.into_inner().into_boxed_slice()
    }
}

impl OctreeItemSource for CompoundVisible {
    type Item = DynVisible;

    fn into_boxed_slice(self) -> Box<[Self::Item]> {
        self.into_inner().into_boxed_slice()
    }
}

impl<P: TimeDependentBounded> OctreeItemSource for Octree<P> {
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
    pub fn read_axis(&self, p: &Point3) -> FloatType {
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

struct OctreeEntry {
    bounding_box: BoundingBox,
    range: std::ops::Range<usize>,
    children: Option<Box<(OctreeEntry, OctreeEntry)>>,
}

fn sort_and_divide<P: TimeDependentBounded>(
    items: &mut [P],
    range: &Range<usize>,
    axis: SortAxis,
    group_size_hint: usize,
    t0: FloatType,
    t1: FloatType,
) -> OctreeEntry {
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

    OctreeEntry {
        bounding_box,
        range: range.clone(),
        children,
    }
}

pub struct OctreeBlockIntersectIterator<'a, P: TimeDependentBounded> {
    items: &'a [P],
    work_stack: Vec<&'a OctreeEntry>,
    intersection_tester: BoundingBoxIntersectionTester,
    t_min: FloatType,
    t_max: FloatType,
}

impl<'a, P: TimeDependentBounded> OctreeBlockIntersectIterator<'a, P> {
    fn new(
        items: &'a [P],
        root: Option<&'a OctreeEntry>,
        ray: &'a Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Self {
        // Pre-allocating some space in the octree work stack allows us to
        // avoid allocating during the octree walk, which speeds things up a bit
        // in the general case. We can compute the worst case depth
        // anyway. We add two to the log so that we round up and include the root node
        let maximum_stack_depth = ((items.len() as f64).log2() + 2.0) as usize;

        let mut work_stack = Vec::with_capacity(maximum_stack_depth);

        if let Some(root) = root {
            work_stack.push(root);
        }

        Self {
            items,
            work_stack,
            intersection_tester: BoundingBoxIntersectionTester::new(ray),
            t_min,
            t_max,
        }
    }
}

impl<'a, P: TimeDependentBounded> Iterator for OctreeBlockIntersectIterator<'a, P> {
    type Item = &'a [P];

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(top) = self.work_stack.pop() {
            // Check if the ray intersects the node
            if top.bounding_box.intersect_with_tester(
                &self.intersection_tester,
                self.t_min,
                self.t_max,
            ) {
                if let Some((left, right)) = top.children.as_deref() {
                    // This is a branch node, so push its children
                    self.work_stack.push(right);
                    self.work_stack.push(left);
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

pub struct Octree<P: TimeDependentBounded> {
    items: Box<[P]>,
    time_range: (FloatType, FloatType),
    octree: Option<OctreeEntry>,
}

impl<P: TimeDependentBounded> Octree<P> {
    pub fn snapshot_into_groups<Items: OctreeItemSource<Item = P>>(
        items: Items,
        group_size_hint: usize,
        t0: FloatType,
        t1: FloatType,
    ) -> Self {
        let mut items = items.into_boxed_slice();
        let octree = if items.is_empty() {
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
            octree,
        }
    }
    pub fn snapshot<Items: OctreeItemSource<Item = P>>(
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

// We do not implement TimeDependentBounded for Octree. If you want a different time then you can always snapshot again
impl<P: TimeDependentBounded> Bounded for Octree<P> {
    fn bounding_box(&self) -> BoundingBox {
        self.octree
            .as_ref()
            .map(|o| o.bounding_box)
            .unwrap_or_else(BoundingBox::empty_box)
    }
}

impl<P: TimeDependentBounded> Octree<P> {
    pub fn intersecting_blocks<'a>(
        &'a self,
        ray: &'a Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> OctreeBlockIntersectIterator<'a, P> {
        // If the ray time is out of range then we panic
        assert!(
            ray.time >= self.min_time() && ray.time <= self.max_time(),
            "Ray time is out of range for octree"
        );

        OctreeBlockIntersectIterator::new(
            self.items.as_ref(),
            self.octree.as_ref(),
            ray,
            t_min,
            t_max,
        )
    }
}

impl<P: TimeDependentBounded + Intersectable> Intersectable for Octree<P> {
    type Result = P::Result;

    fn intersect(&self, ray: &Ray, t_min: FloatType, t_max: FloatType) -> Option<Self::Result> {
        self.intersecting_blocks(ray, t_min, t_max)
            .flat_map(|f| f.iter())
            .filter_map(|i| i.intersect(ray, t_min, t_max))
            .nearest()
    }
}

impl<P: TimeDependentBounded + Intersectable> DefaultTransformable for Octree<P> {}
impl<P: TimeDependentBounded + Intersectable> DefaultSkinnable for Octree<P> {}

impl<P: 'static + TimeDependentBounded + Intersectable<Result = GeometryHitResult> + Primitive>
    Primitive for Octree<P>
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
