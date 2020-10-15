use super::{
    factories::*, CompoundPrimitive, IntoPrimitive, TransformablePrimitive, TransformedXyRectangle,
};
use crate::math::*;

pub mod factories {
    use super::*;

    pub fn unit_box() -> CompoundPrimitive<TransformedXyRectangle> {
        let pt_min = Point3::new(-0.5, -0.5, -0.5);
        let pt_max = Point3::new(0.5, 0.5, 0.5);

        vec![
            xy_rectangle((pt_min.x, pt_max.x), (pt_min.y, pt_max.y), pt_max.z),
            xy_rectangle((pt_min.x, pt_max.x), (pt_min.y, pt_max.y), pt_min.z),
            xz_rectangle((pt_min.x, pt_max.x), (pt_min.z, pt_max.z), pt_max.y),
            xz_rectangle((pt_min.x, pt_max.x), (pt_min.z, pt_max.z), pt_min.y),
            yz_rectangle((pt_min.y, pt_max.y), (pt_min.z, pt_max.z), pt_max.x),
            yz_rectangle((pt_min.y, pt_max.y), (pt_min.z, pt_max.z), pt_min.x),
        ]
        .into_primitive()
    }

    pub fn box_shape(pt_min: Point3, pt_max: Point3) -> CompoundPrimitive<TransformedXyRectangle> {
        let x_range = pt_max.x - pt_min.x;
        let x_center = (pt_max.x + pt_min.x) / 2.0;
        let y_range = pt_max.y - pt_min.y;
        let y_center = (pt_max.y + pt_min.y) / 2.0;
        let z_range = pt_max.z - pt_min.z;
        let z_center = (pt_max.z + pt_min.z) / 2.0;

        unit_box()
            .nonuniform_scale(x_range, y_range, z_range)
            .translate(vec3(x_center, y_center, z_center))
    }
}
