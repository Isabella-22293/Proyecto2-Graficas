mod framebuffer;
mod ray_intersect;
mod cube;
mod color;
mod camera;
mod light;
mod material;
mod texture;

use minifb::{Window, WindowOptions, Key};
use nalgebra_glm::{Vec3, normalize};
use std::time::Duration;
use std::f32::consts::PI;

use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;
use crate::texture::Texture;
use image::{DynamicImage, GenericImageView};

const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(68, 142, 228);

fn load_texture_from_file(file_path: &str) -> Texture {
    // Carga la imagen usando la crate `image`
    let img = image::open(file_path).expect("Failed to open image");
    let (width, height) = img.dimensions();
    
    // Convertir la imagen a un Vec<Color>
    let mut pixel_data = Vec::new();
    if let DynamicImage::ImageRgb8(rgb_image) = img {
        for pixel in rgb_image.pixels() {
            // Usar el constructor `new` para crear un color
            let color = Color::new(pixel[0], pixel[1], pixel[2]);
            pixel_data.push(color);
        }
    }
    
    // Crear la textura
    Texture::new(pixel_data, width as usize, height as usize)
}

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        n_cosi = cosi;
        eta = eta_t;
        n_normal = *normal;
    }
    
    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);
    
    if k < 0.0 {
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn cast_shadow(intersect: &Intersect, light: &Light, objects: &[Cube]) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();
    let shadow_ray_origin = offset_origin(intersect, &light_dir);

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            return 1.0 - (shadow_intersect.distance / light_distance).min(1.0).powf(2.0);
        }
    }

    0.0
}

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, objects: &[Cube], lights: &[Light], depth: u32) -> Color {
    if depth > 3 {
        return SKYBOX_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return SKYBOX_COLOR;
    }

    let material = &intersect.material;
    
    let mut final_color = if let Some(texture) = &material.texture {
        let uv = intersect.uv.unwrap_or((0.0, 0.0));
        texture.get_color_at(uv.0, uv.1)
    } else {
        material.diffuse
    };

    let view_dir = (ray_origin - intersect.point).normalize();

    // Si el material tiene un índice de refracción, calculamos la refracción
    if material.refractive_index > 1.0 {
        let refracted_dir = refract(ray_direction, &intersect.normal, material.refractive_index);
        let refracted_origin = offset_origin(&intersect, &refracted_dir);
        let refracted_color = cast_ray(&refracted_origin, &refracted_dir, objects, lights, depth + 1);
        final_color = final_color * material.albedo[0] + refracted_color * material.albedo[3];
    } else {
        for light in lights {
            let light_dir = (light.position - intersect.point).normalize();
            let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();
            let shadow_intensity = cast_shadow(&intersect, light, objects);
            let light_intensity = light.intensity * (1.0 - shadow_intensity);

            let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
            let diffuse = final_color * material.albedo[0] * diffuse_intensity * light_intensity;

            let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(material.specular);
            let specular = light.color * material.albedo[1] * specular_intensity * light_intensity;

            final_color += diffuse + specular;
        }
    }

    final_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, lights: &[Light]) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let ray_direction = normalize(&Vec3::new(screen_x * aspect_ratio * perspective_scale, screen_y * perspective_scale, -1.0));
            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, lights, 0);

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 200;
    let window_height = 100;
    let framebuffer_width = 200;
    let framebuffer_height = 100;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new("Diorama", window_width, window_height, WindowOptions::default()).unwrap();

    // Cargar las texturas
    let dirt_texture = load_texture_from_file("src/image/Dirt.jpg");
    let grass_texture = load_texture_from_file("src/image/Grass.jpg");
    let cobblestone_texture = load_texture_from_file("src/image/Cobblestone.jpg");
    let plank_texture = load_texture_from_file("src/image/Plank.jpg");
    let glass_texture = load_texture_from_file("src/image/Glass.jpg");
    let door_texture = load_texture_from_file("src/image/door.png"); // Cargar la textura de la puerta

    // Crear los materiales
    let dirt_material = Material::new(Color::black(), 15.0, [0.5, 0.3, 0.0, 0.0], 0.0, Some(dirt_texture));
    let grass_material = Material::new(Color::black(), 15.0, [0.5, 0.5, 0.0, 0.0], 0.0, Some(grass_texture));
    let cobblestone_material = Material::new(Color::black(), 15.0, [0.5, 0.5, 0.0, 0.0], 0.0, Some(cobblestone_texture));
    let plank_material = Material::new(Color::black(), 15.0, [0.5, 0.5, 0.0, 0.0], 0.0, Some(plank_texture));
    let glass_material = Material::new(Color::black(), 15.0, [0.1, 0.1, 0.8, 0.0], 0.0, Some(glass_texture));
    let door_material = Material::new(Color::black(), 15.0, [0.5, 0.5, 0.0, 0.0], 0.0, Some(door_texture)); // Crear material de la puerta

    // Generar cubos de tierra (suelo)
    let mut objects: Vec<Cube> = Vec::new();
    let grid_size = 10; // Tamaño de la cuadrícula (10x10)
    let cube_size = 1.0; // Tamaño de cada cubo de tierra

    // Crear la cuadrícula de cubos de tierra
    for x in 0..grid_size {
        for z in 0..grid_size {
            let x_pos = (x as f32) * cube_size - (grid_size as f32 * cube_size / 2.0);
            let z_pos = (z as f32) * cube_size - (grid_size as f32 * cube_size / 2.0);
            let y_pos = -1.0; // Todos los cubos de tierra estarán en la misma altura

            let cube = Cube::new(
                Vec3::new(x_pos, y_pos, z_pos),                // Posición inicial
                Vec3::new(x_pos + cube_size, y_pos + cube_size, z_pos + cube_size), // Posición final
                dirt_material.clone().into(), // Usar el material de tierra
            );

            objects.push(cube);
        }
    }

    // Crear cubos a la izquierda con textura de cobblestone
    for x in 0..(grid_size / 2) {
        for z in 0..grid_size {
            let x_pos = (x as f32) * cube_size - (grid_size as f32 * cube_size / 2.0);
            let z_pos = (z as f32) * cube_size - (grid_size as f32 * cube_size / 2.0);
            let y_pos = 0.0; // Altura de los cubos de cobblestone

            let cube = Cube::new(
                Vec3::new(x_pos, y_pos, z_pos),                // Posición inicial
                Vec3::new(x_pos + cube_size, y_pos + cube_size, z_pos + cube_size), // Posición final
                cobblestone_material.clone().into(), // Usar el material de cobblestone
            );

            objects.push(cube);
        }
    }

    // Crear cubos a la derecha con textura de grass
    for x in (grid_size / 2)..grid_size {
        for z in 0..grid_size {
            let x_pos = (x as f32) * cube_size - (grid_size as f32 * cube_size / 2.0);
            let z_pos = (z as f32) * cube_size - (grid_size as f32 * cube_size / 2.0);
            let y_pos = 0.0; // Altura de los cubos de grass

            let cube = Cube::new(
                Vec3::new(x_pos, y_pos, z_pos),                // Posición inicial
                Vec3::new(x_pos + cube_size, y_pos + cube_size, z_pos + cube_size), // Posición final
                grass_material.clone().into(), // Usar el material de grass
            );

            objects.push(cube);
        }
    }

    // Definir el tamaño y posición de la casa
    let house_width = 6;
    let house_height = 5;
    let house_depth = 4;
    let cube_size = 1.0;

    // Crear la fachada de la casa con cubos de plank, con la puerta al frente
    for y in 0..house_height {
        for x in 0..house_width {
            for z in 0..house_depth {
                let x_pos = (x as f32) * cube_size - (grid_size as f32 * cube_size / 4.0); // Centrando la casa
                let z_pos = (z as f32) * cube_size - (grid_size as f32 * cube_size / 2.0);
                let y_pos = y as f32; // Altura

                // Colocar la puerta en la fachada delantera
                let material = if y == 0 && x == house_width / 2 && z == 0 {
                    door_material.clone().into() // Puerta en la parte delantera
                // Ventanas de 4 cubos de glass ahora en los niveles y = 2 y y = 3
                } else if y == 2 && (x == 1 || x == house_width - 2) && (z == 0 || z == house_depth - 1) {
                    glass_material.clone().into() // Parte inferior de las ventanas más altas
                } else if y == 3 && (x == 1 || x == house_width - 2) && (z == 0 || z == house_depth - 1) {
                    glass_material.clone().into() // Parte superior de las ventanas más altas
                // Ventanas laterales
                } else if (y == 2 || y == 3) && (x == 0 || x == house_width - 1) && (z == house_depth / 2) {
                    glass_material.clone().into() // Ventana lateral
                } else if y == 2 && (x == 0 || x == house_width - 1) && (z == house_depth / 2 + 1) {
                    plank_material.clone().into() // Cubo de madera entre las ventanas laterales
                } else if (y == 2 || y == 3) && (x == 0 || x == house_width - 1) && (z == house_depth / 2 - 1) {
                    glass_material.clone().into() // Ventana lateral
                // Ventana en el techo
                } else if y == house_height - 1 && (x >= 1 && x <= 4) && z == 1 {
                    glass_material.clone().into() // Ventana en el techo
                } else {
                    plank_material.clone().into() // Pared de plank
                };

                let cube = Cube::new(
                    Vec3::new(x_pos, y_pos, z_pos), // Posición inicial
                    Vec3::new(x_pos + cube_size, y_pos + cube_size, z_pos + cube_size), // Posición final
                    material,
                );

                objects.push(cube);
            }
        }
    }

    // Cámara
    let mut camera = Camera::new(Vec3::new(0.0, 3.0, -10.0), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));

    // Luz
    let mut lights = [
        Light::new(Vec3::new(5.0, 5.0, -10.0), Color::new(255, 255, 255), 1.0),
    ];

    // Bucle principal
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Control de rotación de la cámara
        let rotation_speed = PI / 10.0;
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        // Control de zoom
        if window.is_key_down(Key::W) {
            camera.zoom(0.1);  
        }
        if window.is_key_down(Key::S) {
            camera.zoom(-0.1);  
        }

        // Control de la luz
        if window.is_key_down(Key::I) {
            lights[0].position.y += 0.1;
        }
        if window.is_key_down(Key::K) {
            lights[0].position.y -= 0.1;
        }
        if window.is_key_down(Key::J) {
            lights[0].position.x -= 0.1;
        }
        if window.is_key_down(Key::L) {
            lights[0].position.x += 0.1;
        }
        if window.is_key_down(Key::U) {
            lights[0].position.z += 0.1;
        }
        if window.is_key_down(Key::O) {
            lights[0].position.z -= 0.1;
        }

        render(&mut framebuffer, &objects, &camera, &lights);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
        std::thread::sleep(frame_delay);
    }
}
