pub mod constants;
pub mod models;
pub mod services;
pub mod app;

pub use app::BmiCxrApp;

pub fn run_desktop() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("BMI CXR-AI v2 - Clinical Decision Support System")
        .with_inner_size([1280.0, 840.0])
        .with_min_inner_size([360.0, 600.0]);

    if let Some(icon) = services::load_app_icon_data() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "BMI CXR-AI v2",
        options,
        Box::new(|cc| Ok(Box::new(BmiCxrApp::new(cc)))),
    )
}


#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    use eframe::NativeOptions;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "BMI CXR-AI v2",
        options,
        Box::new(|cc| Ok(Box::new(BmiCxrApp::new(cc)))),
    );
}
