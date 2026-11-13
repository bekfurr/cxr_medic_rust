use std::time::Duration;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use crate::constants::{find_pathology, CdssDisclaimer, ALL_PATHOLOGIES};
use crate::models::{
    GradcamData, HealthStatus, PathologyItem, PatientMeta, PredictionRequest, PredictionResponse,
};

pub const SAMPLE_CXR_PNG: &[u8] = include_bytes!("../assets/demo/sample_cxr.png");
pub const HEATMAP_EMPHYSEMA_PNG: &[u8] = include_bytes!("../assets/demo/heatmap_emphysema.png");
pub const HEATMAP_CARDIOMEGALY_PNG: &[u8] = include_bytes!("../assets/demo/heatmap_cardiomegaly.png");
pub const HEATMAP_PNEUMONIA_PNG: &[u8] = include_bytes!("../assets/demo/heatmap_pneumonia.png");
pub const HEATMAP_EFFUSION_PNG: &[u8] = include_bytes!("../assets/demo/heatmap_effusion.png");
pub const HEATMAP_GENERIC_PNG: &[u8] = include_bytes!("../assets/demo/heatmap_generic.png");
pub const APP_ICON_PNG: &[u8] = include_bytes!("../assets/icons/app_icon.png");

pub fn load_app_icon_data() -> Option<egui::IconData> {
    let img = image::load_from_memory(APP_ICON_PNG).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = image::GenericImageView::dimensions(&img);
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}


pub fn get_demo_heatmap_bytes(label: Option<&str>) -> &'static [u8] {
    match label.map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("emphysema") => HEATMAP_EMPHYSEMA_PNG,
        Some("cardiomegaly") => HEATMAP_CARDIOMEGALY_PNG,
        Some("pneumonia") => HEATMAP_PNEUMONIA_PNG,
        Some("effusion") => HEATMAP_EFFUSION_PNG,
        _ => HEATMAP_GENERIC_PNG,
    }
}

pub struct Base64Helper;

impl Base64Helper {
    pub fn clean_decode(b64_str: &str) -> Option<Vec<u8>> {
        let clean = b64_str.trim();
        let payload = if let Some(idx) = clean.rfind(',') {
            &clean[idx + 1..]
        } else {
            clean
        };
        let sanitized: String = payload.chars().filter(|c| !c.is_whitespace()).collect();
        BASE64.decode(sanitized).ok()
    }

    pub fn clean_encode(bytes: &[u8]) -> String {
        BASE64.encode(bytes)
    }
}

pub struct ApiService;

impl ApiService {
    fn clean_url(base: &str, path: &str) -> String {
        let trimmed = base.trim().trim_end_matches('/');
        format!("{}{}", trimmed, path)
    }

    pub fn check_health(base_url: &str) -> Result<HealthStatus, String> {
        let url = Self::clean_url(base_url, "/api/v1/health");
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(Duration::from_secs(6))
            .build();

        let response = agent
            .get(&url)
            .call()
            .map_err(|e| format!("Serverga ulanish xatosi: {}", e))?;

        if response.status() == 200 {
            response
                .into_json::<HealthStatus>()
                .map_err(|e| format!("JSON tahlil xatosi: {}", e))
        } else {
            Err(format!("Server holati xatosi ({})", response.status()))
        }
    }

    pub fn predict(base_url: &str, req: &PredictionRequest) -> Result<PredictionResponse, String> {
        let url = Self::clean_url(base_url, "/api/v1/predict");
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(8))
            .timeout_read(Duration::from_secs(45))
            .build();

        let response = agent
            .post(&url)
            .set("Content-Type", "application/json; charset=utf-8")
            .set("Accept", "application/json")
            .send_json(req)
            .map_err(|e| format!("Tahlil so'rovi muvaffaqiyatsiz bo'ldi: {}", e))?;

        if response.status() == 200 {
            response
                .into_json::<PredictionResponse>()
                .map_err(|e| format!("JSON tahlil xatosi: {}", e))
        } else {
            Err(format!("Server tahlil xatosi ({})", response.status()))
        }
    }
}

pub struct MockDiagnosisService;

impl MockDiagnosisService {
    pub fn simulate_diagnosis(req: &PredictionRequest) -> PredictionResponse {
        // Base calibrated mock probabilities
        let mock_probs: [(&str, f32); 14] = [
            ("Atelectasis", 0.1820),
            ("Cardiomegaly", 0.3840),
            ("Effusion", 0.2150),
            ("Infiltration", 0.2640),
            ("Mass", 0.0810),
            ("Nodule", 0.1230),
            ("Pneumonia", 0.1940),
            ("Pneumothorax", 0.0420),
            ("Consolidation", 0.1510),
            ("Edema", 0.1430),
            ("Emphysema", 0.5636),
            ("Fibrosis", 0.1120),
            ("Pleural_Thickening", 0.1650),
            ("Hernia", 0.0240),
        ];

        let mut all_findings = Vec::with_capacity(14);
        let mut positive_findings = Vec::new();

        for meta in ALL_PATHOLOGIES.iter() {
            let mut prob = mock_probs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(meta.label))
                .map(|(_, v)| *v)
                .unwrap_or(meta.threshold * 0.4);

            if meta.label == "Cardiomegaly" && req.age > 60 {
                prob = (prob + 0.05).min(0.95);
            }

            let positive = prob >= meta.threshold;
            let prob_percent = (prob * 1000.0).round() / 10.0;

            let (risk_code, risk_level) = if positive {
                if prob >= meta.threshold + 0.25 {
                    ("HIGH", "Yuqori (Musbat)")
                } else {
                    ("MODERATE_HIGH", "O'rta-Yuqori (Musbat)")
                }
            } else if prob >= meta.threshold - 0.10 {
                ("MODERATE_LOW", "O'rta-Quyi (Chegara)")
            } else {
                ("LOW", "Quyi (Manfiy)")
            };

            let item = PathologyItem {
                index: meta.index,
                label: meta.label.to_string(),
                name_uz: meta.name_uz.to_string(),
                description_uz: meta.description_uz.to_string(),
                prob,
                prob_percent,
                threshold: meta.threshold,
                positive,
                risk_code: risk_code.to_string(),
                risk_level: risk_level.to_string(),
            };

            all_findings.push(item.clone());
            if positive {
                positive_findings.push(item);
            }
        }

        // Sort positive findings descending
        positive_findings.sort_by(|a, b| b.prob.partial_cmp(&a.prob).unwrap());

        let mut top_findings = all_findings.clone();
        top_findings.sort_by(|a, b| b.prob.partial_cmp(&a.prob).unwrap());
        top_findings.truncate(4);

        let target_label = req.target_label.clone().unwrap_or_else(|| {
            if !positive_findings.is_empty() {
                positive_findings[0].label.clone()
            } else {
                "Emphysema".to_string()
            }
        });

        let target_meta = find_pathology(&target_label).unwrap_or(&ALL_PATHOLOGIES[10]);
        let heatmap_bytes = get_demo_heatmap_bytes(Some(&target_label));
        let heatmap_b64 = Base64Helper::clean_encode(heatmap_bytes);

        let gradcam = GradcamData {
            target_label: Some(target_meta.label.to_string()),
            target_name_uz: Some(target_meta.name_uz.to_string()),
            lung_focus_ratio: Some(0.9142),
            original_clahe_base64: Some(req.image_base64.clone()),
            overlay_base64: Some(heatmap_b64.clone()),
            heatmap_only_base64: Some(heatmap_b64),
        };

        let is_normal = positive_findings.is_empty();
        let overall_risk = if is_normal {
            "NORMAL"
        } else if positive_findings.len() >= 2 {
            "HIGH"
        } else {
            "MODERATE"
        };

        let clinical_summary = if is_normal {
            "Norma: 14 ta asosiy o'pka patologiyasi bo'yicha me'yordan og'ish aniqlanmadi (Ishonch darajasi: 92.4%).".to_string()
        } else {
            let names: Vec<String> = positive_findings
                .iter()
                .map(|e| format!("{} ({}%)", e.name_uz, e.prob_percent))
                .collect();
            format!(
                "Diqqat: {} ta ehtimoliy patologiya aniqlandi: {}. Klinik korrelyatsiya va rentgenolog maslahati tavsiya etiladi.",
                positive_findings.len(),
                names.join(", ")
            )
        };

        PredictionResponse {
            status: "success".to_string(),
            request_id: Some("mock_rust_demo_7f8a".to_string()),
            inference_ms: 68,
            is_normal,
            no_finding_confidence: 0.4364,
            overall_risk: overall_risk.to_string(),
            clinical_summary,
            patient_meta: Some(PatientMeta {
                view_pos: req.view_pos.clone(),
                age: req.age,
                gender: req.gender.clone(),
            }),
            positive_findings,
            top_findings,
            all_findings,
            gradcam: Some(gradcam),
            disclaimer: Some(CdssDisclaimer::SHORT_FOOTER_DISCLAIMER_UZ.to_string()),
        }
    }
}

