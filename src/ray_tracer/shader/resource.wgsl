struct Param {
    camera: Camera,
    window_size: vec2<u32>,
    max_bounce: u32,
}

struct Camera {
    position: vec3<f32>,
    // unit vectors of the lens plane
    horizontal: vec3<f32>,
    vertical: vec3<f32>,

    start: vec3<f32>,
    // vectors of the focus plane
    vx: vec3<f32>,
    vy: vec3<f32>,

    lens_radius: f32,
}

struct Stat {
    frame_counter: u32,
}
