use crate::{
    math::*, BoundingBox, Primitive, PrimitiveHitResult, PrimitiveIteratorOps, Ray,
    RenderStatsCollector,
};
use anyhow::{anyhow, Result};
use std::slice::ChunksExact;

use super::UntransformedPrimitive;

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    triangles: Vec<usize>,
    vertices: Vec<Point3>,
}

#[derive(Debug, Clone)]
pub struct Triangle<'a> {
    vertex_indices: &'a [usize],
    vertices: &'a [Point3],
}

impl Triangle<'_> {
    #[allow(dead_code)]
    fn vertices(&self) -> impl Iterator<Item = &Point3> {
        (0..3).map(move |idx| self.vertex(idx))
    }

    fn vertex(&self, idx: usize) -> &Point3 {
        unsafe {
            self.vertices
                .get_unchecked(*self.vertex_indices.get_unchecked(idx))
        }
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
        let v0v1 = self.vertex(1) - self.vertex(0);
        let v0v2 = self.vertex(2) - self.vertex(0);
        let pvec = ray.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        if det.abs() < constants::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let tvec = ray.origin - self.vertex(0);
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

        // Faking these for now
        let outward_normal = vec3(0.0, 0.0, 1.0);
        let tangent = vec3(0.0, 1.0, 0.0);
        let front_face = ray.direction.dot(outward_normal) < 0.0;

        let surface_normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        let bitangent = outward_normal.cross(tangent).normalize();

        let uv = (u, v);
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
        self.vertices().collect()
    }
}

impl UntransformedPrimitive for Triangle<'_> {}

impl TriangleMesh {
    #[allow(dead_code)]
    pub fn new(
        triangles: impl IntoIterator<Item = usize>,
        vertices: impl IntoIterator<Item = Point3>,
    ) -> Result<Self> {
        let vertices: Vec<_> = vertices.into_iter().collect();
        let triangles: Vec<_> = triangles
            .into_iter()
            .map(|idx| {
                if idx < vertices.len() {
                    Ok(idx)
                } else {
                    Err(anyhow!("Vertex index {} out of range", idx))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        if triangles.len() % 3 == 0 {
            Ok(Self {
                triangles,
                vertices,
            })
        } else {
            Err(anyhow!("Triangle index length must be a multiple of 3"))
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.triangles.len() / 3
    }
}

impl Primitive for TriangleMesh {
    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        stats: &mut dyn RenderStatsCollector,
    ) -> Option<PrimitiveHitResult> {
        self.iter().intersect(ray, t_min, t_max, stats)
    }

    fn bounding_box(&self, t0: FloatType, t1: FloatType) -> BoundingBox {
        self.iter().bounding_box(t0, t1)
    }
}

impl UntransformedPrimitive for TriangleMesh {}

#[derive(Debug, Clone)]
pub struct TriangleIterator<'a> {
    triangles: ChunksExact<'a, usize>,
    vertices: &'a [Point3],
}

impl<'a> Iterator for TriangleIterator<'a> {
    type Item = Triangle<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.triangles.next().map(|vertex_indices| Self::Item {
            vertex_indices,
            vertices: self.vertices,
        })
    }
}

impl TriangleMesh {
    #[allow(dead_code)]
    pub fn iter(&'_ self) -> TriangleIterator<'_> {
        TriangleIterator {
            triangles: self.triangles.chunks_exact(3),
            vertices: &self.vertices,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_triangle_mesh() {
        let mut stats = crate::TracingStats::new();

        let cube_vertices = [
            point3(0.0, 0.0, 0.0),
            point3(2.0, 0.0, 0.0),
            point3(0.0, 2.0, 0.0),
            point3(2.0, 2.0, 0.0),
            point3(0.0, 0.0, 2.0),
            point3(2.0, 0.0, 2.0),
            point3(0.0, 2.0, 2.0),
            point3(2.0, 2.0, 2.0),
        ];

        // An empty triangle mesh is fine - it just has no triangles in it
        TriangleMesh::new([], cube_vertices.iter().cloned()).expect("Valid empty mesh");
        // The triangle index needs to be a multiple of 3
        TriangleMesh::new([1], cube_vertices.iter().cloned()).expect_err("Invalid mesh");
        // Out of range triangles are out of range
        TriangleMesh::new([0, 1, 9], cube_vertices.iter().cloned()).expect_err("Invalid mesh");

        // One valid triangle is good
        let one_tri = TriangleMesh::new([0, 1, 2], cube_vertices.iter().cloned())
            .expect("Valid single triangle");
        assert_eq!(one_tri.len(), 1);
        let tri = one_tri.iter().next().expect("Missing triangle");

        let bounding = tri.bounding_box(0.0, 0.0);
        assert_eq!(*bounding.min_point(), point3(-0.0001, -0.0001, -0.0001));
        assert_eq!(
            *bounding.max_point(),
            point3(2.0 + 0.0001, 2.0 + 0.0001, 0.0001)
        );

        let intersection = tri
            .intersect(
                &Ray::new(Point3::new(0.5, 0.5, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
                0.0,
                constants::INFINITY,
                &mut stats,
            )
            .expect("Missing intersection");

        assert_eq!(intersection.hit_point(), point3(0.5, 0.5, 0.0));
        assert_eq!(intersection.surface_normal(), vec3(0.0, 0.0, -1.0));
        assert_eq!(intersection.tangent(), vec3(0.0, 1.0, 0.0));
        assert_eq!(intersection.bitangent(), vec3(-1.0, 0.0, 0.0));

        let intersection = tri.intersect(
            &Ray::new(Point3::new(1.5, 1.5, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
            0.0,
            constants::INFINITY,
            &mut stats,
        );
        assert!(
            intersection.is_none(),
            "Unexpected intersection {:?}",
            intersection
        );

        // One valid triangle is good
        let one_tri = TriangleMesh::new([0, 2, 3], cube_vertices.iter().cloned())
            .expect("Valid single triangle");
        assert_eq!(one_tri.len(), 1);
        let tri = one_tri.iter().next().expect("Missing triangle");

        let bounding = tri.bounding_box(0.0, 0.0);
        assert_eq!(*bounding.min_point(), point3(-0.0001, -0.0001, -0.0001));
        assert_eq!(
            *bounding.max_point(),
            point3(2.0 + 0.0001, 2.0 + 0.0001, 0.0001)
        );

        let intersection = tri
            .intersect(
                &Ray::new(Point3::new(0.5, 0.5, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
                0.0,
                constants::INFINITY,
                &mut stats,
            )
            .expect("Missing intersection");

        assert_eq!(intersection.hit_point(), point3(0.5, 0.5, 0.0));
        assert_eq!(intersection.surface_normal(), vec3(0.0, 0.0, -1.0));
        assert_eq!(intersection.tangent(), vec3(0.0, 1.0, 0.0));
        assert_eq!(intersection.bitangent(), vec3(-1.0, 0.0, 0.0));

        let intersection = tri.intersect(
            &Ray::new(Point3::new(1.5, 0.5, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
            0.0,
            constants::INFINITY,
            &mut stats,
        );
        assert!(
            intersection.is_none(),
            "Unexpected intersection {:?}",
            intersection
        );
    }
}

pub mod factories {
    use super::*;

    pub fn triangle_mesh(
        triangles: impl IntoIterator<Item = usize>,
        vertices: impl IntoIterator<Item = Point3>,
    ) -> Result<TriangleMesh> {
        TriangleMesh::new(triangles, vertices)
    }
}
