use crate::math::*;
use crate::utils::*;

const POINT_COUNT: usize = 256;

#[derive(Clone)]
pub struct Perlin {
    ranvec: Box<[Vector3]>,
    perm_x: [i32; POINT_COUNT],
    perm_y: [i32; POINT_COUNT],
    perm_z: [i32; POINT_COUNT],
}

impl Perlin {
    pub fn new() -> Self {
        let ranvec = (0..POINT_COUNT)
            .into_iter()
            .map(|_| random_unit_vector())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            ranvec,
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

    fn generate_perm() -> [i32; POINT_COUNT] {
        let mut ret = [0; POINT_COUNT];

        for (i, ret_i) in ret.iter_mut().enumerate() {
            *ret_i = i as i32;
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
        let uu = p.x - p.x.floor();
        let vv = p.y - p.y.floor();
        let ww = p.z - p.z.floor();

        let ii = p.x.floor() as i32;
        let jj = p.y.floor() as i32;
        let kk = p.z.floor() as i32;

        let mut c = [[[Vector3::new(0.0, 0.0, 0.0); 2]; 2]; 2];

        for di in 0i32..2i32 {
            for dj in 0i32..2i32 {
                for dk in 0i32..2i32 {
                    c[di as usize][dj as usize][dk as usize] = self.ranvec[(self.perm_x
                        [((ii + di) & 255) as usize]
                        ^ self.perm_y[((jj + dj) & 255) as usize]
                        ^ self.perm_z[((kk + dk) & 255) as usize])
                        as usize];
                }
            }
        }

        Self::perlin_interp(&c, uu, vv, ww)
    }

    pub fn turbulence(&self, p: Point3, depth: usize) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }

    fn perlin_interp(c: &[[[Vector3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        for (i, i_item) in c.iter().enumerate() {
            let fi = i as f64;
            for (j, j_item) in i_item.iter().enumerate() {
                let fj = j as f64;
                for (k, k_item) in j_item.iter().enumerate() {
                    let fk = k as f64;
                    let weight = Vector3::new(u - fi, v - fj, w - fk);

                    accum += ((fi * uu) + ((1.0 - fi) * (1.0 - uu)))
                        * ((fj * vv) + ((1.0 - fj) * (1.0 - vv)))
                        * ((fk * ww) + ((1.0 - fk) * (1.0 - ww)))
                        * k_item.dot(weight);
                }
            }
        }
        accum
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Perlin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Perlin").finish()
    }
}
