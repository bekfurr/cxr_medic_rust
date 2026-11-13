use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use egui::{
    vec2, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect,
    RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2,
};

use crate::constants::{
    CdssDisclaimer, MedicalColors, ALL_PATHOLOGIES,
};
use crate::models::{HealthStatus, PathologyItem, PredictionRequest, PredictionResponse};
use crate::services::{
    ApiService, Base64Helper, MockDiagnosisService, APP_ICON_PNG,
    SAMPLE_CXR_PNG,
};

enum AsyncMsg {
    HealthResult(Result<HealthStatus, String>),
    PredictionResult(Result<PredictionResponse, String>),
}

pub struct BmiCxrApp {
    // Server & Config
    server_url: String,
    offline_demo_mode: bool,
    is_connected: bool,
    health_status: Option<HealthStatus>,
    is_checking_connection: bool,
    show_server_modal: bool,

    // Legal / Disclaimer
    is_disclaimer_accepted: bool,
    show_disclaimer_modal: bool,

    // Patient & Image input
    selected_image_bytes: Option<Vec<u8>>,
    selected_image_name: String,
    view_pos: String,
    age: i32,
    gender: String,

    // Viewport & Grad-CAM state
    gradcam_opacity: f32,
    zoom_scale: f32,
    pan_offset: Vec2,

    // Diagnosis state
    is_loading: bool,
    is_switching_cam: bool,
    prediction: Option<PredictionResponse>,
    selected_pathology_label: Option<String>,
    error_message: Option<String>,
    only_positive_filter: bool,

    // Mobile navigation (0: Patient, 1: Viewport, 2: Pathologies)
    mobile_tab: usize,

    // Channel for background threads
    tx: Sender<AsyncMsg>,
    rx: Receiver<AsyncMsg>,

    // Textures cache
    cxr_texture: Option<egui::TextureHandle>,
    heatmap_texture: Option<egui::TextureHandle>,
    icon_texture: Option<egui::TextureHandle>,
    active_cxr_hash: u64,
    active_heatmap_hash: u64,
}

impl BmiCxrApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load clean Windows system font (Segoe UI) if available for pristine typography
        #[cfg(target_os = "windows")]
        if let Ok(font_bytes) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "segoe_ui".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(font_bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "segoe_ui".to_owned());
            cc.egui_ctx.set_fonts(fonts);
        }

        // Configure medical dark theme
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(MedicalColors::TEXT_PRIMARY);
        visuals.panel_fill = MedicalColors::BACKGROUND;
        visuals.window_fill = MedicalColors::SURFACE_ELEVATED;
        visuals.window_stroke = Stroke::new(1.0f32, MedicalColors::BORDER);
        visuals.window_rounding = Rounding::same(12.0);

        visuals.widgets.noninteractive.bg_fill = MedicalColors::SURFACE;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0f32, MedicalColors::BORDER);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0f32, MedicalColors::TEXT_PRIMARY);
        visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

        visuals.widgets.inactive.bg_fill = MedicalColors::SURFACE_ELEVATED;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0f32, MedicalColors::BORDER);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0f32, MedicalColors::TEXT_PRIMARY);
        visuals.widgets.inactive.rounding = Rounding::same(8.0);

        visuals.widgets.hovered.bg_fill = MedicalColors::SURFACE_ELEVATED;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.2f32, MedicalColors::PRIMARY_CYAN);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0f32, Color32::WHITE);
        visuals.widgets.hovered.rounding = Rounding::same(8.0);

        visuals.widgets.active.bg_fill = MedicalColors::PRIMARY_CYAN;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5f32, MedicalColors::PRIMARY_CYAN);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0f32, Color32::BLACK);
        visuals.widgets.active.rounding = Rounding::same(8.0);

        cc.egui_ctx.set_visuals(visuals);

        // Configure default button and item spacing
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.button_padding = vec2(14.0, 8.0);
        style.spacing.item_spacing = vec2(8.0, 8.0);
        cc.egui_ctx.set_style(style);

        let (tx, rx) = channel();

        let mut app = Self {
            server_url: "http://127.0.0.1:5000".to_string(),
            offline_demo_mode: false,
            is_connected: false,
            health_status: None,
            is_checking_connection: false,
            show_server_modal: false,

            is_disclaimer_accepted: false,
            show_disclaimer_modal: true,

            selected_image_bytes: Some(SAMPLE_CXR_PNG.to_vec()),
            selected_image_name: "Namuna Rentgen (Demo PA 58Y M)".to_string(),
            view_pos: "PA".to_string(),
            age: 58,
            gender: "M".to_string(),

            gradcam_opacity: 0.65,
            zoom_scale: 1.0,
            pan_offset: Vec2::ZERO,

            is_loading: false,
            is_switching_cam: false,
            prediction: None,
            selected_pathology_label: None,
            error_message: None,
            only_positive_filter: false,

            mobile_tab: 1, // Default to Viewport tab on mobile
            tx,
            rx,

            cxr_texture: None,
            heatmap_texture: None,
            icon_texture: None,
            active_cxr_hash: 0,
            active_heatmap_hash: 0,
        };

        // Check server health initially
        app.check_server_health();
        app
    }

    pub fn check_server_health(&mut self) {
        if self.offline_demo_mode {
            return;
        }
        self.is_checking_connection = true;
        let url = self.server_url.clone();
        let tx = self.tx.clone();

        thread::spawn(move || {
            let res = ApiService::check_health(&url);
            let _ = tx.send(AsyncMsg::HealthResult(res));
        });
    }

    pub fn load_sample_radiograph(&mut self) {
        self.selected_image_bytes = Some(SAMPLE_CXR_PNG.to_vec());
        self.selected_image_name = "Namuna Rentgen (Demo PA 58Y M)".to_string();
        self.view_pos = "PA".to_string();
        self.age = 58;
        self.gender = "M".to_string();
        self.prediction = None;
        self.selected_pathology_label = None;
        self.error_message = None;
        self.zoom_scale = 1.0;
        self.pan_offset = Vec2::ZERO;
    }

    pub fn pick_image_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rentgen Tasvirlari", &["png", "jpg", "jpeg", "bmp", "webp"])
            .pick_file()
        {
            if let Ok(bytes) = std::fs::read(&path) {
                let name = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "rentgen.png".to_string());
                self.selected_image_bytes = Some(bytes);
                self.selected_image_name = name;
                self.prediction = None;
                self.selected_pathology_label = None;
                self.error_message = None;
                self.zoom_scale = 1.0;
                self.pan_offset = Vec2::ZERO;
            }
        }
    }

    pub fn run_diagnosis(&mut self, target_label: Option<String>) {
        let Some(image_bytes) = &self.selected_image_bytes else {
            self.error_message = Some("Iltimos, avval rentgen tasvirini yuklang.".to_string());
            return;
        };

        if target_label.is_some() {
            self.is_switching_cam = true;
        } else {
            self.is_loading = true;
        }
        self.error_message = None;

        let b64 = Base64Helper::clean_encode(image_bytes);
        let req = PredictionRequest {
            image_base64: b64,
            view_pos: self.view_pos.clone(),
            age: self.age,
            gender: self.gender.clone(),
            with_cam: true,
            target_label: target_label.clone(),
            lang: "uz".to_string(),
        };

        let offline_mode = self.offline_demo_mode || !self.is_connected;
        let server_url = self.server_url.clone();
        let tx = self.tx.clone();

        thread::spawn(move || {
            if offline_mode {
                thread::sleep(Duration::from_millis(250));
                let resp = MockDiagnosisService::simulate_diagnosis(&req);
                let _ = tx.send(AsyncMsg::PredictionResult(Ok(resp)));
            } else {
                match ApiService::predict(&server_url, &req) {
                    Ok(resp) => {
                        let _ = tx.send(AsyncMsg::PredictionResult(Ok(resp)));
                    }
                    Err(err) => {
                        let mut resp = MockDiagnosisService::simulate_diagnosis(&req);
                        resp.clinical_summary = format!(
                            "[Oflayn Zaxira Rejimi]: Serverga ulanishda xatolik ({err}). Namuna tahlili ko'rsatilmoqda."
                        );
                        let _ = tx.send(AsyncMsg::PredictionResult(Ok(resp)));
                    }
                }
            }
        });
    }

    fn poll_async_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMsg::HealthResult(res) => {
                    self.is_checking_connection = false;
                    match res {
                        Ok(health) => {
                            self.is_connected = true;
                            self.health_status = Some(health);
                        }
                        Err(_) => {
                            self.is_connected = false;
                            self.health_status = None;
                        }
                    }
                }
                AsyncMsg::PredictionResult(res) => {
                    self.is_loading = false;
                    self.is_switching_cam = false;
                    match res {
                        Ok(pred) => {
                            self.selected_pathology_label = pred
                                .gradcam
                                .as_ref()
                                .and_then(|g| g.target_label.clone());
                            self.prediction = Some(pred);
                        }
                        Err(e) => {
                            self.error_message = Some(e);
                        }
                    }
                }
            }
        }
    }

    fn ensure_textures(&mut self, ctx: &egui::Context) {
        // 1. App icon texture (from high-res processed icon.jpeg)
        if self.icon_texture.is_none() {
            if let Ok(img) = image::load_from_memory(APP_ICON_PNG) {
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.to_rgba8().into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                self.icon_texture = Some(ctx.load_texture("app_icon", color_image, Default::default()));
            }
        }

        // 2. CXR Image texture
        let current_cxr_bytes = self
            .prediction
            .as_ref()
            .and_then(|p| p.gradcam.as_ref())
            .and_then(|g| g.original_clahe_base64.as_ref())
            .and_then(|b64| Base64Helper::clean_decode(b64))
            .or_else(|| self.selected_image_bytes.clone());

        if let Some(bytes) = current_cxr_bytes {
            let hash = bytes.len() as u64 ^ (bytes.first().copied().unwrap_or(0) as u64);
            if self.cxr_texture.is_none() || self.active_cxr_hash != hash {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let size = [img.width() as usize, img.height() as usize];
                    let pixels = img.to_rgba8().into_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    self.cxr_texture =
                        Some(ctx.load_texture("cxr_viewport", color_image, Default::default()));
                    self.active_cxr_hash = hash;
                }
            }
        } else {
            self.cxr_texture = None;
            self.active_cxr_hash = 0;
        }

        // 3. Grad-CAM Heatmap texture
        let current_heatmap_bytes = self
            .prediction
            .as_ref()
            .and_then(|p| p.gradcam.as_ref())
            .and_then(|g| g.heatmap_only_base64.as_ref())
            .and_then(|b64| Base64Helper::clean_decode(b64));

        if let Some(bytes) = current_heatmap_bytes {
            let hash = bytes.len() as u64 ^ (bytes.first().copied().unwrap_or(0) as u64);
            if self.heatmap_texture.is_none() || self.active_heatmap_hash != hash {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let size = [img.width() as usize, img.height() as usize];
                    let pixels = img.to_rgba8().into_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    self.heatmap_texture =
                        Some(ctx.load_texture("gradcam_heatmap", color_image, Default::default()));
                    self.active_heatmap_hash = hash;
                }
            }
        } else {
            self.heatmap_texture = None;
            self.active_heatmap_hash = 0;
        }
    }
}

impl eframe::App for BmiCxrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_async_messages();
        self.ensure_textures(ctx);

        let screen_width = ctx.screen_rect().width();
        let is_desktop = screen_width >= 1050.0;

        // Top App Bar
        egui::TopBottomPanel::top("top_header_panel")
            .frame(
                Frame::none()
                    .fill(MedicalColors::SURFACE)
                    .inner_margin(Margin::symmetric(16.0, 10.0))
                    .stroke(Stroke::new(1.0f32, MedicalColors::BORDER)),
            )
            .show(ctx, |ui| {
                self.ui_app_bar(ui);
            });

        // Bottom Google Play / Legal Disclaimer Bar
        egui::TopBottomPanel::bottom("bottom_disclaimer_panel")
            .frame(
                Frame::none()
                    .fill(MedicalColors::SURFACE)
                    .inner_margin(Margin::symmetric(14.0, 8.0))
                    .stroke(Stroke::new(1.0f32, MedicalColors::BORDER)),
            )
            .show(ctx, |ui| {
                self.ui_footer_disclaimer(ui);
            });

        // Mobile Bottom Navigation Bar (if on compact/mobile screen)
        if !is_desktop {
            egui::TopBottomPanel::bottom("mobile_nav_panel")
                .frame(
                    Frame::none()
                        .fill(MedicalColors::SURFACE_ELEVATED)
                        .inner_margin(Margin::symmetric(12.0, 6.0))
                        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER)),
                )
                .show(ctx, |ui| {
                    self.ui_mobile_navbar(ui);
                });
        }

        // Central Body
        egui::CentralPanel::default()
            .frame(Frame::none().fill(MedicalColors::BACKGROUND).inner_margin(Margin::same(12.0)))
            .show(ctx, |ui| {
                if is_desktop {
                    self.ui_desktop_layout(ui);
                } else {
                    self.ui_mobile_layout(ui);
                }
            });

        // Modals
        if self.show_disclaimer_modal {
            self.ui_disclaimer_dialog(ctx);
        }
        if self.show_server_modal {
            self.ui_server_config_dialog(ctx);
        }
    }
}

// UI Subcomponents
impl BmiCxrApp {
    fn ui_app_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // App Icon (from icon.jpeg)
            if let Some(tex) = &self.icon_texture {
                let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, Rounding::same(8.0), MedicalColors::SURFACE_ELEVATED);
                painter.rect_stroke(rect, Rounding::same(8.0), Stroke::new(1.0f32, MedicalColors::PRIMARY_CYAN));
                painter.image(
                    tex.id(),
                    rect.shrink(1.0),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                ui.label(RichText::new("CXR").size(18.0).strong().color(MedicalColors::PRIMARY_CYAN));
            }
            ui.add_space(10.0);

            // Title & Subtitle
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(CdssDisclaimer::APP_TITLE)
                        .size(16.0)
                        .strong()
                        .color(MedicalColors::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("DenseNet-121 Multimodal CDSS (Macro-AUROC 0.777)")
                        .size(11.5)
                        .color(MedicalColors::PRIMARY_CYAN),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Info CDSS Disclaimer Button
                let (info_rect, info_resp) = ui.allocate_exact_size(vec2(86.0, 32.0), Sense::click());
                let info_hovered = info_resp.hovered();
                let info_painter = ui.painter_at(info_rect);
                let info_bg = if info_hovered { MedicalColors::SURFACE_ELEVATED } else { MedicalColors::SURFACE };
                info_painter.rect(
                    info_rect,
                    Rounding::same(16.0),
                    info_bg,
                    Stroke::new(1.0f32, if info_hovered { MedicalColors::PRIMARY_CYAN } else { MedicalColors::BORDER_LIGHT }),
                );
                info_painter.circle_stroke(
                    Pos2::new(info_rect.min.x + 16.0, info_rect.center().y),
                    6.0,
                    Stroke::new(1.2f32, MedicalColors::TEXT_SECONDARY),
                );
                info_painter.text(
                    Pos2::new(info_rect.min.x + 16.0, info_rect.center().y),
                    Align2::CENTER_CENTER,
                    "i",
                    FontId::monospace(10.0),
                    MedicalColors::TEXT_SECONDARY,
                );
                info_painter.text(
                    Pos2::new(info_rect.min.x + 28.0, info_rect.center().y),
                    Align2::LEFT_CENTER,
                    "CDSS",
                    FontId::proportional(12.0),
                    MedicalColors::TEXT_PRIMARY,
                );
                if info_resp.clicked() {
                    self.show_disclaimer_modal = true;
                }

                ui.add_space(8.0);

                // Server Status Pill (Vector dot + Clean text)
                let (status_color, status_text) = if self.offline_demo_mode {
                    (MedicalColors::RISK_MODERATE_LOW, "Oflayn Demo")
                } else if self.is_connected {
                    (MedicalColors::RISK_NORMAL, "Server Aloqada")
                } else {
                    (MedicalColors::RISK_HIGH, "Aloqa Yo'q")
                };

                let (pill_rect, pill_resp) = ui.allocate_exact_size(vec2(136.0, 32.0), Sense::click());
                let pill_hovered = pill_resp.hovered();
                let pill_painter = ui.painter_at(pill_rect);
                let pill_bg = if pill_hovered { MedicalColors::SURFACE_ELEVATED } else { MedicalColors::SURFACE };
                pill_painter.rect(
                    pill_rect,
                    Rounding::same(16.0),
                    pill_bg,
                    Stroke::new(1.2f32, status_color),
                );
                // Glowing dot
                pill_painter.circle_filled(
                    Pos2::new(pill_rect.min.x + 16.0, pill_rect.center().y),
                    4.5,
                    status_color,
                );
                pill_painter.text(
                    Pos2::new(pill_rect.min.x + 27.0, pill_rect.center().y),
                    Align2::LEFT_CENTER,
                    status_text,
                    FontId::proportional(12.0),
                    status_color,
                );
                if pill_resp.clicked() {
                    self.show_server_modal = true;
                }
            });
        });
    }

    fn ui_footer_disclaimer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Drawn medical cross/shield badge
            let (badge_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            let painter = ui.painter_at(badge_rect);
            painter.circle_stroke(badge_rect.center(), 7.0, Stroke::new(1.2f32, MedicalColors::PRIMARY_CYAN));
            painter.text(
                badge_rect.center(),
                Align2::CENTER_CENTER,
                "✓",
                FontId::proportional(9.0),
                MedicalColors::PRIMARY_CYAN,
            );

            ui.add_space(6.0);
            let disclaimer_text = self
                .prediction
                .as_ref()
                .and_then(|p| p.disclaimer.as_deref())
                .unwrap_or(CdssDisclaimer::SHORT_FOOTER_DISCLAIMER_UZ);
            ui.label(
                RichText::new(disclaimer_text)
                    .size(11.0)
                    .color(MedicalColors::TEXT_SECONDARY),
            );
        });
    }

    fn ui_mobile_navbar(&mut self, ui: &mut Ui) {
        ui.columns(3, |cols| {
            cols[0].vertical_centered(|ui| {
                let sel = self.mobile_tab == 0;
                let color = if sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::TEXT_MUTED };
                if ui
                    .add_sized(
                        vec2(ui.available_width(), 36.0),
                        egui::Button::new(RichText::new("Bemor & Rasm").size(12.5).strong().color(color))
                            .fill(if sel { MedicalColors::SURFACE } else { Color32::TRANSPARENT }),
                    )
                    .clicked()
                {
                    self.mobile_tab = 0;
                }
            });
            cols[1].vertical_centered(|ui| {
                let sel = self.mobile_tab == 1;
                let color = if sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::TEXT_MUTED };
                if ui
                    .add_sized(
                        vec2(ui.available_width(), 36.0),
                        egui::Button::new(RichText::new("Rentgen & CAM").size(12.5).strong().color(color))
                            .fill(if sel { MedicalColors::SURFACE } else { Color32::TRANSPARENT }),
                    )
                    .clicked()
                {
                    self.mobile_tab = 1;
                }
            });
            cols[2].vertical_centered(|ui| {
                let sel = self.mobile_tab == 2;
                let color = if sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::TEXT_MUTED };
                if ui
                    .add_sized(
                        vec2(ui.available_width(), 36.0),
                        egui::Button::new(RichText::new("14 Patologiya").size(12.5).strong().color(color))
                            .fill(if sel { MedicalColors::SURFACE } else { Color32::TRANSPARENT }),
                    )
                    .clicked()
                {
                    self.mobile_tab = 2;
                }
            });
        });
    }

    fn ui_desktop_layout(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            // Column 1: Patient Input Form (width: 325)
            ui.allocate_ui_with_layout(
                vec2(325.0, ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| {
                    Frame::none()
                        .fill(MedicalColors::SURFACE)
                        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::same(14.0))
                        .show(ui, |ui| {
                            ScrollArea::vertical().id_salt("desktop_input_scroll").show(ui, |ui| {
                                self.ui_patient_input_form(ui);
                            });
                        });
                },
            );

            ui.add_space(10.0);

            // Column 2: Center Viewport + Clinical Summary
            let center_width = (ui.available_width() - 395.0).max(400.0);
            ui.allocate_ui_with_layout(
                vec2(center_width, ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| {
                    let viewport_height = if self.prediction.is_some() {
                        (ui.available_height() - 170.0).max(280.0)
                    } else {
                        ui.available_height()
                    };

                    Frame::none()
                        .fill(MedicalColors::SURFACE)
                        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::same(0.0))
                        .show(ui, |ui| {
                            ui.set_height(viewport_height);
                            self.ui_xray_viewport(ui);
                        });

                    if let Some(pred) = &self.prediction {
                        ui.add_space(10.0);
                        self.ui_clinical_summary_banner(ui, pred);
                    }
                },
            );

            ui.add_space(10.0);

            // Column 3: 14 Pathology Probability Cards (width: 380)
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| {
                    Frame::none()
                        .fill(MedicalColors::SURFACE)
                        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::same(12.0))
                        .show(ui, |ui| {
                            self.ui_pathology_list_view(ui);
                        });
                },
            );
        });
    }

    fn ui_mobile_layout(&mut self, ui: &mut Ui) {
        match self.mobile_tab {
            0 => {
                Frame::none()
                    .fill(MedicalColors::SURFACE)
                    .stroke(Stroke::new(1.0f32, MedicalColors::BORDER))
                    .rounding(Rounding::same(10.0))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ScrollArea::vertical().id_salt("mobile_input_scroll").show(ui, |ui| {
                            self.ui_patient_input_form(ui);
                        });
                    });
            }
            1 => {
                ui.vertical(|ui| {
                    let h = if self.prediction.is_some() {
                        (ui.available_height() - 170.0).max(240.0)
                    } else {
                        ui.available_height()
                    };
                    Frame::none()
                        .fill(MedicalColors::SURFACE)
                        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER))
                        .rounding(Rounding::same(10.0))
                        .show(ui, |ui| {
                            ui.set_height(h);
                            self.ui_xray_viewport(ui);
                        });

                    if let Some(pred) = &self.prediction {
                        ui.add_space(8.0);
                        self.ui_clinical_summary_banner(ui, pred);
                    }
                });
            }
            _ => {
                Frame::none()
                    .fill(MedicalColors::SURFACE)
                    .stroke(Stroke::new(1.0f32, MedicalColors::BORDER))
                    .rounding(Rounding::same(10.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        self.ui_pathology_list_view(ui);
                    });
            }
        }
    }

    fn ui_patient_input_form(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("RENTGENOGRAMMA")
                    .size(11.5)
                    .strong()
                    .color(MedicalColors::PRIMARY_CYAN),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(h) = &self.health_status {
                    Frame::none()
                        .fill(MedicalColors::SURFACE_ELEVATED)
                        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER_LIGHT))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(h.device.to_uppercase())
                                    .size(10.5)
                                    .strong()
                                    .color(MedicalColors::PRIMARY_CYAN),
                            );
                        });
                }
            });
        });
        ui.add_space(8.0);

        // Upload Container Box
        let upload_box = Frame::none()
            .fill(MedicalColors::SURFACE_ELEVATED)
            .stroke(Stroke::new(1.2f32, MedicalColors::BORDER_LIGHT))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(14.0));

        upload_box.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                // Drawn folder icon
                let (box_rect, _) = ui.allocate_exact_size(vec2(36.0, 30.0), Sense::hover());
                let p = ui.painter_at(box_rect);
                p.rect(
                    box_rect,
                    Rounding::same(4.0),
                    Color32::from_rgba_premultiplied(0, 229, 255, 25),
                    Stroke::new(1.2f32, MedicalColors::PRIMARY_CYAN),
                );
                p.text(
                    box_rect.center(),
                    Align2::CENTER_CENTER,
                    "CXR",
                    FontId::monospace(10.0),
                    MedicalColors::PRIMARY_CYAN,
                );

                ui.add_space(8.0);
                ui.label(
                    RichText::new(&self.selected_image_name)
                        .size(12.5)
                        .strong()
                        .color(MedicalColors::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("PNG / JPG / DICOM tasvirlar qo'llab-quvvatlanadi")
                        .size(10.5)
                        .color(MedicalColors::TEXT_MUTED),
                );
                ui.add_space(10.0);

                let pick_btn = egui::Button::new(
                    RichText::new("Faylni Tanlash...")
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(MedicalColors::SURFACE)
                .stroke(Stroke::new(1.2f32, MedicalColors::PRIMARY_CYAN))
                .rounding(Rounding::same(6.0));

                if ui.add_sized(vec2(180.0, 36.0), pick_btn).clicked() {
                    self.pick_image_file();
                }
            });
        });

        ui.add_space(10.0);

        // Load Sample Button (Enlarged, solid)
        let sample_btn = egui::Button::new(
            RichText::new("Namuna Rentgenni Yuklash (Demo)")
                .size(12.5)
                .strong()
                .color(MedicalColors::PRIMARY_CYAN),
        )
        .fill(MedicalColors::SURFACE_ELEVATED)
        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER_LIGHT))
        .rounding(Rounding::same(6.0));

        if ui.add_sized(vec2(ui.available_width(), 38.0), sample_btn).clicked() {
            self.load_sample_radiograph();
        }

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);

        // Patient Metadata Section
        ui.label(
            RichText::new("KLINIK METAMA'LUMOTLAR")
                .size(11.5)
                .strong()
                .color(MedicalColors::PRIMARY_CYAN),
        );
        ui.add_space(8.0);

        // View Position (PA vs AP) Segmented Button
        ui.label(RichText::new("Proyeksiya (View Pos):").size(12.0).color(MedicalColors::TEXT_SECONDARY));
        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 8.0) / 2.0;
            let pa_sel = self.view_pos == "PA";
            let ap_sel = self.view_pos == "AP";

            let pa_btn = egui::Button::new(
                RichText::new("PA")
                    .size(13.0)
                    .strong()
                    .color(if pa_sel { Color32::BLACK } else { MedicalColors::TEXT_PRIMARY }),
            )
            .fill(if pa_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::SURFACE_ELEVATED })
            .stroke(Stroke::new(1.0f32, if pa_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::BORDER }))
            .rounding(Rounding::same(6.0));
            if ui.add_sized(vec2(half_w, 34.0), pa_btn).clicked() {
                self.view_pos = "PA".to_string();
            }

            let ap_btn = egui::Button::new(
                RichText::new("AP")
                    .size(13.0)
                    .strong()
                    .color(if ap_sel { Color32::BLACK } else { MedicalColors::TEXT_PRIMARY }),
            )
            .fill(if ap_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::SURFACE_ELEVATED })
            .stroke(Stroke::new(1.0f32, if ap_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::BORDER }))
            .rounding(Rounding::same(6.0));
            if ui.add_sized(vec2(half_w, 34.0), ap_btn).clicked() {
                self.view_pos = "AP".to_string();
            }
        });

        ui.add_space(10.0);

        // Gender (Erkak vs Ayol) Segmented Button
        ui.label(RichText::new("Bemor Jinsi (Gender):").size(12.0).color(MedicalColors::TEXT_SECONDARY));
        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 8.0) / 2.0;
            let m_sel = self.gender == "M";
            let f_sel = self.gender == "F";

            let m_btn = egui::Button::new(
                RichText::new("Erkak (M)")
                    .size(13.0)
                    .strong()
                    .color(if m_sel { Color32::BLACK } else { MedicalColors::TEXT_PRIMARY }),
            )
            .fill(if m_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::SURFACE_ELEVATED })
            .stroke(Stroke::new(1.0f32, if m_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::BORDER }))
            .rounding(Rounding::same(6.0));
            if ui.add_sized(vec2(half_w, 34.0), m_btn).clicked() {
                self.gender = "M".to_string();
            }

            let f_btn = egui::Button::new(
                RichText::new("Ayol (F)")
                    .size(13.0)
                    .strong()
                    .color(if f_sel { Color32::BLACK } else { MedicalColors::TEXT_PRIMARY }),
            )
            .fill(if f_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::SURFACE_ELEVATED })
            .stroke(Stroke::new(1.0f32, if f_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::BORDER }))
            .rounding(Rounding::same(6.0));
            if ui.add_sized(vec2(half_w, 34.0), f_btn).clicked() {
                self.gender = "F".to_string();
            }
        });

        ui.add_space(12.0);

        // Age Slider (1-100)
        ui.horizontal(|ui| {
            ui.label(RichText::new("Bemor Yoshi (Age):").size(12.0).color(MedicalColors::TEXT_SECONDARY));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Frame::none()
                    .fill(MedicalColors::SURFACE_ELEVATED)
                    .stroke(Stroke::new(1.0f32, MedicalColors::BORDER_LIGHT))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(8.0, 3.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("{} yosh", self.age))
                                .size(12.5)
                                .strong()
                                .color(MedicalColors::PRIMARY_CYAN),
                        );
                    });
            });
        });
        ui.add_space(4.0);
        ui.add(egui::Slider::new(&mut self.age, 1..=100).show_value(false));

        ui.add_space(18.0);

        // Run Diagnosis Action Button (Large, glowing)
        let btn_text = if self.is_loading {
            "Multimodal Tahlil Qilinmoqda..."
        } else {
            "Tahlilni Boshlash (CDSS)"
        };

        let btn_resp = ui.add_sized(
            vec2(ui.available_width(), 48.0),
            egui::Button::new(
                RichText::new(btn_text)
                    .size(14.0)
                    .strong()
                    .color(Color32::BLACK),
            )
            .fill(MedicalColors::PRIMARY_CYAN)
            .rounding(Rounding::same(8.0)),
        );

        if btn_resp.clicked() && !self.is_loading {
            self.run_diagnosis(None);
        }

        if let Some(err) = &self.error_message {
            ui.add_space(10.0);
            Frame::none()
                .fill(Color32::from_rgba_premultiplied(255, 42, 85, 30))
                .stroke(Stroke::new(1.0f32, MedicalColors::RISK_HIGH))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(err).size(11.5).color(MedicalColors::RISK_HIGH));
                });
        }
    }

    fn ui_xray_viewport(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Viewport Top Toolbar
            Frame::none()
                .fill(MedicalColors::SURFACE_ELEVATED)
                .inner_margin(Margin::symmetric(14.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let target_name = self
                            .prediction
                            .as_ref()
                            .and_then(|p| p.gradcam.as_ref())
                            .and_then(|g| g.target_name_uz.as_deref().or(g.target_label.as_deref()))
                            .unwrap_or("Rentgenogramma");

                        // Grad-CAM Tag
                        Frame::none()
                            .fill(Color32::from_rgba_premultiplied(0, 229, 255, 25))
                            .stroke(Stroke::new(1.0f32, MedicalColors::PRIMARY_CYAN))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("Grad-CAM++")
                                        .size(10.5)
                                        .strong()
                                        .color(MedicalColors::PRIMARY_CYAN),
                                );
                            });

                        ui.label(
                            RichText::new(format!(": {}", target_name))
                                .size(13.0)
                                .strong()
                                .color(MedicalColors::TEXT_PRIMARY),
                        );

                        if self.is_switching_cam {
                            ui.spinner();
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // Zoom / Pan controls (Enlarged, clean)
                            if ui
                                .add_sized(
                                    vec2(52.0, 28.0),
                                    egui::Button::new(RichText::new("Tiklash").size(11.0)),
                                )
                                .clicked()
                            {
                                self.zoom_scale = 1.0;
                                self.pan_offset = Vec2::ZERO;
                            }
                            if ui
                                .add_sized(
                                    vec2(30.0, 28.0),
                                    egui::Button::new(RichText::new("-").size(14.0).strong()),
                                )
                                .clicked()
                            {
                                self.zoom_scale = (self.zoom_scale * 0.8).max(0.5);
                            }
                            if ui
                                .add_sized(
                                    vec2(30.0, 28.0),
                                    egui::Button::new(RichText::new("+").size(14.0).strong()),
                                )
                                .clicked()
                            {
                                self.zoom_scale = (self.zoom_scale * 1.25).min(5.0);
                            }

                            if let Some(pred) = &self.prediction {
                                if let Some(ratio) = pred.gradcam.as_ref().and_then(|g| g.lung_focus_ratio) {
                                    Frame::none()
                                        .fill(Color32::from_rgba_premultiplied(0, 229, 255, 20))
                                        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER_LIGHT))
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(Margin::symmetric(6.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("O'pka fokusi: {:.1}%", ratio * 100.0))
                                                    .size(11.0)
                                                    .strong()
                                                    .color(MedicalColors::PRIMARY_CYAN),
                                            );
                                        });
                                }
                            }
                        });
                    });
                });

            // Viewport Interactive Canvas
            let canvas_rect = ui.available_rect_before_wrap();
            let canvas_height = (canvas_rect.height() - 44.0).max(180.0);
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), canvas_height), Sense::drag());

            // Handle Panning with drag
            if response.dragged() {
                self.pan_offset += response.drag_delta();
            }

            // Handle Zoom with mouse scroll
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if response.hovered() && scroll_delta.abs() > 0.0 {
                let factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                self.zoom_scale = (self.zoom_scale * factor).clamp(0.5, 5.0);
            }

            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, Rounding::ZERO, Color32::BLACK);

            // Compute centered, scaled, panned display rect
            if let Some(cxr_tex) = &self.cxr_texture {
                let tex_size = cxr_tex.size_vec2();
                let fit_scale = (rect.width() / tex_size.x).min(rect.height() / tex_size.y) * 0.95;
                let draw_size = tex_size * fit_scale * self.zoom_scale;
                let center = rect.center() + self.pan_offset;
                let draw_rect = Rect::from_center_size(center, draw_size);

                // Draw Base CXR Image
                painter.image(
                    cxr_tex.id(),
                    draw_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Draw Overlaid Grad-CAM++ Heatmap if available
                if let Some(heat_tex) = &self.heatmap_texture {
                    let alpha = (self.gradcam_opacity * 255.0).clamp(0.0, 255.0) as u8;
                    let tint = Color32::from_rgba_premultiplied(alpha, alpha, alpha, alpha);
                    painter.image(
                        heat_tex.id(),
                        draw_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        tint,
                    );
                }
            } else {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "Rentgen tasviri yuklanmagan",
                    FontId::proportional(14.0),
                    MedicalColors::TEXT_MUTED,
                );
            }

            // Bottom Grad-CAM Opacity Slider Bar
            Frame::none()
                .fill(MedicalColors::SURFACE_ELEVATED)
                .inner_margin(Margin::symmetric(14.0, 8.0))
                .stroke(Stroke::new(1.0f32, MedicalColors::BORDER))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Grad-CAM++ Shaffofligi:")
                                .size(12.0)
                                .strong()
                                .color(MedicalColors::PRIMARY_CYAN),
                        );
                        ui.label(
                            RichText::new(format!("{:.0}%", self.gradcam_opacity * 100.0))
                                .size(12.0)
                                .strong()
                                .color(MedicalColors::TEXT_PRIMARY),
                        );

                        ui.add(
                            egui::Slider::new(&mut self.gradcam_opacity, 0.0..=1.0)
                                .show_value(false),
                        );

                        let toggle_lbl = if self.gradcam_opacity > 0.05 {
                            "Yashirish"
                        } else {
                            "Ko'rsatish"
                        };
                        if ui
                            .add_sized(
                                vec2(80.0, 28.0),
                                egui::Button::new(RichText::new(toggle_lbl).size(11.5).color(MedicalColors::PRIMARY_CYAN)),
                            )
                            .clicked()
                        {
                            self.gradcam_opacity = if self.gradcam_opacity > 0.05 { 0.0 } else { 0.65 };
                        }
                    });
                });
        });
    }

    fn ui_clinical_summary_banner(&self, ui: &mut Ui, pred: &PredictionResponse) {
        let is_normal = pred.is_normal;
        let risk_color = MedicalColors::for_risk(Some(&pred.overall_risk), !is_normal);

        Frame::none()
            .fill(MedicalColors::SURFACE_ELEVATED)
            .stroke(Stroke::new(1.2f32, risk_color))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    // Risk badge
                    Frame::none()
                        .fill(Color32::from_rgba_premultiplied(
                            risk_color.r() / 4,
                            risk_color.g() / 4,
                            risk_color.b() / 4,
                            60,
                        ))
                        .stroke(Stroke::new(1.0f32, risk_color))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            let icon_txt = if is_normal { "[NORMA]" } else { "[DIQQAT]" };
                            ui.label(
                                RichText::new(format!("{} XAVF DARAJASI: {}", icon_txt, pred.overall_risk))
                                    .size(12.0)
                                    .strong()
                                    .color(risk_color),
                            );
                        });

                    // Inference time
                    ui.label(
                        RichText::new(format!("Vaqt: {} ms", pred.inference_ms))
                            .size(11.5)
                            .color(MedicalColors::TEXT_SECONDARY),
                    );

                    // Normal confidence
                    ui.label(
                        RichText::new(format!("Norma ishonchi: {:.1}%", pred.no_finding_confidence * 100.0))
                            .size(11.5)
                            .color(MedicalColors::TEXT_SECONDARY),
                    );
                });

                ui.add_space(8.0);
                ui.label(
                    RichText::new(&pred.clinical_summary)
                        .size(12.0)
                        .color(MedicalColors::TEXT_PRIMARY),
                );
            });
    }

    fn ui_pathology_list_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("14 TA PATOLOGIYA EHTIMOLLIGI")
                    .size(12.0)
                    .strong()
                    .color(MedicalColors::PRIMARY_CYAN),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(pred) = &self.prediction {
                    let pos_count = pred.positive_findings.len();
                    if pos_count > 0 {
                        Frame::none()
                            .fill(Color32::from_rgba_premultiplied(255, 42, 85, 40))
                            .stroke(Stroke::new(1.0f32, MedicalColors::RISK_HIGH))
                            .rounding(Rounding::same(10.0))
                            .inner_margin(Margin::symmetric(8.0, 3.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("{} musbat", pos_count))
                                        .size(11.0)
                                        .strong()
                                        .color(MedicalColors::RISK_HIGH),
                                );
                            });
                    }
                }
            });
        });

        ui.add_space(8.0);

        // Filter chips (Enlarged)
        ui.horizontal(|ui| {
            let all_sel = !self.only_positive_filter;
            let all_btn = egui::Button::new(
                RichText::new("Barchasi (14)")
                    .size(11.5)
                    .color(if all_sel { Color32::BLACK } else { MedicalColors::TEXT_PRIMARY }),
            )
            .fill(if all_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::SURFACE_ELEVATED })
            .stroke(Stroke::new(1.0f32, if all_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::BORDER }))
            .rounding(Rounding::same(14.0));

            if ui.add_sized(vec2(100.0, 28.0), all_btn).clicked() {
                self.only_positive_filter = false;
            }

            let pos_sel = self.only_positive_filter;
            let pos_btn = egui::Button::new(
                RichText::new("Faqat Musbat")
                    .size(11.5)
                    .color(if pos_sel { Color32::BLACK } else { MedicalColors::TEXT_PRIMARY }),
            )
            .fill(if pos_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::SURFACE_ELEVATED })
            .stroke(Stroke::new(1.0f32, if pos_sel { MedicalColors::PRIMARY_CYAN } else { MedicalColors::BORDER }))
            .rounding(Rounding::same(14.0));

            if ui.add_sized(vec2(100.0, 28.0), pos_btn).clicked() {
                self.only_positive_filter = true;
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        // Build list of items
        let mut items: Vec<PathologyItem> = if let Some(pred) = &self.prediction {
            pred.all_findings.clone()
        } else {
            ALL_PATHOLOGIES
                .iter()
                .map(|m| PathologyItem {
                    index: m.index,
                    label: m.label.to_string(),
                    name_uz: m.name_uz.to_string(),
                    description_uz: m.description_uz.to_string(),
                    prob: 0.0,
                    prob_percent: 0.0,
                    threshold: m.threshold,
                    positive: false,
                    risk_code: "LOW".to_string(),
                    risk_level: "Kutilmoqda".to_string(),
                })
                .collect()
        };

        // Sort: positive first, then descending prob
        items.sort_by(|a, b| {
            if a.positive && !b.positive {
                std::cmp::Ordering::Less
            } else if !a.positive && b.positive {
                std::cmp::Ordering::Greater
            } else {
                b.prob.partial_cmp(&a.prob).unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        let target_label = self
            .selected_pathology_label
            .clone()
            .or_else(|| self.prediction.as_ref().and_then(|p| p.gradcam.as_ref()).and_then(|g| g.target_label.clone()));

        let mut clicked_label = None;

        ScrollArea::vertical().id_salt("pathology_scroll").show(ui, |ui| {
            for item in items.iter() {
                if self.only_positive_filter && !item.positive {
                    continue;
                }

                let is_selected = target_label
                    .as_ref()
                    .map(|l| l.eq_ignore_ascii_case(&item.label))
                    .unwrap_or(false);

                if self.ui_pathology_card(ui, item, is_selected) {
                    clicked_label = Some(item.label.clone());
                }
                ui.add_space(8.0);
            }
        });

        if let Some(lbl) = clicked_label {
            if self.prediction.is_some() {
                self.selected_pathology_label = Some(lbl.clone());
                self.run_diagnosis(Some(lbl));
            }
        }
    }

    fn ui_pathology_card(&self, ui: &mut Ui, item: &PathologyItem, is_selected: bool) -> bool {
        let is_pos = item.positive;
        let border_color = if is_selected {
            MedicalColors::PRIMARY_CYAN
        } else if is_pos {
            MedicalColors::RISK_HIGH
        } else {
            MedicalColors::BORDER
        };

        let bg_color = if is_selected {
            Color32::from_rgba_premultiplied(0, 229, 255, 20)
        } else {
            MedicalColors::SURFACE_ELEVATED
        };

        let card_frame = Frame::none()
            .fill(bg_color)
            .stroke(Stroke::new(if is_selected { 1.5f32 } else { 1.0f32 }, border_color))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(10.0));

        let mut clicked = false;

        card_frame.show(ui, |ui| {
            let resp = ui.interact(ui.max_rect(), ui.id().with(&item.label), Sense::click());
            if resp.clicked() {
                clicked = true;
            }

            ui.vertical(|ui| {
                // Row 1: Status Icon Badge + Name Uz + English label & CAM badge
                ui.horizontal(|ui| {
                    // Vector Drawn Status Icon (No missing font square!)
                    let (badge_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                    let p = ui.painter_at(badge_rect);
                    if is_pos {
                        p.circle_filled(badge_rect.center(), 7.0, MedicalColors::RISK_HIGH);
                        p.text(
                            badge_rect.center(),
                            Align2::CENTER_CENTER,
                            "!",
                            FontId::monospace(10.0),
                            Color32::WHITE,
                        );
                    } else {
                        p.circle_stroke(
                            badge_rect.center(),
                            6.0,
                            Stroke::new(1.2f32, MedicalColors::RISK_LOW),
                        );
                        p.circle_filled(
                            badge_rect.center(),
                            2.5,
                            MedicalColors::RISK_LOW,
                        );
                    }

                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(&item.name_uz)
                            .size(13.0)
                            .strong()
                            .color(if is_pos { Color32::WHITE } else { MedicalColors::TEXT_PRIMARY }),
                    );
                    ui.label(
                        RichText::new(format!("({})", item.label))
                            .size(11.0)
                            .color(MedicalColors::TEXT_SECONDARY),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if is_selected {
                            Frame::none()
                                .fill(MedicalColors::PRIMARY_CYAN)
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Grad-CAM").size(10.0).strong().color(Color32::BLACK));
                                });
                        } else {
                            ui.label(RichText::new("CAM ko'rish").size(10.0).color(MedicalColors::PRIMARY_CYAN));
                        }
                    });
                });

                ui.add_space(6.0);

                // Row 2: Metrics (% vs Threshold)
                ui.horizontal(|ui| {
                    let prob_color = if is_pos { MedicalColors::RISK_HIGH } else { MedicalColors::TEXT_PRIMARY };
                    ui.label(
                        RichText::new(format!("{:.1}%", item.prob_percent))
                            .size(13.5)
                            .strong()
                            .color(prob_color),
                    );

                    Frame::none()
                        .fill(MedicalColors::SURFACE)
                        .stroke(Stroke::new(1.0f32, MedicalColors::BORDER_LIGHT))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("Chegara: {:.0}%", item.threshold * 100.0))
                                    .size(10.5)
                                    .color(MedicalColors::TEXT_SECONDARY),
                            );
                        });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let level_color = if is_pos { MedicalColors::RISK_HIGH } else { MedicalColors::TEXT_SECONDARY };
                        ui.label(RichText::new(&item.risk_level).size(11.0).color(level_color));
                    });
                });

                ui.add_space(6.0);

                // Row 3: Custom Progress Bar with White Vertical Threshold Mark
                let (bar_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 13.0), Sense::hover());
                let painter = ui.painter_at(bar_rect);

                // Bar Background
                painter.rect_filled(bar_rect, Rounding::same(6.0), Color32::from_rgb(16, 25, 40));

                // Filled Bar
                let fill_width = (bar_rect.width() * item.prob.clamp(0.0, 1.0)).max(0.0);
                if fill_width > 0.0 {
                    let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_width, bar_rect.height()));
                    let fill_color = if is_pos {
                        if item.risk_code == "HIGH" { MedicalColors::RISK_HIGH } else { MedicalColors::RISK_MODERATE_HIGH }
                    } else if item.prob >= item.threshold * 0.75 {
                        MedicalColors::RISK_MODERATE_LOW
                    } else {
                        MedicalColors::PRIMARY_TEAL
                    };
                    painter.rect_filled(fill_rect, Rounding::same(6.0), fill_color);
                }

                // White Vertical Threshold Tick Line
                let tick_x = bar_rect.min.x + bar_rect.width() * item.threshold.clamp(0.0, 1.0);
                painter.line_segment(
                    [Pos2::new(tick_x, bar_rect.min.y - 3.0), Pos2::new(tick_x, bar_rect.max.y + 3.0)],
                    Stroke::new(2.0f32, Color32::WHITE),
                );
            });
        });

        clicked
    }

    fn ui_disclaimer_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new(CdssDisclaimer::DIALOG_TITLE_UZ)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .fixed_size(vec2(500.0, 380.0))
            .show(ctx, |ui| {
                ScrollArea::vertical().max_height(290.0).show(ui, |ui| {
                    ui.label(RichText::new(CdssDisclaimer::FULL_DISCLAIMER_UZ).size(12.5));
                });

                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_sized(
                                vec2(220.0, 36.0),
                                egui::Button::new(
                                    RichText::new("Tushundim va Roziman")
                                        .size(13.0)
                                        .strong()
                                        .color(Color32::BLACK),
                                )
                                .fill(MedicalColors::PRIMARY_CYAN)
                                .rounding(Rounding::same(6.0)),
                            )
                            .clicked()
                        {
                            self.is_disclaimer_accepted = true;
                            self.show_disclaimer_modal = false;
                        }
                    });
                });
            });
    }

    fn ui_server_config_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Server va Ulanish Sozlamalari")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .fixed_size(vec2(460.0, 250.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("DenseNet-121 Flask / PyTorch Server URL:").size(12.5));
                ui.add(egui::TextEdit::singleline(&mut self.server_url).desired_width(ui.available_width()));

                ui.add_space(10.0);
                ui.checkbox(&mut self.offline_demo_mode, "Oflayn Namuna Rejimi (Demo Mode)");

                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            vec2(160.0, 32.0),
                            egui::Button::new(RichText::new("Ulanishni Tekshirish").size(12.0)),
                        )
                        .clicked()
                    {
                        self.check_server_health();
                    }

                    if self.is_checking_connection {
                        ui.label(RichText::new("Tekshirilmoqda...").size(11.5).color(MedicalColors::PRIMARY_CYAN));
                    } else if self.is_connected {
                        ui.label(RichText::new("Aloqa muvaffaqiyatli").size(11.5).color(MedicalColors::RISK_NORMAL));
                    } else {
                        ui.label(RichText::new("Server topilmadi").size(11.5).color(MedicalColors::RISK_HIGH));
                    }
                });

                ui.add_space(16.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_sized(
                            vec2(100.0, 32.0),
                            egui::Button::new(RichText::new("Yopish").size(12.0).color(MedicalColors::PRIMARY_CYAN)),
                        )
                        .clicked()
                    {
                        self.show_server_modal = false;
                    }
                });
            });
    }
}
