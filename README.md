# Ray Tracing in Rust with WGPU

![](docs/weekend_scene.png)

A simple ray tracing implementation in Rust using WGPU, inspired by the [Ray Tracing in One Weekend Series](https://raytracing.github.io/) series.

## Running the Project

1. Install rust compiler `rustc` and builder/package manager `cargo` from [Install Rust](https://www.rust-lang.org/tools/install).
2. Build and run the project by:
    ```bash
    cargo run --release
    ```

### Camera and Render Controls

- Move the camera using WASD keys.
- Pan the camera by dragging the mouse.
- Change field of view using the mouse wheel.
- Adjust other parameters in the control panel.

## Project Structure

### Rust

I choose to use Rust as the main programming language mainly because I want to learn it and to experience its safety features and funtional programming designs.

It has a great ecosystem for system programming, providing numerous crates (Rust packages) for building desktop applications. Inclueding but not limited to:

- [egui](https://www.egui.rs): GUI framework for handling input and building the control panel.
- [wgpu](https://wgpu.rs): GPU programming framework based on the WebGPU API.
- [cgmath](https://docs.rs/cgmath/latest/cgmath): Math library for graphics applications.
- [encase](https://docs.rs/encase/latest/encase): Utility library that serializes Rust structures to conform to WebGPU memory alignment requirements.

### Graphics

This project implements all ray tracing features in [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html), including three types of materials (lambertian, metal, and dielectric), and rendering spheres.

To be continued...

### Web Compatibility

TDB

## References

### WebGPU/WGPU

- Tutorials: [WebGPU Fundamentals](https://webgpufundamentals.org/)
- Tutorials: [Learn Wgpu](https://sotrh.github.io/learn-wgpu/)

### Ray Tracing

- Books: [Ray Tracing in One Weekend Series](https://raytracing.github.io/)
- GitHub: [Nelarius/weekend-raytracer-wgpu](https://github.com/Nelarius/weekend-raytracer-wgpu)
- GitHub: [gnikoloff/webgpu-raytracer](https://github.com/gnikoloff/webgpu-raytracer)
- Tutorial: [Ray-Tracing: Rendering a Triangle](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html)
