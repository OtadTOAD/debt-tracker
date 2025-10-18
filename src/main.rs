#![windows_subsystem = "windows"]

mod app;
mod database;
mod models;

use eframe::egui;

use crate::app::BankingApp;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([700.0, 500.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "מעקב אחר בנקאות אישית",
        native_options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "fallback".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/NotoSans-Regular.ttf")),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "fallback".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("fallback".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            let mut app = BankingApp::default();

            if let Ok(img) = image::load_from_memory(include_bytes!("../assets/logo.png")) {
                let img = img.to_rgba8();
                let (w, h) = img.dimensions();
                let pixels = img.into_raw();
                let image =
                    egui::ColorImage::from_rgba_premultiplied([w as usize, h as usize], &pixels);
                let texture = cc
                    .egui_ctx
                    .load_texture("logo", image, egui::TextureOptions::LINEAR);
                app.logo_texture = Some(texture);
            } else {
                app.logo_texture = None;
            }

            Box::new(app)
        }),
    )
}

fn load_icon() -> egui::IconData {
    if let Ok(image_data) = image::load_from_memory(include_bytes!("../assets/logo.png")) {
        let image_buffer = image_data.to_rgba8();
        let (width, height) = image_buffer.dimensions();
        let pixels = image_buffer.into_raw();

        egui::IconData {
            rgba: pixels,
            width,
            height,
        }
    } else {
        egui::IconData {
            rgba: vec![0, 120, 255, 255].repeat(32 * 32),
            width: 32,
            height: 32,
        }
    }
}
