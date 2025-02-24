use rand::Rng;
use std::f64::consts::PI;

use crate::colour::Colour;
use crate::utils::*;
use crate::vector::Vec3;

pub trait BxDF: std::fmt::Debug + Send + Sync {
    fn f(&self, wo: &Vec3, wi: &Vec3, n: &Vec3, colour: &Colour) -> Colour;
    fn sample_wh(&self, wo: &Vec3) -> Vec3;
}

#[derive(Debug)]
pub struct MicrofacetReflection {
    metallic: f64,
    roughness: f64,
    reflectance: f64,
    include_diffuse: bool,
}
impl MicrofacetReflection {
    pub fn new(
        metallic: f64,
        roughness: f64,
        reflectance: f64,
        include_diffuse: bool,
    ) -> MicrofacetReflection {
        MicrofacetReflection {
            metallic,
            roughness,
            reflectance,
            include_diffuse,
        }
    }
}

impl BxDF for MicrofacetReflection {
    fn f(&self, wo: &Vec3, wi: &Vec3, n: &Vec3, colour: &Colour) -> Colour {
        let h = (wo + wi).unit();

        let no_v = n.dot(wo).clamp(0.0, 1.0);
        let no_l = n.dot(wi).clamp(0.0, 1.0);
        let no_h = n.dot(&h).clamp(0.0, 1.0);
        let vo_h = wo.dot(&h).clamp(0.0, 1.0);

        let r = 0.36 * self.reflectance * self.reflectance;
        let f0 = r;

        let f = fresnel_schlik(vo_h, f0);
        let d = d_ggx(no_h, self.roughness);
        let g = g_smith(no_v, no_l, self.roughness);

        let c = (f * d * g) / (4.0 * no_v.max(0.001) * no_l.max(0.001));
        let spec = Colour::new(c, c, c);

        let rho_d = (1.0 - self.metallic) * colour;
        let diff = (1.0 / PI) * rho_d;

        if self.include_diffuse {
            spec + diff
        } else {
            spec + 1.0 * colour
        }
    }

    fn sample_wh(&self, wo: &Vec3) -> Vec3 {
        let u: (f64, f64) = (rand::thread_rng().gen(), rand::thread_rng().gen());
        let mut log_sample = f64::ln(1.0 - u.0);
        if log_sample.is_infinite() {
            log_sample = 0.0;
        }

        let alpha = self.roughness * self.roughness;
        let tan_2_theta = -alpha * alpha * log_sample;
        let phi = u.1 * 2.0 * PI;

        let cos_theta = 1.0 / (1.0 + tan_2_theta).sqrt();
        let sin_theta = f64::sqrt(f64::max(0.0, 1.0 - cos_theta * cos_theta));

        let mut wh = spherical_direction(sin_theta, cos_theta, phi);
        if !same_hemisphere(wo, &wh) {
            wh = -wh;
        }

        wh
    }
}

pub fn fresnel_schlik(cos_theta: f64, f: f64) -> f64 {
    f + (1.0 - f) * f64::powf(1.0 - cos_theta, 5.0)
}

pub fn d_ggx(no_h: f64, roughness: f64) -> f64 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let no_h2 = no_h * no_h;
    let b = no_h2 * (alpha2 - 1.0) + 1.0;
    alpha2 * (1.0 / PI) / (b * b)
}

pub fn g_smith(no_v: f64, no_l: f64, roughness: f64) -> f64 {
    g1_ggx_schlick(no_l, roughness) * g1_ggx_schlick(no_v, roughness)
}

pub fn g1_ggx_schlick(no_v: f64, roughness: f64) -> f64 {
    let alpha = roughness * roughness;
    let k = alpha / 2.0;
    no_v.max(0.001) / (no_v * (1.0 - k) + k)
}
