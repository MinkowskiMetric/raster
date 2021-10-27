use super::{TriangleMesh, VertexTuple};
use crate::math::*;
use anyhow::{anyhow, Result};
use obj::{Group, IndexTuple, Obj, Object};
use std::{collections::HashMap, path::Path};

fn validate_tuple(
    tuple: IndexTuple,
    position: &[[f32; 3]],
    texture: &[[f32; 2]],
    normal: &[[f32; 3]],
) -> Result<VertexTuple> {
    if tuple.0 >= position.len() {
        return Err(anyhow!("Position index {} out of range", tuple.0));
    }

    if tuple.1.map(|idx| idx >= texture.len()).unwrap_or(false) {
        return Err(anyhow!("Texture index {} out of scope", tuple.1.unwrap()));
    }

    if tuple.2.map(|idx| idx >= normal.len()).unwrap_or(false) {
        return Err(anyhow!("Normal index {} out of scope", tuple.1.unwrap()));
    }

    Ok(VertexTuple {
        vertex: tuple.0,
        uv: tuple.1,
        normal: tuple.2,
        tangent: None,
    })
}

fn group_to_mesh(
    group: &Group,
    position: &[[f32; 3]],
    texture: &[[f32; 2]],
    normal: &[[f32; 3]],
) -> Result<(String, TriangleMesh)> {
    let noof_triangles: usize = group
        .polys
        .iter()
        .filter_map(|poly| {
            if poly.0.len() >= 3 {
                Some(poly.0.len() - 2)
            } else {
                None
            }
        })
        .sum();
    let mut indices = Vec::with_capacity(3 * noof_triangles);

    for poly in group.polys.iter() {
        if poly.0.len() >= 3 {
            for idx in 1..(poly.0.len() - 1) {
                indices.push(validate_tuple(poly.0[0], position, texture, normal)?);
                indices.push(validate_tuple(poly.0[idx], position, texture, normal)?);
                indices.push(validate_tuple(poly.0[idx + 1], position, texture, normal)?);
            }
        }
    }

    let position = position
        .iter()
        .map(|pos| point3(pos[0], pos[1], pos[2]))
        .collect::<Vec<_>>();
    let texture = texture
        .iter()
        .map(|texture| point2(texture[0], texture[1]))
        .collect::<Vec<_>>();
    let normal = normal
        .iter()
        .map(|normal| vec3(normal[0], normal[1], normal[2]))
        .collect::<Vec<_>>();

    let mesh = unsafe {
        TriangleMesh::from_split_unchecked(indices, position, texture, normal, Vec::new())
    };
    Ok((group.name.to_string(), mesh))
}

fn obj_to_mesh(
    obj: &Object,
    position: &[[f32; 3]],
    texture: &[[f32; 2]],
    normal: &[[f32; 3]],
) -> Result<(String, HashMap<String, TriangleMesh>)> {
    let groups = obj
        .groups
        .iter()
        .map(|group| group_to_mesh(group, position, texture, normal))
        .collect::<Result<HashMap<_, _>>>()?;
    Ok((obj.name.to_string(), groups))
}

pub mod factories {
    use super::*;

    pub fn load_obj_mesh(
        path: impl AsRef<Path>,
    ) -> Result<HashMap<String, HashMap<String, TriangleMesh>>> {
        let obj = Obj::load(path)?;

        let objects = obj
            .data
            .objects
            .iter()
            .map(|o| obj_to_mesh(o, &obj.data.position, &obj.data.texture, &obj.data.normal))
            .collect::<Result<HashMap<_, _>>>()?;
        Ok(objects)
    }
}
