pub trait RenderStatsCollector {
    fn count_ray_cast(&mut self);
    fn count_bounding_box_test(&mut self);
    fn count_sphere_test(&mut self);
    fn count_moving_sphere_test(&mut self);
    fn count_pixel(&mut self);
}

#[derive(Debug, Clone)]
pub struct RenderStats {
    pub rays_cast: usize,
    pub bounding_box_tests: usize,
    pub sphere_tests: usize,
    pub moving_sphere_tests: usize,
    pub pixels: usize,
}

impl RenderStats {
    fn new() -> Self {
        Self {
            rays_cast: 0,
            bounding_box_tests: 0,
            sphere_tests: 0,
            moving_sphere_tests: 0,
            pixels: 0,
        }
    }
}

pub trait RenderStatsSource {
    fn get_stats(&self) -> RenderStats;
}

pub trait RenderStatsAccumulator {
    fn add_stats(&mut self, stats: RenderStats);
}

pub struct TracingStats(RenderStats);

impl TracingStats {
    pub fn new() -> Self {
        Self(RenderStats::new())
    }

    fn get_stats_mut(&mut self) -> &mut RenderStats {
        &mut self.0
    }
}

impl RenderStatsCollector for TracingStats {
    fn count_ray_cast(&mut self) {
        self.get_stats_mut().rays_cast += 1;
    }

    fn count_bounding_box_test(&mut self) {
        self.get_stats_mut().bounding_box_tests += 1;
    }

    fn count_sphere_test(&mut self) {
        self.get_stats_mut().sphere_tests += 1;
    }

    fn count_moving_sphere_test(&mut self) {
        self.get_stats_mut().moving_sphere_tests += 1;
    }

    fn count_pixel(&mut self) {
        self.get_stats_mut().pixels += 1;
    }
}

impl RenderStatsAccumulator for TracingStats {
    fn add_stats(&mut self, stats: RenderStats) {
        let render_stats = self.get_stats_mut();

        *render_stats = RenderStats {
            rays_cast: render_stats.rays_cast + stats.rays_cast,
            bounding_box_tests: render_stats.bounding_box_tests + stats.bounding_box_tests,
            sphere_tests: render_stats.sphere_tests + stats.sphere_tests,
            moving_sphere_tests: render_stats.moving_sphere_tests + stats.moving_sphere_tests,
            pixels: render_stats.pixels + stats.pixels,
        }
    }
}

impl RenderStatsSource for TracingStats {
    fn get_stats(&self) -> RenderStats {
        self.0.clone()
    }
}

impl From<TracingStats> for RenderStats {
    fn from(s: TracingStats) -> Self {
        s.0
    }
}

impl<'a> From<&'a TracingStats> for &'a RenderStats {
    fn from(s: &'a TracingStats) -> Self {
        &s.0
    }
}
