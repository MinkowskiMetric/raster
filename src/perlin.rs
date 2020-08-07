use crate::math::*;
use crate::utils::*;

const POINT_COUNT: usize = 256;

#[derive(Clone)]
pub struct Perlin {
    ranfloat: [FloatType; POINT_COUNT],
    perm_x: [i32; POINT_COUNT],
    perm_y: [i32; POINT_COUNT],
    perm_z: [i32; POINT_COUNT],
}

impl Perlin {
    pub fn new() -> Self {
        let mut ranfloat = [0.0; POINT_COUNT];
        for f in 0..POINT_COUNT {
            ranfloat[f] = random_in_range(0.0, 1.0);
        }

        Self {
            ranfloat,
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

    fn generate_perm() -> [i32; POINT_COUNT] {
        let mut ret = [0; POINT_COUNT];

        for i in 0..POINT_COUNT {
            ret[i] = i as i32;
        }

        Self::permute(&mut ret);

        ret
    }

    fn permute(sl: &mut [i32]) {
        for i in (1..sl.len()).rev() {
            let target = random_int_in_range(0, i as i32) as usize;
            sl.swap(target, i);
        }
    }

    pub fn noise(&self, p: Point3) -> FloatType {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = (((4.0 * p.x) as i32) & 255) as usize;
        let j = (((4.0 * p.y) as i32) & 255) as usize;
        let k = (((4.0 * p.z) as i32) & 255) as usize;
        let idx = (self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]) as usize;

        self.ranfloat[idx]
    }
}

impl std::fmt::Debug for Perlin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Perlin")
         .finish()
    }
}
/*class perlin {
    public:
        perlin() {
            ranfloat = new double[point_count];
            for (int i = 0; i < point_count; ++i) {
                ranfloat[i] = random_double();
            }

            perm_x = perlin_generate_perm();
            perm_y = perlin_generate_perm();
            perm_z = perlin_generate_perm();
        }

        ~perlin() {
            delete[] ranfloat;
            delete[] perm_x;
            delete[] perm_y;
            delete[] perm_z;
        }

        double noise(const point3& p) const {
            auto u = p.x() - floor(p.x());
            auto v = p.y() - floor(p.y());
            auto w = p.z() - floor(p.z());

            auto i = static_cast<int>(4*p.x()) & 255;
            auto j = static_cast<int>(4*p.y()) & 255;
            auto k = static_cast<int>(4*p.z()) & 255;

            return ranfloat[perm_x[i] ^ perm_y[j] ^ perm_z[k]];
        }

    private:
        static const int point_count = 256;
        double* ranfloat;
        int* perm_x;
        int* perm_y;
        int* perm_z;

        static int* perlin_generate_perm() {
            auto p = new int[point_count];

            for (int i = 0; i < perlin::point_count; i++)
                p[i] = i;

            permute(p, point_count);

            return p;
        }

        static void permute(int* p, int n) {
            for (int i = n-1; i > 0; i--) {
                int target = random_int(0, i);
                int tmp = p[i];
                p[i] = p[target];
                p[target] = tmp;
            }
        }
};*/