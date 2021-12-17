use crate::{math::*, BaseMaterial};
use std::sync::Arc;

pub trait IntersectResult {
    fn distance(&self) -> FloatType;
    fn set_distance(&mut self, distance: FloatType);

    fn hit_point(&self) -> Point3;
    fn set_hit_point(&mut self, hit_point: Point3);

    fn surface_normal(&self) -> Vector3;
    fn set_surface_normal(&mut self, surface_normal: Vector3);

    fn tangent(&self) -> Vector3;
    fn set_tangent(&mut self, tangent: Vector3);

    fn bitangent(&self) -> Vector3;
    fn set_bitangent(&mut self, bitangent: Vector3);

    fn front_face(&self) -> bool;
    fn set_front_face(&mut self, front_face: bool);

    fn uv(&self) -> Point2;

    fn as_geometry_hit_result(&self) -> GeometryHitResult;
}

pub trait IntersectResultIteratorOps {
    type Result: IntersectResult;

    fn nearest(self) -> Option<Self::Result>;
}

impl<'a, I: IntersectResult, Iter: Iterator<Item = I>> IntersectResultIteratorOps for Iter {
    type Result = I;

    fn nearest(self) -> Option<Self::Result> {
        self.min_by(|xr, yr| {
            let xr_distance = xr.distance();
            let yr_distance = yr.distance();

            xr_distance
                .partial_cmp(&yr_distance)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
    }
}

#[derive(Debug, Clone)]
pub struct GeometryHitResult {
    distance: FloatType,
    hit_point: Point3,
    surface_normal: Vector3,
    tangent: Vector3,
    bitangent: Vector3,
    front_face: bool,
    uv: Point2,
}

impl GeometryHitResult {
    pub fn new(
        distance: FloatType,
        hit_point: Point3,
        surface_normal: Vector3,
        tangent: Vector3,
        bitangent: Vector3,
        front_face: bool,
        uv: Point2,
    ) -> Self {
        Self {
            distance,
            hit_point,
            surface_normal,
            tangent,
            bitangent,
            front_face,
            uv,
        }
    }
}

impl IntersectResult for GeometryHitResult {
    fn distance(&self) -> FloatType {
        self.distance
    }

    fn set_distance(&mut self, distance: FloatType) {
        self.distance = distance
    }

    fn hit_point(&self) -> Point3 {
        self.hit_point
    }

    fn set_hit_point(&mut self, hit_point: Point3) {
        self.hit_point = hit_point
    }

    fn surface_normal(&self) -> Vector3 {
        self.surface_normal
    }

    fn set_surface_normal(&mut self, surface_normal: Vector3) {
        self.surface_normal = surface_normal
    }

    fn tangent(&self) -> Vector3 {
        self.tangent
    }

    fn set_tangent(&mut self, tangent: Vector3) {
        self.tangent = tangent
    }

    fn bitangent(&self) -> Vector3 {
        self.bitangent
    }

    fn set_bitangent(&mut self, bitangent: Vector3) {
        self.bitangent = bitangent
    }

    fn front_face(&self) -> bool {
        self.front_face
    }

    fn set_front_face(&mut self, front_face: bool) {
        self.front_face = front_face
    }

    fn uv(&self) -> Point2 {
        self.uv
    }

    fn as_geometry_hit_result(&self) -> GeometryHitResult {
        self.clone()
    }
}

pub struct SkinnedHitResult {
    hit_result: GeometryHitResult,
    material: Arc<dyn BaseMaterial>,
}

impl SkinnedHitResult {
    pub fn new(hit_result: GeometryHitResult, material: Arc<dyn BaseMaterial>) -> Self {
        Self {
            hit_result,
            material,
        }
    }

    pub fn hit_result(&self) -> &GeometryHitResult {
        &self.hit_result
    }

    pub fn hit_result_mut(&mut self) -> &mut GeometryHitResult {
        &mut self.hit_result
    }

    pub fn material(&self) -> &dyn BaseMaterial {
        self.material.as_ref()
    }

    pub fn split(self) -> (GeometryHitResult, Arc<dyn BaseMaterial>) {
        (self.hit_result, self.material)
    }
}

impl IntersectResult for SkinnedHitResult {
    fn distance(&self) -> FloatType {
        self.hit_result().distance
    }

    fn set_distance(&mut self, distance: FloatType) {
        self.hit_result_mut().distance = distance
    }

    fn hit_point(&self) -> Point3 {
        self.hit_result().hit_point
    }

    fn set_hit_point(&mut self, hit_point: Point3) {
        self.hit_result_mut().hit_point = hit_point
    }

    fn surface_normal(&self) -> Vector3 {
        self.hit_result().surface_normal
    }

    fn set_surface_normal(&mut self, surface_normal: Vector3) {
        self.hit_result_mut().surface_normal = surface_normal
    }

    fn tangent(&self) -> Vector3 {
        self.hit_result().tangent
    }

    fn set_tangent(&mut self, tangent: Vector3) {
        self.hit_result_mut().tangent = tangent
    }

    fn bitangent(&self) -> Vector3 {
        self.hit_result().bitangent
    }

    fn set_bitangent(&mut self, bitangent: Vector3) {
        self.hit_result_mut().bitangent = bitangent
    }

    fn front_face(&self) -> bool {
        self.hit_result().front_face
    }

    fn set_front_face(&mut self, front_face: bool) {
        self.hit_result_mut().front_face = front_face
    }

    fn uv(&self) -> Point2 {
        self.hit_result().uv
    }

    fn as_geometry_hit_result(&self) -> GeometryHitResult {
        self.hit_result.clone()
    }
}
