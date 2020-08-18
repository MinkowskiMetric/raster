use super::{CoreHittable, HitResult};
use crate::math::*;
use crate::ray_scanner::Ray;
use crate::RenderStatsCollector;
use crate::{BoundingBox, Material};

#[derive(Debug)]
pub struct ParabolaXY<M: Material + Clone>(M);

impl<M: 'static + Material + Clone> Clone for ParabolaXY<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

const pa: FloatType = -(1.0 / 4.0);
const pb: FloatType = 0.0;
const pc: FloatType = 8.0;

const pr: FloatType = 2.0;

impl<M: 'static + Material + Clone> CoreHittable for ParabolaXY<M> {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
        _stats: &mut dyn RenderStatsCollector,
    ) -> Option<HitResult<'a>> {
        // A paraboloid is all the points that are equidistant between the focus of the parabola, and the directrix plane,
        // which is a plane that does not pass through the focus.

        // We should be able to derive the intersections from that.

        // We don't actually want to take the focal point as input, mostly because it is inconvenient for positioning.
        // Instead we take the midpoint of the line between the directrix and the focus, which is the extremum point.
        let extremum_point = Point3::new(0.0, 0.0, 8.0);
        let focus_point = Point3::new(0.0, 0.0, 7.75);
        let orientation_vector = focus_point - extremum_point;
        let directrix_origin = extremum_point - orientation_vector;
        let focus_distance = orientation_vector.magnitude();
        let orientation_vector = orientation_vector / focus_distance;

        let ray_origin = ray.origin.into_point();
        let ray_direction = ray.direction.into_vector();

        // So, we're looking for places where the ray is the same distance from the plane as from the focus.

        // Shorthand:
        // - V: the orientation_vector, which is the unit vector specifying which way the paraboloid points,
        // - P: the intersection point - this is what we're solving for, and there are two of them.
        // - k: The length of the vector between the extremum point and the focus, or between the extremum point and the directrix plane
        // - F: the focus point of the paraboloid
        // - A: a point determined by casting P onto the directrix plane
        // - O: the ray origin
        // - D: the ray direction
        // - X: (O - C) - this is a useful vector to have. Effectively this simplifies some of the maths because it puts the extremum at 0,0,0

        // So, any point is on the directrix plane if (A-(C-V*k))|V = 0 since V is also the normal of the plane, or A-C-V*k

        // P = A + V * r, where r is the distance between the plane and the point
        // A = P - V * r

        // Define point B which is the point on the plane behind the extremum as B = C - V*k
        // Define point F which is the focus point as F = C + V*k

        // So, (A-B|V) = 0, substitute for P
        // (P - V*r - B)|V = 0
        // P|V - V|V*r - B|V = 0
        // P|V - B|V = r

        // We also know that len(P - F) = r

        // len(P - F) = P|V - B|V

        // The equation of the ray is P = O + D*t

        // len(O + D*t - F) = (O + D*t)|V - B|V
        // len(D*t + O - F) = D|V*t + O|V - B|V
        // len(D*t + O - F) = D|V*t + (O - B)|V
        // square both sides
        // dot(D*t + O - F) = (D|V*t + (O - B)|V)^2
        // Expand the left first
        // (D*t + (O - F))|(D*t + (O - F))
        // D|D*t^2 + 2 * D|(O - F) * t + (O-F)|(O-F)
        // Then the right
        // So, if (A-C-V*k)|V = 0, substitue for P, (P - V*r - C - V*k)|V = 0
        // Simplify - (P-C-V*k)|V - V|V*r = 0, and since V is a unit vector, V|V*r is r
        // So (P-C-V*k)|V = r

        // But we also know that the distance to the focus, given by P-C+V*k is also r.

        // So len(P-C+V*k) = (P-C-V*k)|V

        let oc = ray_origin - extremum_point; // Called X above
        let oc_dot_oc = oc.dot(oc);
        let oc_dot_n = oc.dot(orientation_vector);
        let dir_dot_oc = ray_direction.dot(oc);
        let dir_dot_n = ray_direction.dot(orientation_vector);
        let dir_dot_dir = ray_direction.dot(ray_direction);

        if oc_dot_n > 2.0 {
            return None;
        }

        let a = dir_dot_dir - (dir_dot_n * dir_dot_n);
        let b = 2.0 * (dir_dot_oc - (dir_dot_n * (oc_dot_n + (2.0 * focus_distance))));
        let c = oc_dot_oc - (oc_dot_n * (oc_dot_n + (4.0 * focus_distance)));

        let t = if a == 0.0 {
            // Not a quadratic. This can happen when the ray and the shape normal
            // are parallel. In this case 0 = b*t + c, so t = -c/b
            -c / b
        } else {
            // Solve for that
            let intval = (b * b) - (4.0 * a * c);
            if intval < 0.0 {
                // No intersection
                return None;
            }

            let t0 = (-b - intval.sqrt().copysign(a)) / (2.0 * a);
            let t1 = (-b + intval.sqrt().copysign(a)) / (2.0 * a);
            debug_assert!(t0 <= t1);

            if t0 < t_min {
                t1
            } else {
                t0
            }
        };

        if t < t_min || t > t_max {
            // While the infinite ray does intersect, it doesn't do it within it's extent
            return None;
        }

        let hit_point = ray_origin + t * ray_direction;
        let outward_normal = (focus_point - hit_point).normalize();
        let front_face = ray_direction.dot(outward_normal) < 0.0;
        let surface_normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        // How do I get the tangent?
        let tangent = vec3(0.0, 1.0, 0.0);
        let bitangent = surface_normal.cross(tangent);
        let (u, v) = (0.0, 0.0); // TODOTODOTODO

        Some(HitResult {
            distance: t,
            hit_point: hit_point.into(),
            surface_normal: surface_normal.into(),
            tangent,
            bitangent,
            front_face,
            material: &self.0,
            u,
            v,
        })
    }

    fn bounding_box(&self, _t0: FloatType, _t1: FloatType) -> BoundingBox {
        // For now we'll make this quite big
        BoundingBox::new(
            Point3::new(-1000.0, -1000.0, -1000.0),
            Point3::new(1000.0, 1000.0, 1000.0),
        )
    }
}

pub mod factories {
    use super::*;

    pub fn parabola<M: Material + Clone>(material: M) -> ParabolaXY<M> {
        ParabolaXY(material)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::factories::*;

    #[test]
    fn test_parabola_intersection() {
        let p = parabola(dielectric(1.5));
        let mut stats = crate::TracingStats::new();

        let hr = p
            .intersect(
                &Ray::new(Point3::new(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), 0.0),
                0.001,
                constants::INFINITY,
                &mut stats,
            )
            .unwrap();
        assert_eq!(hr.hit_point, Point3::new(0.0, 0.0, 8.0));
    }

    /*const EPSILON: FloatType = 1.0;
    static mut max_error: FloatType = 0.0;

    fn check_error(error: FloatType, msg: String) {
        unsafe { max_error = max_error.max(error);
        println!("MAX ERROR: {}", max_error);}


        assert!(error < EPSILON, msg);
    }

    fn test_parabola_hit_point<T: 'static + Material + Clone>(p: &ParabolaXY<T>, ray_origin: Point3, ray_direction: Vector3, hit_point: Option<Point3>) {
        let ray_direction = ray_direction.normalize();
        let r = Ray::new(ray_origin, ray_direction, 0.0);

        let mut stats = crate::TracingStats::new();
        println!("Sending ray from {:?} with direction {:?}", ray_origin, ray_direction);
        match (hit_point, p.intersect(&r, 0.001, constants::INFINITY, &mut stats)) {
            (Some(hit_point), Some(hit_result)) => {
                check_error((hit_point.x - hit_result.hit_point.x).abs(), format!("Expected ray from {:?} in direction {:?} to hit at {:?}, actually hit at {:?} (X ERROR: {})", ray_origin, ray_direction, hit_point, hit_result.hit_point, (hit_point.x - hit_result.hit_point.x).abs()));
                check_error((hit_point.y - hit_result.hit_point.y).abs(), format!("Expected ray from {:?} in direction {:?} to hit at {:?}, actually hit at {:?} (Y ERROR: {})", ray_origin, ray_direction, hit_point, hit_result.hit_point, (hit_point.y - hit_result.hit_point.y).abs()));
                check_error((hit_point.z - hit_result.hit_point.z).abs(), format!("Expected ray from {:?} in direction {:?} to hit at {:?}, actually hit at {:?} (Z ERROR: {})", ray_origin, ray_direction, hit_point, hit_result.hit_point, (hit_point.z - hit_result.hit_point.z).abs()));
            }

            (None, None) => (),         // this is fine - we didn't expect a hit and there wasn't one

            (None, Some(hit_result)) => panic!("Expected ray from {:?} in direction {:?} to miss, actually hit at {:?}", ray_origin, ray_direction, hit_result.hit_point),
            (Some(hit_point), None) => panic!("Expected ray from {:?} in direction {:?} to hit at {:?}, actually missed", ray_origin, ray_direction, hit_point),
        }
    }

    #[test]
    fn test_parabola_intersection_with_constant_x_and_y() {
        let p = parabola(dielectric(1.5));

        for degrees in 0..360 {
            let angle: Rad<_> = Deg(degrees as FloatType).into();

            for r in -100..=100 {
                let r = 8.0 * (r as FloatType) / 100.0;

                println!("Hitting origin from angle of {} with radius of {} and constant x and y", degrees, r);
                let ray_origin = Point3::new(r * angle.cos(), r * angle.sin(), 0.0);
                let ray_target = Point3::new(r * angle.cos(), r * angle.sin(), pa * r * r + pc);
                let ray_direction = (ray_target - ray_origin).normalize();

                let computed_radius = (ray_target.x.powf(2.0) + ray_target.y.powf(2.0)).sqrt();
                if (computed_radius - pr).abs() < 0.0001 {
                    println!("skipping as too close to edge");
                    continue;
                }

                let expected = if computed_radius > pr { None } else { Some(ray_target) };
                test_parabola_hit_point(&p, ray_origin, ray_direction, expected);
            }
        }
    }

    #[test]
    fn test_parabola_intersection_with_constant_x() {
        let p = parabola(dielectric(1.5));

        for degrees in 0..360 {
            let angle: Rad<_> = Deg(degrees as FloatType).into();

            for r in -100..=100 {
                let r = 8.0 * (r as FloatType) / 100.0;
                let target_x = r * angle.cos();
                let target_y: FloatType = 0.0;
                let target_r = (target_x.powf(2.0) + target_y.powf(2.0)).sqrt();
                if (target_r - pr).abs() < 0.0001 {
                    continue;
                }
                let target_z = pa * target_r * target_r + pc;

                let ray_origin = Point3::new(r * angle.cos(), r * angle.sin(), 0.0);
                let ray_target = Point3::new(target_x, target_y, target_z);
                let ray_direction = (ray_target - ray_origin).normalize();
                let expected = if target_r > pr { None } else { Some(ray_target) };
                println!("Hitting {:?} from angle of {} with radius of {} and constant x", ray_target, degrees, r);
                test_parabola_hit_point(&p, ray_origin, ray_direction, expected);
            }
        }
    }

    #[test]
    fn test_parabola_intersection() {
        let p = parabola(dielectric(1.5));

        for degrees in 0..360 {
            let angle: Rad<_> = Deg(degrees as FloatType).into();

            for r in -100..=100 {
                let r = 8.0 * (r as FloatType) / 100.0;
                let target_x = -r * angle.cos();
                let target_y = -r * angle.sin();
                let target_r = (target_x.powf(2.0) + target_y.powf(2.0)).sqrt();
                if (target_r - pr).abs() < 0.0001 {
                    continue;
                }
                let target_z = pa * target_r * target_r + pc;

                let ray_origin = Point3::new(r * angle.cos(), r * angle.sin(), 0.0);
                let ray_target = Point3::new(target_x, target_y, target_z);
                let ray_direction = (ray_target - ray_origin).normalize();
                let expected = if target_r > pr { None } else { Some(ray_target) };
                println!("Hitting {:?} from angle of {} with radius of {} and constant x", ray_target, degrees, r);
                test_parabola_hit_point(&p, ray_origin, ray_direction, expected);
            }
        }
    }*/
}
