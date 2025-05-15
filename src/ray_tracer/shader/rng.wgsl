// adapted from [Nelarius/weekend-raytracer-wgpu]
// (https://github.com/Nelarius/weekend-raytracer-wgpu/blob/main/src/raytracer/raytracer.wgsl#L531)

const PI = 3.1415927f;

var<private> _rng: u32;

fn rng_init(pixel: vec2<u32>, resolution: vec2<u32>, frame: u32) {
    let seed = dot(pixel, vec2<u32>(1u, resolution.x)) ^ hash(frame);
    _rng = hash(seed);
}

fn rng_u32() -> u32 {
    _rng = _rng * 747796405u + 2891336453u;
    let word = ((_rng >> ((_rng >> 28u) + 4u)) ^ _rng) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rng_range_u32(min: u32, max: u32) -> u32 {
    return rng_u32() % (max - min) + min;
}

fn rng_f32() -> f32 {
    let x = rng_u32();
    return f32(x) / f32(0xffffffffu);
}

fn rng_unit_disk_f32() -> vec3<f32> {
    let r = sqrt(rng_f32());
    let alpha = 2 * PI * rng_f32();
    let x = r * cos(alpha);
    let y = r * sin(alpha);
    return vec3<f32>(x, y, 0);
}

fn rng_unit_sphere_f32() -> vec3<f32> {
    let r = pow(rng_f32(), 0.33333f);
    let cosTheta = 1 - 2 * rng_f32();
    let sinTheta = sqrt(1 - cosTheta * cosTheta);
    let phi = 2 * PI * rng_f32();
    let x = r * sinTheta * cos(phi);
    let y = r * sinTheta * sin(phi);
    let z = cosTheta;
    return vec3<f32>(x, y, z);
}

fn rng_unit_hemisphere_f32() -> vec3<f32> {
    let r1 = rng_f32();
    let r2 = rng_f32();
    let phi = 2 * PI * r1;
    let sinTheta = sqrt(1 - r2 * r2);
    let x = cos(phi) * sinTheta;
    let y = sin(phi) * sinTheta;
    let z = r2;
    return vec3<f32>(x, y, z);
}

fn rng_cosine_weighted_hemisphere_f32() -> vec3<f32> {
    let r1 = rng_f32();
    let r2 = rng_f32();
    let sqrt_r2 = sqrt(r2);
    let z = sqrt(1 - r2);
    let phi = 2 * PI * r1;
    let x = cos(phi) * sqrt_r2;
    let y = sin(phi) * sqrt_r2;
    return vec3<f32>(x, y, z);
}

fn hash(x: u32) -> u32 {
    var h = x;
    h += h << 10u;
    h ^= h >> 6u;
    h += h << 3u;
    h ^= h >> 11u;
    h += h << 15u;
    return h;
}
