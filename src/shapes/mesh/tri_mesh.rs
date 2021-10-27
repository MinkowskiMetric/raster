use super::{Triangle, TriangleVertex};
use crate::{
    math::*, BoundingBox, Primitive, PrimitiveHitResult, PrimitiveIteratorOps, Ray,
    RenderStatsCollector, UntransformedPrimitive,
};
use anyhow::{anyhow, Result};
use std::{iter::FromIterator, slice::ChunksExact};

#[derive(Clone, Debug)]
pub struct VertexTuple {
    pub vertex: usize,
    pub uv: Option<usize>,
    pub normal: Option<usize>,
    pub tangent: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    triangles: Vec<VertexTuple>,
    vertices: Vec<Point3>,
    uvs: Vec<Point2>,
    normals: Vec<Vector3>,
    tangents: Vec<Vector3>,
}

impl TriangleMesh {
    pub(super) unsafe fn from_split_unchecked(
        triangles: Vec<VertexTuple>,
        vertices: Vec<Point3>,
        uvs: Vec<Point2>,
        normals: Vec<Vector3>,
        tangents: Vec<Vector3>,
    ) -> Self {
        Self {
            triangles,
            vertices,
            uvs,
            normals,
            tangents,
        }
    }

    pub fn from_split(
        triangles: impl IntoIterator<Item = VertexTuple>,
        vertices: impl IntoIterator<Item = Point3>,
        uvs: impl IntoIterator<Item = Point2>,
        normals: impl IntoIterator<Item = Vector3>,
        tangents: impl IntoIterator<Item = Vector3>,
    ) -> Result<Self> {
        let vertices: Vec<_> = vertices.into_iter().collect();
        let uvs: Vec<_> = uvs.into_iter().collect();
        let normals: Vec<_> = normals.into_iter().collect();
        let tangents: Vec<_> = tangents.into_iter().collect();

        let triangles: Vec<_> = triangles
            .into_iter()
            .map(|vt| -> Result<VertexTuple> {
                if vt.vertex >= vertices.len() {
                    return Err(anyhow!("Vertex {} out of range", vt.vertex));
                }

                if let Some(uv) = vt.uv {
                    if uv >= uvs.len() {
                        return Err(anyhow!("UV {} out of range", uv));
                    }
                }

                if let Some(normal) = vt.normal {
                    if normal >= normals.len() {
                        return Err(anyhow!("Normal {} out of range", normal));
                    }
                }

                if let Some(tangent) = vt.tangent {
                    if tangent >= tangents.len() {
                        return Err(anyhow!("Tangent {} out of range", tangent));
                    }
                }

                Ok(VertexTuple {
                    vertex: vt.vertex,
                    uv: Some(vt.vertex),
                    normal: Some(vt.vertex),
                    tangent: Some(vt.vertex),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        if triangles.len() % 3 == 0 {
            Ok(unsafe { Self::from_split_unchecked(triangles, vertices, uvs, normals, tangents) })
        } else {
            Err(anyhow!("Triangle index length must be a multiple of 3"))
        }
    }

    pub fn new(
        triangles: impl IntoIterator<Item = usize>,
        vertices: impl IntoIterator<Item = TriangleVertex>,
    ) -> Result<Self> {
        // This is a pain and we should lose this flow as soon
        // as possible because we make a lot of copies
        let full_vertices = Vec::from_iter(vertices);
        let vertices = Vec::from_iter(full_vertices.iter().map(TriangleVertex::pos));
        let uvs = Vec::from_iter(full_vertices.iter().map(TriangleVertex::uv));
        let normals = Vec::from_iter(full_vertices.iter().map(TriangleVertex::surface_normal));
        let tangents = Vec::from_iter(full_vertices.iter().map(TriangleVertex::tangent));

        let triangles = Vec::from_iter(triangles.into_iter().map(|vertex| VertexTuple {
            vertex,
            uv: Some(vertex),
            normal: Some(vertex),
            tangent: Some(vertex),
        }));

        Self::from_split(triangles, vertices, uvs, normals, tangents)
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
    triangles: ChunksExact<'a, VertexTuple>,
    vertices: &'a [Point3],
    uvs: &'a [Point2],
    normals: &'a [Vector3],
    tangents: &'a [Vector3],
}

impl<'a> Iterator for TriangleIterator<'a> {
    type Item = Triangle<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.triangles.next().map(|vertex_indices| {
            Self::Item::new(
                vertex_indices,
                self.vertices,
                self.uvs,
                self.normals,
                self.tangents,
            )
        })
    }
}

impl TriangleMesh {
    #[allow(dead_code)]
    pub fn iter(&'_ self) -> TriangleIterator<'_> {
        TriangleIterator {
            triangles: self.triangles.chunks_exact(3),
            vertices: &self.vertices,
            uvs: &self.uvs,
            normals: &self.normals,
            tangents: &self.tangents,
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
            TriangleVertex::new(
                point3(0.0, 0.0, 0.0),
                point2(0.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
            ),
            TriangleVertex::new(
                point3(2.0, 0.0, 0.0),
                point2(1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
            ),
            TriangleVertex::new(
                point3(0.0, 2.0, 0.0),
                point2(0.0, 1.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
            ),
            TriangleVertex::new(
                point3(2.0, 2.0, 0.0),
                point2(1.0, 1.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
            ),
            TriangleVertex::new(
                point3(0.0, 0.0, 2.0),
                point2(0.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
            ),
            TriangleVertex::new(
                point3(2.0, 0.0, 2.0),
                point2(1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
            ),
            TriangleVertex::new(
                point3(0.0, 2.0, 2.0),
                point2(0.0, 1.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
            ),
            TriangleVertex::new(
                point3(2.0, 2.0, 2.0),
                point2(1.0, 1.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
            ),
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
        vertices: impl IntoIterator<Item = TriangleVertex>,
    ) -> Result<TriangleMesh> {
        TriangleMesh::new(triangles, vertices)
    }
}
