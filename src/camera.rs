use crate::{math::*, utils::*, Ray};

#[derive(Clone, Debug)]
pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vector3,
    vertical: Vector3,
    u: Vector3,
    v: Vector3,
    lens_radius: FloatType,
}

impl Camera {
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vector3,
        fov: Rad<FloatType>,
        aspect_ratio: FloatType,
        aperture: FloatType,
        focus_dist: FloatType,
    ) -> Self {
        let h = (fov / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        Self {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
            u,
            v,
            lens_radius: aperture / 2.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreparedCamera {
    camera: Camera,
    t0: FloatType,
    t1: FloatType,
}

impl PreparedCamera {
    pub fn make(camera: Camera, t0: FloatType, t1: FloatType) -> Self {
        Self { camera, t0, t1 }
    }

    pub fn make_ray(&self, s: FloatType, t: FloatType) -> Ray {
        let rd = self.camera.lens_radius * random_in_unit_disk();
        let offset = self.camera.u * rd.x + self.camera.v * rd.y;

        let direction =
            self.camera.lower_left_corner + s * self.camera.horizontal + t * self.camera.vertical
                - self.camera.origin
                - offset;
        let time = random_in_range(self.t0, self.t1);
        Ray::new(self.camera.origin + offset, direction.normalize(), time)
    }
}
