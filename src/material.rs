use std::f64::consts::PI;

use crate::bxdf::BxDF;
use crate::camera::Camera;
use crate::colour::Colour;
use crate::fresnel::{Fresnel, FresnelConductor};
use crate::hittable::HitRecord;
use crate::microfacet::{BeckmannDistribution, Microfacet};
use crate::onb::OrthonormalBasis;
use crate::ray::Ray;
use crate::texture::Texture;
use crate::utils::random_cosine_direction;
use crate::utils::{random_in_unit_sphere, reflect, refract};
use crate::vector::Vec3;

use rand::Rng;

pub trait Material: Send + Sync + std::fmt::Debug {
    fn scatter(
        &self,
        _ray_in: &Ray,
        _hit_record: &HitRecord,
        camera: &Camera,
    ) -> (Ray, Colour, bool) {
        (
            Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)),
            Colour::new(0.0, 0.0, 0.0),
            false,
        )
    }

    fn emitted(&self, _u: f64, _v: f64, _p: &Vec3) -> Colour {
        Colour::new(0.0, 0.0, 0.0)
    }

    fn scattering_pdf(&self, _ray_in: &Ray, _hit_record: &HitRecord, _scattered: &Ray) -> f64 {
        1.0
    }

    fn use_pdfs(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct UnitMaterial {}
impl Material for UnitMaterial {}

#[derive(Debug)]
pub struct MicrofacetReflectance {
    pub albedo: Box<dyn Texture + Send + Sync>,
    pub bxdf: Box<dyn BxDF + Send + Sync>,
}

impl Material for MicrofacetReflectance {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        camera: &Camera,
    ) -> (Ray, Colour, bool) {
        let normal = match self
            .albedo
            .normal_value(hit_record.u, hit_record.v, &hit_record.p)
        {
            Some(normal) => normal,
            None => hit_record.normal,
        };

        let onb_normal = OrthonormalBasis::build_from_w(&hit_record.normal);

        let wo = &camera.origin - &hit_record.p;
        let wi = reflect(&ray_in.direction, &hit_record.normal);

        let onb = OrthonormalBasis::build_from_w(&hit_record.normal);

        let mut scatter_direction = onb.local_vec(&random_cosine_direction());

        if scatter_direction.near_zero() {
            scatter_direction = Vec3::copy(&normal);
        }

        let scattered = Ray::new(Vec3::copy(&hit_record.p), scatter_direction);

        let metallic = 0.0;
        let roughness = 0.45;
        let reflectance = 1.0;

        let mut colour = self.albedo.value(hit_record.u, hit_record.v, &hit_record.p);
        colour.r = colour.r.powf(1.5);
        colour.g = colour.g.powf(1.5);
        colour.b = colour.b.powf(1.5);
        (
            scattered,
            self.bxdf.f(
                &wi,
                &wo,
                &onb_normal.local_vec(&normal),
                //&hit_record.normal,
                &colour,
            ),
            true,
        )
    }

    fn scattering_pdf(&self, _ray_in: &Ray, _hit_record: &HitRecord, _scattered: &Ray) -> f64 {
        1.0
    }

    fn use_pdfs(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct SpecularReflectance {
    pub albedo: Box<dyn Texture + Send + Sync>,
}

impl Material for SpecularReflectance {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        camera: &Camera,
    ) -> (Ray, Colour, bool) {
        let normal = match self
            .albedo
            .normal_value(hit_record.u, hit_record.v, &hit_record.p)
        {
            Some(normal) => normal,
            None => hit_record.normal,
        };

        let onb = OrthonormalBasis::build_from_w(&normal);

        let reflected_world = reflect(&ray_in.direction, &normal);
        //let reflected = onb.local_vec(&reflected_world).unit();
        let wo = camera.origin - hit_record.p;

        let metallic = 0.0;
        let roughness = 0.01;
        let reflectance = 1.0;

        //let fresnel = FresnelConductor::new(0.1, 0.1, 1.1);
        //let brdf = SpecularReflection {
        //    scale: 0.5,
        //    fresnel: Box::new(fresnel),
        //};

        //        let scattered = Ray::new(Vec3::copy(&hit_record.p), reflected_world);
        let scatter_direction = reflected_world;

        let scattered = Ray::new(Vec3::copy(&hit_record.p), scatter_direction);
        let scattered_b = scattered.direction.dot(&normal) > 0.0;
        let mut colour = self.albedo.value(hit_record.u, hit_record.v, &hit_record.p);

        let c = Colour::new(colour.r, colour.g, colour.b);
        (
            scattered, colour, // * brdf.sample_f(&ray_in.direction, &reflected, (0.0, 0.0)),
            //       brdf_microfacet(
            //           &reflected_world,
            //           &wo,
            //           &hit_record.normal,
            //           metallic,
            //           roughness,
            //           reflectance,
            //           &colour,
            //           false,
            //       ),
            true,
        )
    }

    fn scattering_pdf(&self, _ray_in: &Ray, _hit_record: &HitRecord, _scattered: &Ray) -> f64 {
        1.0
    }

    fn use_pdfs(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Lambertian {
    pub albedo: Box<dyn Texture + Send + Sync>,
}

impl Material for Lambertian {
    fn scatter(
        &self,
        _ray_in: &Ray,
        hit_record: &HitRecord,
        camera: &Camera,
    ) -> (Ray, Colour, bool) {
        let normal = match self
            .albedo
            .normal_value(hit_record.u, hit_record.v, &hit_record.p)
        {
            Some(normal) => normal,
            None => hit_record.normal,
        };

        let onb = OrthonormalBasis::build_from_w(&normal);
        let mut scatter_direction = onb.local_vec(&random_cosine_direction());

        if scatter_direction.near_zero() {
            scatter_direction = Vec3::copy(&normal);
        }

        let scattered = Ray::new(Vec3::copy(&hit_record.p), scatter_direction);

        (
            scattered,
            Colour::copy(&self.albedo.value(hit_record.u, hit_record.v, &hit_record.p)),
            true,
        )
    }

    fn scattering_pdf(&self, _ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f64 {
        let normal = match self
            .albedo
            .normal_value(hit_record.u, hit_record.v, &hit_record.p)
        {
            Some(normal) => normal,
            None => hit_record.normal,
        };

        let cosine = normal.dot(&scattered.direction.unit());

        if cosine < 0.0 {
            0.0
        } else {
            cosine / PI
        }
    }

    fn use_pdfs(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Metal {
    pub albedo: Box<dyn Texture + Send + Sync>,
    pub f: f64,
}

impl Material for Metal {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        camera: &Camera,
    ) -> (Ray, Colour, bool) {
        let normal = match self
            .albedo
            .normal_value(hit_record.u, hit_record.v, &hit_record.p)
        {
            Some(normal) => normal,
            None => hit_record.normal,
        };

        let reflected = reflect(&ray_in.direction.unit(), &normal);
        let scattered_ray = Ray::new(
            Vec3::copy(&hit_record.p),
            reflected + self.f.clamp(0.0, 1.0) * &random_in_unit_sphere(),
        );
        let scattered = scattered_ray.direction.dot(&normal) > 0.0;
        (
            scattered_ray,
            self.albedo.value(hit_record.u, hit_record.v, &hit_record.p),
            scattered,
        )
    }
}

#[derive(Debug)]
pub struct Dialectric {
    pub albedo: Option<Box<dyn Texture + Send + Sync>>,
    pub index_of_refraction: f64,
}

impl Material for Dialectric {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        camera: &Camera,
    ) -> (Ray, Colour, bool) {
        let mut normal = hit_record.normal;
        let mut attenuation = Colour::new(1.0, 1.0, 1.0);

        if let Some(albedo) = &self.albedo {
            normal = match albedo.normal_value(hit_record.u, hit_record.v, &hit_record.p) {
                Some(normal) => normal,
                None => hit_record.normal,
            };
            attenuation = albedo.value(hit_record.u, hit_record.v, &hit_record.p);
        }

        let refraction_ratio = if hit_record.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let unit_direction = ray_in.direction.unit();
        let cos_theta = -unit_direction.dot(&normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction = if cannot_refract
            || reflectance(cos_theta, refraction_ratio) > rand::thread_rng().gen_range(0.0..1.0)
        {
            reflect(&unit_direction, &normal)
        } else {
            refract(&unit_direction, &normal, refraction_ratio)
        };

        (
            Ray::new(Vec3::copy(&hit_record.p), direction),
            attenuation,
            true,
        )
    }
}

#[derive(Debug)]
pub struct Light {
    pub intensity: f64,
    pub albedo: Box<dyn Texture + Send + Sync>,
}

impl Material for Light {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        camera: &Camera,
    ) -> (Ray, Colour, bool) {
        if self.albedo.alpha_value(hit_record.u, hit_record.v) < 0.1 {
            let onb = OrthonormalBasis::build_from_w(&hit_record.normal);
            //let mut scatter_direction = &hit_record.normal + random_in_unit_vector();
            let mut scatter_direction = onb.local_vec(&random_cosine_direction());

            if scatter_direction.near_zero() {
                scatter_direction = Vec3::copy(&hit_record.normal);
            }

            let scattered = Ray::new(Vec3::copy(&hit_record.p), scatter_direction);

            return (
                scattered,
                Colour::copy(&self.albedo.value(hit_record.u, hit_record.v, &hit_record.p)),
                true,
            );
        }

        let ray = Ray::new(Vec3::copy(&ray_in.origin), Vec3::copy(&ray_in.direction));
        (ray, Colour::new(0.0, 0.0, 0.0), false)
    }

    fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Colour {
        let alpha_value = self.albedo.alpha_value(u, v);
        if alpha_value > 0.1 {
            return alpha_value.powf(2.0) * self.intensity * self.albedo.value(u, v, p);
        }
        Colour::new(0.0, 0.0, 0.0)
    }
}

#[derive(Debug)]
pub struct Isotropic {
    pub albedo: Box<dyn Texture + Send + Sync>,
}

impl Material for Isotropic {
    fn scatter(
        &self,
        _ray_in: &Ray,
        hit_record: &HitRecord,
        camera: &Camera,
    ) -> (Ray, Colour, bool) {
        let ray = Ray::new(Vec3::copy(&hit_record.p), random_in_unit_sphere());
        (
            ray,
            self.albedo.value(hit_record.u, hit_record.v, &hit_record.p),
            true,
        )
    }
}

fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}
