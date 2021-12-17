use crate::{BaseMaterial, Color, IntersectResult, Ray};

use super::material::BaseMaterialScatterResult;

#[derive(Clone, Debug)]
pub enum DebugMaterial {
    Uv,
    Normal,
    Tangent,
    Bitangent,
}

impl BaseMaterial for DebugMaterial {
    fn base_scatter(
        &self,
        _ray_in: &Ray,
        hit_record: &dyn IntersectResult,
    ) -> BaseMaterialScatterResult {
        let color = match self {
            Self::Uv => {
                let uv = hit_record.uv();
                Color([uv.x, uv.y, 0.0, 0.0])
            }

            Self::Normal => {
                let normal = hit_record.surface_normal();
                Color([
                    (normal.x + 1.0) / 2.0,
                    (normal.y + 1.0) / 2.0,
                    (normal.z + 1.0) / 2.0,
                    0.0,
                ])
            }

            Self::Tangent => {
                let tangent = hit_record.tangent();
                Color([
                    (tangent.x + 1.0) / 2.0,
                    (tangent.y + 1.0) / 2.0,
                    (tangent.z + 1.0) / 2.0,
                    0.0,
                ])
            }

            Self::Bitangent => {
                let bitangent = hit_record.bitangent();
                Color([
                    (bitangent.x + 1.0) / 2.0,
                    (bitangent.y + 1.0) / 2.0,
                    (bitangent.z + 1.0) / 2.0,
                    0.0,
                ])
            }
        };

        BaseMaterialScatterResult {
            emitted: color,
            scatter: None,
        }
    }
}

pub mod factories {
    use super::*;

    pub fn uv_material() -> DebugMaterial {
        DebugMaterial::Uv
    }

    pub fn normal_material() -> DebugMaterial {
        DebugMaterial::Normal
    }

    pub fn tangent_material() -> DebugMaterial {
        DebugMaterial::Tangent
    }

    pub fn bitangent_material() -> DebugMaterial {
        DebugMaterial::Bitangent
    }
}
