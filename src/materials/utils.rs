use crate::math::*;

pub fn reflect(v: Vector3, n: Vector3) -> Vector3 {
    return v - (2.0 * v.dot(n) * n);
}

pub fn refract(v: Vector3, n: Vector3, etai_over_etat: FloatType) -> Vector3 {
    let cos_theta = (-v).dot(n);
    let r_out_perp = etai_over_etat * (v + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.magnitude2()).abs().sqrt() * n;

    r_out_perp + r_out_parallel
}

pub fn schlick(cosine: FloatType, ref_idx: FloatType) -> FloatType {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}
