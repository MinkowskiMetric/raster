#![feature(maybe_uninit_extra, maybe_uninit_uninit_array)]
#![feature(portable_simd)]

mod bounded;
mod bounding_box;
mod camera;
mod color;
#[macro_use]
mod compound;
mod fixed_size_stack;
mod hit_result;
mod intersectable;
mod kdtree;
mod materials;
mod perlin;
mod ray;
mod ray_scanner;
mod scene;
mod shapes;
mod skinnable;
mod sky;
mod stats;
mod textures;
mod transform;

pub mod math;
pub mod utils;

pub use bounded::{
    Bounded, BoundedIteratorOps, TimeDependentBounded, TimeDependentBoundedIteratorOps,
};
pub use bounding_box::{BoundingBox, BoundingBoxIntersectionTester};
pub use camera::{Camera, PreparedCamera};
pub use color::Color;
pub use compound::{
    CompoundPrimitive, CompoundVisible, DefaultPrimitive, DefaultVisible, DynPrimitive, DynVisible,
    Primitive, SharedPrimitive, Visible,
};
pub use hit_result::{
    GeometryHitResult, IntersectResult, IntersectResultIteratorOps, SkinnedHitResult,
    WrappedIntersectResult,
};
pub use intersectable::{Intersectable, IntersectableIteratorOps};
pub use kdtree::KDTree;
pub use materials::{BaseMaterial, Material, PartialScatterResult, ScatterResult, SurfaceMapper};
pub use ray::Ray;
pub use ray_scanner::scan;
pub use scene::Scene;
pub use shapes::{MediumDensity, Sphere, TriangleVertex};
pub use skinnable::{DefaultSkinnable, Skinnable};
pub use sky::Sky;
pub use stats::{
    RenderStats, RenderStatsAccumulator, RenderStatsCollector, RenderStatsSource, TracingStats,
};
pub use textures::Texture;
pub use transform::{DefaultTransformable, Transformable};

pub mod constants {
    use super::*;

    pub use color::constants::*;
    pub use math::constants::*;
}

pub mod noise {
    use super::*;

    pub use perlin::Perlin;
}

pub mod factories {
    use super::*;

    pub use materials::factories::*;
    pub use shapes::factories::*;
    pub use sky::factories::*;
    pub use textures::factories::*;
}

pub mod prelude {
    use super::*;

    pub use factories::*;
    pub use math::*;
    pub use utils::*;

    pub use super::constants;
}

pub mod macro_helpers {
    pub use super::compound::macro_helpers::*;
}
