use super::VertexTuple;
use crate::{
    math::*, BoundingBox, Primitive, PrimitiveHitResult, Ray, RenderStatsCollector,
    UntransformedPrimitive,
};

#[derive(Debug, Clone)]
pub struct Triangle<'a> {
    vertex_indices: &'a [VertexTuple],
    vertices: &'a [Point3],
    uvs: &'a [Point2],
    normals: &'a [Vector3],
    tangents: &'a [Vector3],
}

impl<'a> Triangle<'a> {
    pub fn new(
        vertex_indices: &'a [VertexTuple],
        vertices: &'a [Point3],
        uvs: &'a [Point2],
        normals: &'a [Vector3],
        tangents: &'a [Vector3],
    ) -> Self {
        assert_eq!(vertex_indices.len(), 3, "Incorrect vertex size");
        for vt in vertex_indices {
            assert!(vt.vertex < vertices.len(), "Vertex out of range");
            assert!(
                vt.uv.map(|uv| uv < uvs.len()).unwrap_or(true),
                "UV out of range"
            );
            assert!(
                vt.normal
                    .map(|normal| normal < normals.len())
                    .unwrap_or(true),
                "Normal out of range"
            );
            assert!(
                vt.tangent
                    .map(|tangent| tangent < tangents.len())
                    .unwrap_or(true),
                "Tangent out of range"
            );
        }

        unsafe { Self::new_unchecked(vertex_indices, vertices, uvs, normals, tangents) }
    }

    pub(crate) unsafe fn new_unchecked(
        vertex_indices: &'a [VertexTuple],
        vertices: &'a [Point3],
        uvs: &'a [Point2],
        normals: &'a [Vector3],
        tangents: &'a [Vector3],
    ) -> Self {
        Self {
            vertex_indices,
            vertices,
            uvs,
            normals,
            tangents,
        }
    }

    pub(crate) fn vertex(
        &self,
        idx: usize,
    ) -> (&Point3, Option<&Point2>, Option<&Vector3>, Option<&Vector3>) {
        assert!(idx < 3, "Index out of range");
        unsafe { self.vertex_unchecked(idx) }
    }

    pub(crate) unsafe fn vertex_unchecked(
        &self,
        idx: usize,
    ) -> (&Point3, Option<&Point2>, Option<&Vector3>, Option<&Vector3>) {
        let vertex_tuple = self.vertex_indices.get_unchecked(idx);

        (
            self.vertices.get_unchecked(vertex_tuple.vertex),
            vertex_tuple.uv.map(|uv_idx| self.uvs.get_unchecked(uv_idx)),
            vertex_tuple
                .normal
                .map(|normal_idx| self.normals.get_unchecked(normal_idx)),
            vertex_tuple
                .tangent
                .map(|tangent_idx| self.tangents.get_unchecked(tangent_idx)),
        )
    }
}

impl Primitive for Triangle<'_> {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
        stats.count_triangle_test();
        let (pos0, uv0, normal0, tangent0) = self.vertex(0);
        let (pos1, uv1, normal1, tangent1) = self.vertex(1);
        let (pos2, uv2, normal2, tangent2) = self.vertex(2);

        let v0v1 = pos1 - pos0;
        let v0v2 = pos2 - pos0;

        let fixed_uv = point2(0.0, 0.0);
        let uv0 = uv0.unwrap_or(&fixed_uv);
        let uv1 = uv1.unwrap_or(&fixed_uv);
        let uv2 = uv2.unwrap_or(&fixed_uv);

        let triangle_normal = v0v1.cross(v0v2).normalize();
        let normal0 = normal0.unwrap_or(&triangle_normal);
        let normal1 = normal1.unwrap_or(&triangle_normal);
        let normal2 = normal2.unwrap_or(&triangle_normal);

        let triangle_tangent = v0v1.normalize();
        let tangent0 = tangent0.unwrap_or(&triangle_tangent);
        let tangent1 = tangent1.unwrap_or(&triangle_tangent);
        let tangent2 = tangent2.unwrap_or(&triangle_tangent);

        let pvec = ray.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        if det.abs() < constants::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let tvec = ray.origin - pos0;
        let u = tvec.dot(pvec) * inv_det;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let qvec = tvec.cross(v0v1);
        let v = ray.direction.dot(qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = v0v2.dot(qvec) * inv_det;
        if !(t_min..=t_max).contains(&t) {
            return None;
        }

        let hit_point = ray.origin + (t * ray.direction);

        // surface normal
        let outward_normal = (1.0 - u - v) * normal0;
        let outward_normal = outward_normal + (u * normal1);
        let outward_normal = outward_normal + (v * normal2);
        let outward_normal = outward_normal.normalize();

        // tangent
        let tangent = (1.0 - u - v) * tangent0;
        let tangent = tangent + (u * tangent1);
        let tangent = tangent + (v * tangent2);
        let tangent = tangent.normalize();

        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let surface_normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        let bitangent = outward_normal.cross(tangent).normalize();

        // texture coordinates
        let uv = (1.0 - u - v) * uv0;
        let uv = uv.add_element_wise(u * uv1);
        let uv = uv.add_element_wise(v * uv2);

        Some(PrimitiveHitResult::new(
            t,
            hit_point,
            surface_normal,
            tangent,
            bitangent,
            front_face,
            uv,
        ))
    }

    fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        (0..3)
            .map(|idx| unsafe { self.vertex_unchecked(idx).0 })
            .cloned()
            .collect()
    }
}

impl UntransformedPrimitive for Triangle<'_> {}
