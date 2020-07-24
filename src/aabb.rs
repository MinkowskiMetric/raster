use crate::math::*;
use crate::ray_scanner::Ray;

#[derive(Clone, Debug)]
pub struct BoundingBox {
    pt_min: Point3,
    pt_max: Point3,
}

fn test_axis(
    pt_min: FloatType,
    pt_max: FloatType,
    ray_origin: FloatType,
    ray_direction: FloatType,
    t_min: &mut FloatType,
    t_max: &mut FloatType,
) -> bool {
    let inverse_direction = 1.0 / ray_direction;
    let mut t0 = (pt_min - ray_origin) * inverse_direction;
    let mut t1 = (pt_max - ray_origin) * inverse_direction;

    if inverse_direction < 0.0 {
        std::mem::swap(&mut t0, &mut t1);
    }

    *t_min = t0.max(*t_min);
    *t_max = t1.min(*t_max);

    !(t_max <= t_min)
}

impl BoundingBox {
    pub fn new(pt1: Point3, pt2: Point3) -> Self {
        BoundingBox {
            pt_min: Point3::new(pt1.x.min(pt2.x), pt1.y.min(pt2.y), pt1.z.min(pt2.z)),
            pt_max: Point3::new(pt1.x.max(pt2.x), pt1.y.max(pt2.y), pt1.z.max(pt2.z)),
        }
    }

    pub fn empty_box() -> Self {
        let zero_point = Point3::new(0.0, 0.0, 0.0);
        BoundingBox {
            pt_min: zero_point,
            pt_max: zero_point,
        }
    }

    pub fn surrounding_box(box0: &BoundingBox, box1: &BoundingBox) -> Self {
        let pt_min = Point3::new(
            box0.min_point().x.min(box1.min_point().x),
            box0.min_point().y.min(box1.min_point().y),
            box0.min_point().z.min(box1.min_point().z),
        );
        let pt_max = Point3::new(
            box0.max_point().x.max(box1.max_point().x),
            box0.max_point().y.max(box1.max_point().y),
            box0.max_point().z.max(box1.max_point().z),
        );
        Self::new(pt_min, pt_max)
    }

    pub fn min_point(&self) -> &Point3 {
        &self.pt_min
    }

    pub fn max_point(&self) -> &Point3 {
        &self.pt_max
    }

    pub fn intersect(&self, ray: &Ray, mut t_min: FloatType, mut t_max: FloatType) -> bool {
        test_axis(
            self.pt_min.x,
            self.pt_max.x,
            ray.origin.x,
            ray.direction.x,
            &mut t_min,
            &mut t_max,
        ) && test_axis(
            self.pt_min.y,
            self.pt_max.y,
            ray.origin.y,
            ray.direction.y,
            &mut t_min,
            &mut t_max,
        ) && test_axis(
            self.pt_min.z,
            self.pt_max.z,
            ray.origin.z,
            ray.direction.z,
            &mut t_min,
            &mut t_max,
        )
    }
}
