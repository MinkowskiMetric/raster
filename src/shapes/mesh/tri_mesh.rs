use super::arch::mesh::*;
use super::TriangleVertex;
use crate::{
    math::*, Bounded, BoundingBox, DefaultPrimitive, DefaultSkinnable, DefaultTransformable,
    GeometryHitResult, Intersectable, Octree, Ray,
};
use anyhow::{anyhow, Result};
use std::iter::FromIterator;

#[derive(Clone, Debug)]
pub struct VertexTuple {
    pub vertex: usize,
    pub uv: Option<usize>,
    pub normal: Option<usize>,
    pub tangent: Option<usize>,
}

pub struct TriangleMesh {
    intersect_triangles: Octree<IntersectTriangle>,
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
        let intersect_triangles: Vec<_> = triangles
            .chunks_exact(3)
            .enumerate()
            .map(|(idx, verts)| {
                let start_index = idx * 3;
                let v0 = vertices[verts[0].vertex];
                let v1 = vertices[verts[1].vertex];
                let v2 = vertices[verts[2].vertex];

                IntersectTriangle::new(start_index, v0, v1, v2)
            })
            .collect();

        let intersect_triangles = Octree::snapshot_into_groups(
            intersect_triangles,
            TRIANGLE_MESH_GROUP_SIZE,
            -constants::INFINITY,
            constants::INFINITY,
        );

        Self {
            intersect_triangles,
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

    fn vertex(
        &self,
        index: usize,
    ) -> (&Point3, Option<&Point2>, Option<&Vector3>, Option<&Vector3>) {
        let vertex_tuple = &self.triangles[index];

        (
            &self.vertices[vertex_tuple.vertex],
            vertex_tuple.uv.map(|uv_idx| &self.uvs[uv_idx]),
            vertex_tuple.normal.map(|uv_idx| &self.normals[uv_idx]),
            vertex_tuple.tangent.map(|uv_idx| &self.tangents[uv_idx]),
        )
    }
}

impl Intersectable for TriangleMesh {
    type Result = GeometryHitResult;

    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<GeometryHitResult> {
        self.intersect_triangles
            .intersecting_blocks(ray, t_min, t_max)
            .filter_map(|intersect_triangle| {
                intersect_triangle_slice(intersect_triangle, ray, t_min, t_max)
            })
            .min_by(|l, r| l.t.partial_cmp(&r.t).unwrap())
            .map(|hit_result| {
                let (pos0, uv0, normal0, tangent0) = self.vertex(hit_result.start_index);
                let (pos1, uv1, normal1, tangent1) = self.vertex(hit_result.start_index + 1);
                let (pos2, uv2, normal2, tangent2) = self.vertex(hit_result.start_index + 2);

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

                let t = hit_result.t;
                let u = hit_result.u;
                let v = hit_result.v;

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

                GeometryHitResult::new(
                    t,
                    hit_point,
                    surface_normal,
                    tangent,
                    bitangent,
                    front_face,
                    uv,
                )
            })
    }
}

impl Bounded for TriangleMesh {
    fn bounding_box(&self) -> BoundingBox {
        self.intersect_triangles.bounding_box()
    }
}

impl DefaultTransformable for TriangleMesh {}
impl DefaultSkinnable for TriangleMesh {}
impl DefaultPrimitive for TriangleMesh {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::IntersectResult;

    #[test]
    fn test_triangle_mesh() {
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
        assert!(TriangleMesh::new([1], cube_vertices.iter().cloned()).is_err());
        // Out of range triangles are out of range
        assert!(TriangleMesh::new([0, 1, 9], cube_vertices.iter().cloned()).is_err());

        // One valid triangle is good
        let one_tri = TriangleMesh::new([0, 1, 2], cube_vertices.iter().cloned())
            .expect("Valid single triangle");

        let bounding = one_tri.bounding_box();
        assert_eq!(*bounding.min_point(), point3(-0.0001, -0.0001, -0.0001));
        assert_eq!(
            *bounding.max_point(),
            point3(2.0 + 0.0001, 2.0 + 0.0001, 0.0001)
        );

        let intersection = one_tri
            .intersect(
                &Ray::new(Point3::new(0.5, 0.5, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
                0.0,
                constants::INFINITY,
            )
            .expect("Missing intersection");

        assert_eq!(intersection.hit_point(), point3(0.5, 0.5, 0.0));
        assert_eq!(intersection.surface_normal(), vec3(0.0, 0.0, -1.0));
        assert_eq!(intersection.tangent(), vec3(0.0, 1.0, 0.0));
        assert_eq!(intersection.bitangent(), vec3(-1.0, 0.0, 0.0));

        let intersection = one_tri.intersect(
            &Ray::new(Point3::new(1.5, 1.5, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
            0.0,
            constants::INFINITY,
        );
        assert!(
            intersection.is_none(),
            "Unexpected intersection {:?}",
            intersection
        );

        // One valid triangle is good
        let one_tri = TriangleMesh::new([0, 2, 3], cube_vertices.iter().cloned())
            .expect("Valid single triangle");

        let bounding = one_tri.bounding_box();
        assert_eq!(*bounding.min_point(), point3(-0.0001, -0.0001, -0.0001));
        assert_eq!(
            *bounding.max_point(),
            point3(2.0 + 0.0001, 2.0 + 0.0001, 0.0001)
        );

        let intersection = one_tri
            .intersect(
                &Ray::new(Point3::new(0.5, 0.5, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
                0.0,
                constants::INFINITY,
            )
            .expect("Missing intersection");

        assert_eq!(intersection.hit_point(), point3(0.5, 0.5, 0.0));
        assert_eq!(intersection.surface_normal(), vec3(0.0, 0.0, -1.0));
        assert_eq!(intersection.tangent(), vec3(0.0, 1.0, 0.0));
        assert_eq!(intersection.bitangent(), vec3(-1.0, 0.0, 0.0));

        let intersection = one_tri.intersect(
            &Ray::new(Point3::new(1.5, 0.5, -10.0), vec3(0.0, 0.0, 1.0), 0.0),
            0.0,
            constants::INFINITY,
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
