use eframe::{egui_wgpu, wgpu};

use ray_tracing_wgpu::ray_tracer::{self, RayTracer};

const SPEED: f32 = 0.1;
const SENSITIVITY: f32 = 0.1;

pub struct App {
    param: ray_tracer::param::Param,
    dragging: bool,
    last_mouse_pos: Option<egui::Pos2>,
}

impl App {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let state = cc.wgpu_render_state.as_ref().unwrap();
        let ray_traycer = RayTracer::new(&state.device, &state.queue, state.target_format.into());
        state
            .renderer
            .write()
            .callback_resources
            .insert(ray_traycer);

        Self {
            param: ray_tracer::param::Param::default(),
            dragging: false,
            last_mouse_pos: None,
        }
    }

    fn paint_canvas(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();

        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::drag());

        self.param.display_size = cgmath::Vector2::new(rect.size().x as u32, rect.size().y as u32);

        if response.drag_started() {
            self.dragging = true;
            self.last_mouse_pos = response.interact_pointer_pos();
        }
        if response.drag_stopped() {
            self.dragging = false;
            self.last_mouse_pos = None;
        }
        if self.dragging {
            if let Some(current_pos) = response.interact_pointer_pos() {
                if let Some(last_pos) = self.last_mouse_pos {
                    let delta = current_pos - last_pos;
                    self.param.camera.yaw -= delta.x * SENSITIVITY;
                    self.param.camera.pitch += delta.y * SENSITIVITY;

                    self.param.camera.pitch = self.param.camera.pitch.clamp(-89.9, 89.9);
                }
                self.last_mouse_pos = Some(current_pos);
            }
        }

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RayTracerCallback {
                param: self.param.clone(),
            },
        ));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|input| {
            let yaw = self.param.camera.yaw;
            let forward =
                -cgmath::Matrix3::from_angle_y(cgmath::Deg(yaw)) * cgmath::Vector3::unit_z();
            let right = cgmath::Matrix3::from_angle_y(cgmath::Deg(yaw)) * cgmath::Vector3::unit_x();

            for event in &input.events {
                match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        match key {
                            egui::Key::W => self.param.camera.position += SPEED * forward,
                            egui::Key::A => self.param.camera.position -= SPEED * right,
                            egui::Key::S => self.param.camera.position -= SPEED * forward,
                            egui::Key::D => self.param.camera.position += SPEED * right,
                            egui::Key::Space => self.param.camera.position.y += SPEED,
                            _ => {}
                        };

                        if modifiers.shift {
                            self.param.camera.position.y -= SPEED;
                        }
                    }
                    egui::Event::MouseWheel { delta, .. } => {
                        self.param.camera.fov += delta.y * 10.0 * SENSITIVITY;
                        self.param.camera.fov = self.param.camera.fov.clamp(20.0, 120.0);
                    }
                    _ => {}
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.paint_canvas(ui);
            });
        });
    }
}

struct RayTracerCallback {
    param: ray_tracer::param::Param,
}

impl egui_wgpu::CallbackTrait for RayTracerCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let ray_tracer: &mut ray_tracer::RayTracer = resources.get_mut().unwrap();

        ray_tracer.set_params(queue, &self.param);
        ray_tracer.update(device, queue);

        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        ray_tracer: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &ray_tracer::RayTracer = resources.get().unwrap();

        resources.render(ray_tracer);
    }
}
