mod camera;
mod color;
mod fixed_size_stack;
mod materials;
mod perlin;
mod ray_scanner;
mod scene;
mod shapes;
mod sky;
mod stats;
mod textures;

pub mod math;
pub mod utils;

pub use camera::Camera;
pub use color::Color;
pub use materials::{Material, PartialScatterResult, ScatterResult, SurfaceMapper};
pub use ray_scanner::{scan, Ray};
pub use scene::Scene;
pub use shapes::{
    BoundingBox, CollectionShape, CompoundPrimitive, CompoundShape, HitResult, IntoPrimitive,
    MediumDensity, Primitive, PrimitiveHitResult, Shape, ShapeList, SkinnablePrimitive, Sphere,
    TransformablePrimitive, TransformableShape, TransformedXyRectangle,
};
pub use sky::Sky;
pub use stats::{
    RenderStats, RenderStatsAccumulator, RenderStatsCollector, RenderStatsSource, TracingStats,
};
pub use textures::Texture;

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
