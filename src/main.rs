extern crate clap;
use clap::{App, Arg};

use std::convert::TryInto;

use image::RgbImage;

use raster::{
    prelude::*, shapes, Color, CompoundPrimitive, IntoPrimitive, RenderStatsSource, ShapeList,
    SkinnablePrimitive, Sphere, Texture, TransformablePrimitive, TransformableShape,
    TransformedXyRectangle,
};

use std::sync::{Arc, RwLock};

fn attenuate_color(color: Color, attenuation: FloatType) -> Color {
    color.attenuate(attenuation)
}

fn random_scene(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let mut shapes = ShapeList::build();

    shapes.push(
        sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0).apply_material(lambertian(checker_texture(
            solid_texture(vec3(0.2, 0.3, 0.1).try_into().unwrap()),
            solid_texture(vec3(0.9, 0.9, 0.9).try_into().unwrap()),
        ))),
    );

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_in_range(0.0, 1.0);
            let center = Point3::new(
                (a as FloatType) + 0.9 * random_in_range(0.0, 1.0),
                0.2,
                (b as FloatType) + 0.9 * random_in_range(0.0, 1.0),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).magnitude() > 0.9 {
                if choose_mat < 0.8 {
                    let center2 = center + vec3(0.0, random_in_range(0.0, 0.5), 0.0);
                    let material = lambertian(solid_texture(random_color_in_range(0.0, 1.0)));
                    shapes.push(
                        moving_sphere((center, 0.0), (center2, 1.0), 0.2).apply_material(material),
                    );
                } else if choose_mat < 0.95 {
                    let albedo = random_color_in_range(0.5, 1.0);
                    let fuzz = random_in_range(0.0, 1.0);
                    let material = metal(albedo, fuzz);
                    shapes.push(sphere(center, 0.2).apply_material(material))
                } else {
                    let material = dielectric(1.5);
                    shapes.push(sphere(center, 0.2).apply_material(material))
                }
            }
        }
    }

    shapes.push(sphere(Point3::new(0.0, 1.0, 0.0), 1.0).apply_material(dielectric(1.5)));

    shapes.push(
        sphere(Point3::new(-4.0, 1.0, 0.0), 1.0).apply_material(lambertian(solid_texture(
            vec3(0.4, 0.2, 0.1).try_into().unwrap(),
        ))),
    );

    shapes.push(
        sphere(Point3::new(3.0, 1.0, 0.0), 1.0)
            .apply_material(metal(vec3(0.7, 0.6, 0.5).try_into().unwrap(), 0.0)),
    );

    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    (camera, regular_sky(), shapes)
}

fn my_test_scene(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(-5.0, 2.0, 1.0);
    let lookat = Point3::new(0.0, 0.0, -3.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.1;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(60.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes = shapes![
        sphere(Point3::new(-0.5, 0.0, -3.0), 1.0).apply_material(dielectric(1.5)),
        invert_normal(sphere(Point3::new(-0.5, 0.0, -3.0), 0.999).apply_material(dielectric(1.5))),
        sphere(Point3::new(0.5, 0.0, -5.0), 1.0)
            .apply_material(metal(attenuate_color(constants::MAGENTA, 0.8), 0.2),),
        sphere(Point3::new(-0.5, 0.0, -5.0), 1.0)
            .apply_material(metal(attenuate_color(constants::WHITE, 0.8), 0.0),),
        sphere(Point3::new(0.0, -51.0, -5.0), 50.0).apply_material(lambertian(solid_texture(
            attenuate_color(constants::YELLOW, 0.5)
        )),),
    ];
    (camera, regular_sky(), shapes)
}

fn two_spheres(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes = shapes![
        sphere(Point3::new(0.0, -10.0, 0.0), 10.0).apply_material(lambertian(checker_texture(
            solid_texture(vec3(0.2, 0.3, 0.1).try_into().unwrap()),
            solid_texture(vec3(0.9, 0.9, 0.9).try_into().unwrap()),
        )),),
        sphere(Point3::new(0.0, 10.0, 0.0), 10.0).apply_material(lambertian(checker_texture(
            solid_texture(vec3(0.2, 0.3, 0.1).try_into().unwrap()),
            solid_texture(vec3(0.9, 0.9, 0.9).try_into().unwrap()),
        )),),
    ];
    (camera, regular_sky(), shapes)
}

fn two_perlin_spheres(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let pertext = noise_texture(4.0);

    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes = shapes![
        sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0).apply_material(lambertian(pertext.clone()),),
        sphere(Point3::new(0.0, 2.0, 0.0), 2.0).apply_material(lambertian(pertext.clone())),
    ];

    (camera, regular_sky(), shapes)
}

fn earth_map() -> impl Texture + Clone {
    let earth_bytes = include_bytes!("earthmap.jpg");
    let earth_image = image::load_from_memory(earth_bytes).unwrap();
    image_texture(earth_image)
}

fn brick_image() -> impl Texture + Clone {
    let brick_bytes = include_bytes!("brickwall.jpg");
    let brick_image = image::load_from_memory(brick_bytes).unwrap();
    image_texture(brick_image)
}

fn brick_normal_map() -> impl Texture + Clone {
    let brick_bytes = include_bytes!("brickwall_normal.jpg");
    let brick_image = image::load_from_memory(brick_bytes).unwrap();
    image_texture(brick_image)
}

fn textured_earth(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let earth_image = earth_map();

    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes =
        shapes![sphere(Point3::new(0.0, 0.0, 0.0), 2.0).apply_material(lambertian(earth_image),)];

    (camera, regular_sky(), shapes)
}

fn simple_light(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let pertext = noise_texture(4.0);

    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(26.0, 3.0, 6.0);
    let lookat = Point3::new(0.0, 2.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );
    let shapes = shapes![
        sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0).apply_material(lambertian(pertext.clone()),),
        sphere(Point3::new(0.0, 2.0, 0.0), 2.0).apply_material(lambertian(pertext.clone())),
        sphere(Point3::new(0.0, 7.0, 0.0), 2.0)
            .apply_material(diffuse_light(solid_texture(Color([4.0, 4.0, 4.0, 1.0]))),),
        xy_rectangle((3.0, 5.0), (1.0, 3.0), -2.0)
            .apply_material(diffuse_light(solid_texture(Color([4.0, 4.0, 4.0, 1.0]))),),
        yz_rectangle((1.0, 3.0), (3.0, 4.0), -2.0)
            .apply_material(diffuse_light(solid_texture(Color([4.0, 4.0, 4.0, 1.0]))),),
    ];

    (camera, black_sky(), shapes)
}

fn cornell_box(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(278.0, 278.0, -800.0);
    let lookat = Point3::new(278.0, 278.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(40.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let red = lambertian(solid_texture(Color([0.65, 0.05, 0.05, 1.0])));
    let white = lambertian(solid_texture(Color([0.73, 0.73, 0.73, 1.0])));
    let green = lambertian(solid_texture(Color([0.12, 0.45, 0.15, 1.0])));
    let light = diffuse_light(solid_texture(Color([15.0, 15.0, 15.0, 1.0])));
    let unit_cube = || {
        box_shape(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0))
            .apply_material(white.clone())
    };

    let shapes = shapes![
        yz_rectangle((0.0, 555.0), (0.0, 555.0), 555.0).apply_material(green.clone()),
        yz_rectangle((0.0, 555.0), (0.0, 555.0), 0.0).apply_material(red.clone()),
        xz_rectangle((213.0, 343.0), (227.0, 332.0), 554.0).apply_material(light.clone()),
        xz_rectangle((0.0, 555.0), (0.0, 555.0), 0.0).apply_material(white.clone()),
        xz_rectangle((0.0, 555.0), (0.0, 555.0), 555.0).apply_material(white.clone()),
        xy_rectangle((0.0, 555.0), (0.0, 555.0), 555.0).apply_material(white.clone()),
        unit_cube()
            .nonuniform_scale(165.0, 330.0, 160.0)
            .rotate_y(Deg(15.0).into())
            .translate(vec3(265.0, 0.0, 295.0)),
        unit_cube()
            .nonuniform_scale(165.0, 165.0, 165.0)
            .rotate_y(Deg(-18.0).into())
            .translate(vec3(130.0, 0.0, 65.0)),
    ];

    let shapes = shapes![shapes.nonuniform_scale(1.0, 1.0, 1.0)];

    (camera, black_sky(), shapes)
}

fn cornell_smoke(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(278.0, 278.0, -800.0);
    let lookat = Point3::new(278.0, 278.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(40.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let red = lambertian(solid_texture(Color([0.65, 0.05, 0.05, 1.0])));
    let white = lambertian(solid_texture(Color([0.73, 0.73, 0.73, 1.0])));
    let green = lambertian(solid_texture(Color([0.12, 0.45, 0.15, 1.0])));
    let light = diffuse_light(solid_texture(Color([7.0, 7.0, 7.0, 1.0])));
    let unit_cube = || {
        box_shape(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0))
            .apply_material(white.clone())
    };

    let shapes = shapes![
        yz_rectangle((0.0, 555.0), (0.0, 555.0), 555.0).apply_material(green.clone()),
        yz_rectangle((0.0, 555.0), (0.0, 555.0), 0.0).apply_material(red.clone()),
        xz_rectangle((113.0, 443.0), (127.0, 432.0), 554.0).apply_material(light.clone()),
        xz_rectangle((0.0, 555.0), (0.0, 555.0), 0.0).apply_material(white.clone()),
        xz_rectangle((0.0, 555.0), (0.0, 555.0), 555.0).apply_material(white.clone()),
        xy_rectangle((0.0, 555.0), (0.0, 555.0), 555.0).apply_material(white.clone()),
        constant_medium(
            0.01,
            unit_cube()
                .nonuniform_scale(165.0, 330.0, 160.0)
                .rotate_y(Deg(15.0).into())
                .translate(vec3(265.0, 0.0, 295.0)),
            isotropic(solid_texture(Color([0.0, 0.0, 0.0, 1.0])))
        ),
        constant_medium(
            0.01,
            unit_cube()
                .nonuniform_scale(165.0, 165.0, 165.0)
                .rotate_y(Deg(-18.0).into())
                .translate(vec3(130.0, 0.0, 65.0)),
            isotropic(solid_texture(Color([1.0, 1.0, 1.0, 1.0])))
        ),
    ];

    let shapes = shapes![shapes.nonuniform_scale(1.0, 1.0, 1.0)];

    (camera, black_sky(), shapes)
}

fn prism(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(278.0, 278.0, -800.0);
    let lookat = Point3::new(278.0, 278.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(40.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let white = lambertian(solid_texture(Color([0.73, 0.73, 0.73, 1.0])));
    let glass = dielectric(1.5);
    let light = diffuse_light(solid_texture(Color([30.0, 30.0, 30.0, 1.0])));

    let shapes = shapes![
        xz_rectangle((0.0, 555.0), (0.0, 555.0), 0.0).apply_material(white.clone()), // The floor
        xy_rectangle((0.0, 555.0), (0.0, 555.0), 555.0).apply_material(white.clone()), // The back wall
        yz_rectangle((250.0, 350.0), (0.0, 555.0), 1000.0).apply_material(light.clone()), // The light
        yz_rectangle((0.0, 270.0), (0.0, 555.0), 500.0).apply_material(white.clone()), // Bottom of the slit
        yz_rectangle((290.0, 555.0), (0.0, 555.0), 500.0).apply_material(white.clone()), // Top of the slit
        yz_rectangle((0.0, 555.0), (0.0, 555.0), 0.0).apply_material(white.clone()), // Target wall
        box_shape(Point3::new(0.0, 0.0, 0.0), Point3::new(50.0, 100.0, 555.0))
            .apply_material(glass.clone())
            .rotate_z(Deg(15.0).into())
            .translate(vec3(300.0, 250.0, 0.0)),
    ];

    let shapes = shapes![shapes.nonuniform_scale(1.0, 1.0, 1.0)];

    (camera, color_sky(Color([0.1, 0.1, 0.1, 1.0])), shapes)
}

fn book2_boxes_1(boxes_per_side: usize) -> CompoundPrimitive<TransformedXyRectangle> {
    let mut ret = Vec::new();

    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let (i, j) = (i as FloatType, j as FloatType);
            let w = 100.0;
            let x0 = -1000.0 + (i * w);
            let z0 = -1000.0 + (j * w);
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = random_in_range(1.0, 101.0);
            let z1 = z0 + w;

            ret.extend(box_shape(Point3::new(x0, y0, z0), Point3::new(x1, y1, z1)));
        }
    }

    ret.into_primitive()
}

fn book2_boxes_2(ns: usize) -> CompoundPrimitive<<Sphere as TransformablePrimitive>::Target> {
    (0..ns)
        .into_iter()
        .map(|_| {
            sphere(
                Point3::new(
                    random_in_range(0.0, 165.0),
                    random_in_range(0.0, 165.0),
                    random_in_range(0.0, 165.0),
                ),
                10.0,
            )
        })
        .into_primitive()
}

fn book2(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(478.0, 278.0, -600.0);
    let lookat = Point3::new(278.0, 278.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(40.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let shapes = shapes![
        book2_boxes_1(20).apply_material(lambertian(solid_texture(Color([0.48, 0.83, 0.53, 1.0])))),
        xz_rectangle((123.0, 423.0), (147.0, 412.0), 554.0)
            .apply_material(diffuse_light(solid_texture(Color([7.0, 7.0, 7.0, 1.0])))),
        moving_sphere(
            (Point3::new(400.0, 400.0, 400.0), 0.0),
            (Point3::new(430.0, 400.0, 200.0), 1.0),
            50.0
        )
        .apply_material(lambertian(solid_texture(Color([0.7, 0.3, 0.1, 1.0])))),
        sphere(Point3::new(260.0, 150.0, 45.0), 50.0).apply_material(dielectric(1.5)),
        sphere(Point3::new(0.0, 150.0, 145.0), 70.0)
            .apply_material(metal(Color([0.8, 0.8, 0.9, 1.0]), 10.0)),
        sphere(Point3::new(360.0, 150.0, 145.0), 70.0).apply_material(dielectric(1.5)),
        constant_medium(
            0.2,
            sphere(Point3::new(360.0, 150.0, 145.0), 70.0).apply_material(dielectric(1.5)),
            isotropic(solid_texture(Color([0.2, 0.4, 0.9, 1.0])))
        ),
        constant_medium(
            0.0001,
            sphere(Point3::new(0.0, 0.0, 0.0), 5000.0).apply_material(dielectric(1.5)),
            isotropic(solid_texture(Color([1.0, 1.0, 1.0, 1.0])))
        ),
        sphere(Point3::new(400.0, 200.0, 400.0), 100.0).apply_material(lambertian(earth_map())),
        sphere(Point3::new(220.0, 280.0, 300.0), 80.0)
            .apply_material(lambertian(noise_texture(0.1))),
        book2_boxes_2(1000)
            .apply_material(lambertian(solid_texture(Color([0.73, 0.73, 0.73, 1.0]))))
            .rotate_y(Deg(15.0).into())
            .translate(vec3(-100.0, 270.0, 295.0)),
    ];

    (camera, black_sky(), shapes)
}

fn orange(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(2.0, 2.0, -10.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(60.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let shapes = shapes![
        sphere(Point3::new(-3.0, 0.0, 0.0), 2.0).apply_material(bump_mapper(
            noise_normal(10.0, 0.4),
            lambertian(solid_texture(Color([1.0, 69.0 / 255.0, 0.0, 1.0])))
        )),
        sphere(Point3::new(3.0, 0.0, 0.0), 2.0).apply_material(bump_mapper(
            noise_normal(10.0, 0.2),
            metal(Color([1.0, 69.0 / 255.0, 0.0, 1.0]), 0.4)
        )),
        xy_rectangle((-8.0, 8.0), (-8.0, 8.0), 8.0).apply_material(bump_mapper(
            brick_normal_map(),
            metal_with_texture(brick_image(), 0.7)
        )),
        xz_rectangle((-10.0, 10.0), (-10.0, 10.0), -8.0)
            .apply_material(lambertian(solid_texture(Color([1.0, 1.0, 1.0, 1.0]))),),
        xz_rectangle((-6.0, 6.0), (-6.0, 6.0), 7.0)
            .apply_material(diffuse_light(solid_texture(Color([7.0, 7.0, 7.0, 1.0])))),
    ];

    (camera, black_sky(), shapes)
}

fn orange_parabola(width: usize, height: usize) -> (raster::Camera, raster::Sky, ShapeList) {
    let aspect_ratio = (width as FloatType) / (height as FloatType);
    let lookfrom = Point3::new(20.0, 5.0, -5.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).magnitude();
    let aperture = 0.0;
    let camera = raster::Camera::new(
        lookfrom,
        lookat,
        vup,
        Deg(20.0).into(),
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let shapes = shapes![
        /*sphere(
            Point3::new(-3.0, 0.0, 0.0),
            2.0,
            bump_mapper(
                noise_normal(10.0, 0.4),
                lambertian(solid_texture(Color([1.0, 69.0 / 255.0, 0.0, 1.0])))
            )
        ),
        sphere(
            Point3::new(3.0, 0.0, 0.0),
            2.0,
            bump_mapper(
                noise_normal(10.0, 0.2),
                metal(Color([1.0, 69.0 / 255.0, 0.0, 1.0]), 0.4)
            )
        ),*/
        parabola(Point3::new(0.0, 0.0, 7.5), Point3::new(0.0, 0.0, 7.0), 2.0)
            .apply_material(metal(Color([1.0, 1.0, 1.0, 1.0]), 0.1)),
        parabola(
            Point3::new(0.0, 0.0, -8.0),
            Point3::new(0.0, 0.0, -7.0),
            2.0
        )
        .apply_material(metal(Color([1.0, 1.0, 1.0, 1.0]), 0.1)),
        sphere(Point3::new(0.0, 0.0, 7.0), 0.5)
            .apply_material(diffuse_light(solid_texture(Color([7.0, 7.0, 7.0, 1.0]))),),
        xz_rectangle((-100.0, 100.0), (-100.0, 100.0), -3.0)
            .apply_material(lambertian(solid_texture(Color([1.0, 1.0, 1.0, 1.0]))),),
        yz_rectangle((-100.0, 100.0), (-100.0, 100.0), -2.0)
            .apply_material(lambertian(solid_texture(Color([1.0, 1.0, 1.0, 1.0]))),) /*xz_rectangle(
                                                                                         (-6.0, 6.0),
                                                                                         (-6.0, 6.0),
                                                                                         7.0,
                                                                                         diffuse_light(solid_texture(Color([7.0, 7.0, 7.0, 1.0])))
                                                                                     ),*/
    ];

    (camera, black_sky(), shapes)
}

const DEFAULT_WIDTH: usize = 1920;
const DEFAULT_HEIGHT: usize = 1080;
const DEFAULT_MIN_PASSES: usize = 100;
const DEFAULT_THREADS: usize = 8;
const DEFAULT_ENABLE_SPATIAL_PARTITIONING: bool = true;

const BUILTIN_SCENES: [(
    &'static str,
    fn(usize, usize) -> (raster::Camera, raster::Sky, ShapeList),
); 12] = [
    ("random", random_scene),
    ("mine", my_test_scene),
    ("twospheres", two_spheres),
    ("twoperlinspheres", two_perlin_spheres),
    ("earth", textured_earth),
    ("simplelight", simple_light),
    ("cornell", cornell_box),
    ("cornell_smoke", cornell_smoke),
    ("prism", prism),
    ("book2", book2),
    ("orange", orange),
    ("orange_parabola", orange_parabola),
];

fn command_line() -> clap::ArgMatches<'static> {
    App::new("raster")
        .version("1.0")
        .author("Stewart Tootill <stewart.tootill@live.co.uk>")
        .about("My raytracer")
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .help(&format!("Width of image, defaults to {}", DEFAULT_WIDTH))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .help(&format!("Height of image, defaults to {}", DEFAULT_HEIGHT))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("scene")
                .long("scene")
                .takes_value(true)
                .help(&format!(
                    "Choose a scene to render, defaults to {}",
                    BUILTIN_SCENES[0].0
                ))
                .possible_values(&BUILTIN_SCENES.iter().map(|a| a.0).collect::<Vec<_>>()),
        )
        .arg(
            Arg::with_name("threads")
                .long("threads")
                .help(&format!(
                    "Number of threads, defaults to {}",
                    DEFAULT_THREADS
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("min-passes")
                .long("min-passes")
                .help(&format!(
                    "Minimum number of passes, defaults to {}",
                    DEFAULT_MIN_PASSES
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("enable-spatial-partitioning")
                .long("enable-spatial-partitioning")
                .possible_values(&["true", "false"])
                .help(&format!(
                    "Enable spatial partitioning, defaults to {}",
                    DEFAULT_ENABLE_SPATIAL_PARTITIONING
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .help("File to write to")
                .required(true)
                .index(1)
                .takes_value(true),
        )
        .get_matches()
}

#[tokio::main]
async fn main() {
    let matches = command_line();

    let width = matches
        .value_of("width")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_WIDTH);
    let height = matches
        .value_of("height")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_HEIGHT);
    let output_file = matches.value_of("output").unwrap().to_string();
    let min_passes = matches
        .value_of("min-passes")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MIN_PASSES);
    let threads = matches
        .value_of("threads")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_THREADS);
    let enable_spatial_partitioning = matches
        .value_of("enable-spatial-partitioning")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(DEFAULT_ENABLE_SPATIAL_PARTITIONING);

    let scene_name = matches.value_of("scene").unwrap_or(BUILTIN_SCENES[0].0);
    let (scene_name, scene_function) = BUILTIN_SCENES.iter().find(|a| a.0 == scene_name).unwrap();

    let (camera, sky, shapes) = scene_function(width, height);
    let scene = raster::Scene::new(camera, sky, enable_spatial_partitioning, shapes);

    let (t0, t1) = (0.0, 1.0);

    println!(
        "Rendering scene \"{}\" at ({}, {})",
        scene_name, width, height
    );
    println!(
        "Using {} threads, with a minimum of {} passes per pixel",
        threads, min_passes
    );

    let expected_pass_count = ((min_passes + threads - 1) / threads) * threads;
    let expected_pixel_count = width * height * expected_pass_count;

    let start_time = std::time::Instant::now();
    let stats = Arc::new(RwLock::new(raster::TracingStats::new()));

    tokio::pin! {
        let scanner = raster::scan(scene, width, height, t0, t1, threads, min_passes, stats.clone());
    }
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

    let vector_image = loop {
        tokio::select! {
            _ = interval.tick() => {
                let stats_value = stats.read().unwrap().get_stats();
                let done_ratio = stats_value.pixels as f64 / expected_pixel_count as f64;
                let elapsed_time = start_time.elapsed().as_secs_f64();
                let estimated_time = (elapsed_time / done_ratio) - elapsed_time;
                println!("Elapsed time: {} seconds", elapsed_time);
                println!("{}% complete, estimated {} remaining", done_ratio * 100.0, estimated_time);
                println!("Tracing stats: {:#?}", stats_value);
            }

            image = &mut scanner => break image,
        }
    };

    let stats_value = stats.read().unwrap().get_stats();
    println!("FINISHED");
    println!(
        "Elapsed time: {} seconds",
        start_time.elapsed().as_secs_f64()
    );
    println!("Tracing stats: {:#?}", stats_value);

    let mut surf = RgbImage::new(width as u32, height as u32);

    vector_image
        .pixels()
        .zip(surf.pixels_mut())
        .fold({}, |_, (src, dst)| {
            let color = src / src.w;
            let color: Color = color.try_into().unwrap();
            *dst = color.gamma(2.0).into();
        });

    if let Err(e) = surf.save(output_file) {
        println!("Failed to write output: {}", e);
    }
}
