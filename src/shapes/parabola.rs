use crate::{
    math::*, Bounded, BoundingBox, DefaultPrimitive, DefaultSkinnable, DefaultTransformable,
    GeometryHitResult, Intersectable, Ray,
};

#[derive(Debug, Clone)]
pub struct ParabolaXY {
    extremum_point: Point3,
    focus_point: Point3,
    pr: FloatType,
}

impl Intersectable for ParabolaXY {
    type Result = GeometryHitResult;

    fn intersect(
        &self,
        ray: &Ray,
        t_min: FloatType,
        t_max: FloatType,
    ) -> Option<GeometryHitResult> {
        // A paraboloid is all the points that are equidistant between the focus of the parabola, and the directrix plane,
        // which is a plane that does not pass through the focus.

        // We should be able to derive the intersections from that.

        // We don't actually want to take the focal point as input, mostly because it is inconvenient for positioning.
        // Instead we take the midpoint of the line between the directrix and the focus, which is the extremum point.
        let orientation_vector = self.focus_point - self.extremum_point;
        let focus_distance = orientation_vector.magnitude();
        let orientation_vector = orientation_vector / focus_distance;

        let ray_origin = ray.origin;
        let ray_direction = ray.direction;

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

        let oc = ray_origin - self.extremum_point; // Called X above
        let oc_dot_oc = oc.dot(oc);
        let oc_dot_n = oc.dot(orientation_vector);
        let dir_dot_oc = ray_direction.dot(oc);
        let dir_dot_n = ray_direction.dot(orientation_vector);
        let dir_dot_dir = ray_direction.dot(ray_direction);

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

        // Project the hit point onto the axis of the paraboloid
        let center_to_hit_point = hit_point - self.extremum_point;
        let axial_point = orientation_vector * center_to_hit_point.dot(orientation_vector);
        let radial_point = center_to_hit_point - axial_point;
        // From that we can check the radius
        if radial_point.magnitude() > self.pr {
            return None;
        }

        let outward_normal = (self.focus_point - hit_point).normalize();
        let front_face = ray_direction.dot(outward_normal) < 0.0;
        let surface_normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        // We can work out the tangent from the bitangent, and we can get the bitangent from the axial vector and the normal
        // because it is at right angles to them
        let tangent = (-radial_point).cross(outward_normal);
        let bitangent = outward_normal.cross(tangent);

        let uv = point2(radial_point.magnitude() / self.pr, 0.0); // TODOTODOTODO - could use the angle

        Some(GeometryHitResult::new(
            ray,
            t,
            surface_normal,
            tangent,
            bitangent,
            front_face,
            uv,
        ))
    }
}

impl Bounded for ParabolaXY {
    fn bounding_box(&self) -> BoundingBox {
        // For now we'll make this quite big
        BoundingBox::new(
            Point3::new(-1000.0, -1000.0, -1000.0),
            Point3::new(1000.0, 1000.0, 1000.0),
        )
    }
}

impl DefaultTransformable for ParabolaXY {}
impl DefaultSkinnable for ParabolaXY {}
impl DefaultPrimitive for ParabolaXY {}

pub mod factories {
    use super::*;

    pub fn parabola(extremum_point: Point3, focus_point: Point3, radius: FloatType) -> ParabolaXY {
        ParabolaXY {
            extremum_point,
            focus_point,
            pr: radius,
        }
    }
}
