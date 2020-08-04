#[derive(Debug)]
pub struct TracingStats {
    rays_cast: usize,
    bounding_box_tests: usize,
    sphere_tests: usize,
    moving_sphere_tests: usize,
}

impl TracingStats {
    pub fn new() -> Self {
        Self {
            rays_cast: 0,
            bounding_box_tests: 0,
            sphere_tests: 0,
            moving_sphere_tests: 0,
        }
    }

    pub fn count_ray_cast(&mut self) {
        self.rays_cast += 1;
    }

    pub fn count_bounding_box_test(&mut self) {
        self.bounding_box_tests += 1;
    }

    pub fn count_sphere_test(&mut self) {
        self.sphere_tests += 1;
    }

    pub fn count_moving_sphere_test(&mut self) {
        self.moving_sphere_tests += 1;
    }
}

impl std::ops::Add for TracingStats {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            rays_cast: self.rays_cast + other.rays_cast,
            bounding_box_tests: self.bounding_box_tests + other.bounding_box_tests,
            sphere_tests: self.sphere_tests + other.sphere_tests,
            moving_sphere_tests: self.moving_sphere_tests + other.moving_sphere_tests,
        }
    }
}
