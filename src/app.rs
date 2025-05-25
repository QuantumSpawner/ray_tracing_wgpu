use eframe::{egui_wgpu, wgpu};
use std::sync::{Arc, Mutex};

use ray_tracing_wgpu::ray_tracer::{self, RayTracer};

const SPEED: f32 = 0.1;
const SENSITIVITY: f32 = 0.1;

#[cfg(not(target_arch = "wasm32"))]
const MOUSE_WHEEL_SENSITIVITY: f32 = 1.0;
#[cfg(target_arch = "wasm32")]
const MOUSE_WHEEL_SENSITIVITY: f32 = 0.1;

pub struct App {
    param: ray_tracer::Param,
    stat: Arc<Mutex<ray_tracer::Stat>>,
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
            param: ray_tracer::Param::default(),
            stat: Arc::new(Mutex::new(ray_tracer::Stat::default())),
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
                    self.param.camera.pitch -= delta.y * SENSITIVITY;

                    self.param.camera.pitch = self.param.camera.pitch.clamp(-89.9, 89.9);
                }
                self.last_mouse_pos = Some(current_pos);
            }
        }

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RayTracerCallback {
                param: self.param.clone(),
                stat: self.stat.clone(),
            },
        ));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let stat = self.stat.lock().unwrap();

        if stat.is_rendering {
            ctx.request_repaint();
        }

        egui::Window::new("Control Panel")
            .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
            .resizable(false)
            .collapsible(true)
            .show(ctx, |ui| {
                let panel_width = 320.0;
                let label_width = 100.0;
                ui.set_width(panel_width);
                ui.add_space(4.0);

                let status = if stat.is_rendering {
                    format!("Rendering ({:.1}s elapsed)", stat.time_spent.as_secs_f32())
                } else {
                    format!("Finished ({:.1}s total)", stat.time_spent.as_secs_f32())
                };
                ui.label(egui::RichText::new(status).heading().strong());
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.active.bg_fill))
                    .show(ui, |ui| {
                        ui.add(
                            egui::ProgressBar::new(
                                stat.frame_counter as f32 / self.param.max_sample as f32,
                            )
                            .text(format!(
                                "Frame: {}/{}",
                                stat.frame_counter, self.param.max_sample
                            )),
                        );

                        ui.label(format!(
                            "Screen Size: {}x{}",
                            self.param.display_size.x, self.param.display_size.y
                        ));
                    });

                ui.label(egui::RichText::new("Sampling").heading().strong());
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.active.bg_fill))
                    .show(ui, |ui| {
                        ui.set_width(panel_width - 16.0);

                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("Samples per Pixel"));
                            ui.add(egui::Slider::new(&mut self.param.max_sample, 1..=4096));
                        });
                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("Max Bounces"));
                            ui.add(egui::Slider::new(&mut self.param.max_bounce, 1..=32));
                        });
                    });

                ui.label(egui::RichText::new("Camera").heading().strong());
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.active.bg_fill))
                    .show(ui, |ui| {
                        ui.set_width(panel_width - 16.0);

                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("FOV"));
                            ui.add(egui::Slider::new(&mut self.param.camera.fov, 10.0..=120.0));
                        });
                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("Aperture"));
                            ui.add(egui::Slider::new(
                                &mut self.param.camera.aperture,
                                0.0..=1.0,
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("Focus Distance"));
                            ui.add(egui::Slider::new(
                                &mut self.param.camera.focus_distance,
                                0.0..=100.0,
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("Pitch"));
                            ui.add(egui::Slider::new(
                                &mut self.param.camera.pitch,
                                -89.9..=89.9,
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("Yaw"));
                            ui.add(egui::Slider::new(
                                &mut self.param.camera.yaw,
                                -180.0..=180.0,
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Position");
                            ui.add(
                                egui::DragValue::new(&mut self.param.camera.position.x)
                                    .speed(0.1)
                                    .prefix("x: "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.param.camera.position.y)
                                    .speed(0.1)
                                    .prefix("y: "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.param.camera.position.z)
                                    .speed(0.1)
                                    .prefix("z: "),
                            );
                        });
                    });

                ui.label(egui::RichText::new("Algorithm").heading().strong());
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.active.bg_fill))
                    .show(ui, |ui| {
                        ui.set_width(panel_width - 16.0);

                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("Hit"));
                            ui.radio_value(
                                &mut self.param.hit_algorithm,
                                ray_tracer::HitAlgorithm::Brute,
                                "Brute Force",
                            );
                            ui.radio_value(
                                &mut self.param.hit_algorithm,
                                ray_tracer::HitAlgorithm::BVH,
                                "BVH",
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, 0.0], egui::Label::new("Shading"));
                            ui.radio_value(
                                &mut self.param.shading_algorithm,
                                ray_tracer::ShadingAlgorithm::Flat,
                                "Flat",
                            );
                            ui.radio_value(
                                &mut self.param.shading_algorithm,
                                ray_tracer::ShadingAlgorithm::Smooth,
                                "Smooth",
                            );
                        });
                    });
            });

        ctx.input(|input| {
            let yaw = self.param.camera.yaw;
            let forward =
                -cgmath::Matrix3::from_angle_y(cgmath::Deg(yaw)) * cgmath::Vector3::unit_z();
            let right = cgmath::Matrix3::from_angle_y(cgmath::Deg(yaw)) * cgmath::Vector3::unit_x();

            for event in &input.events {
                match event {
                    egui::Event::Key {
                        key, pressed: true, ..
                    } => match key {
                        egui::Key::W => self.param.camera.position += SPEED * forward,
                        egui::Key::A => self.param.camera.position -= SPEED * right,
                        egui::Key::S => self.param.camera.position -= SPEED * forward,
                        egui::Key::D => self.param.camera.position += SPEED * right,
                        egui::Key::Space => self.param.camera.position.y += SPEED,
                        egui::Key::ArrowUp => self.param.camera.position += SPEED * forward,
                        egui::Key::ArrowDown => self.param.camera.position -= SPEED * forward,
                        egui::Key::ArrowLeft => self.param.camera.position -= SPEED * right,
                        egui::Key::ArrowRight => self.param.camera.position += SPEED * right,
                        _ => {}
                    },
                    egui::Event::MouseWheel { delta, .. } => {
                        self.param.camera.fov -= delta.y * MOUSE_WHEEL_SENSITIVITY;
                        self.param.camera.fov = self.param.camera.fov.clamp(10.0, 120.0);
                    }
                    _ => {}
                }
            }

            if input.modifiers.shift {
                self.param.camera.position.y -= SPEED;
            }
        });

        drop(stat);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.paint_canvas(ui);
            });
        });
    }
}

struct RayTracerCallback {
    param: ray_tracer::Param,
    stat: Arc<Mutex<ray_tracer::Stat>>,
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

        let mut stat = self.stat.lock().unwrap();
        *stat = ray_tracer.get_stat().clone();

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
