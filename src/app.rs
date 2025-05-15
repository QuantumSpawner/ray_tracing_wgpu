use eframe::{egui_wgpu, wgpu};

use ray_tracing_wgpu::ray_tracer;

pub struct App {}

impl App {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let state = cc.wgpu_render_state.as_ref().unwrap();
        let ray_traycer =
            ray_tracer::RayTracer::new(&state.device, &state.queue, state.target_format.into());
        state
            .renderer
            .write()
            .callback_resources
            .insert(ray_traycer);

        Self {}
    }

    fn paint_canvas(&self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();

        let (rect, _) = ui.allocate_exact_size(available_size, egui::Sense::drag());

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RayTracerCallback {
                window_size: cgmath::Vector2::new(rect.size().x as u32, rect.size().y as u32),
            },
        ));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.paint_canvas(ui);
            });
        });
    }
}

struct RayTracerCallback {
    window_size: cgmath::Vector2<u32>,
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

        ray_tracer.set_params(
            queue,
            ray_tracer::Param {
                display_size: self.window_size,
                ..ray_tracer.get_params().clone()
            },
        );

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
