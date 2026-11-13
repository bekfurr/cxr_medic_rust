use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionRequest {
    pub image_base64: String,
    pub view_pos: String,
    pub age: i32,
    pub gender: String,
    pub with_cam: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
    pub lang: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PatientMeta {
    #[serde(default = "default_view_pos")]
    pub view_pos: String,
    #[serde(default = "default_age")]
    pub age: i32,
    #[serde(default = "default_gender")]
    pub gender: String,
}

fn default_view_pos() -> String {
    "PA".to_string()
}
fn default_age() -> i32 {
    50
}
fn default_gender() -> String {
    "M".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathologyItem {
    pub index: usize,
    pub label: String,
    pub name_uz: String,
    pub description_uz: String,
    pub prob: f32,
    pub prob_percent: f32,
    pub threshold: f32,
    pub positive: bool,
    pub risk_code: String,
    pub risk_level: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GradcamData {
    pub target_label: Option<String>,
    pub target_name_uz: Option<String>,
    pub lung_focus_ratio: Option<f32>,
    pub original_clahe_base64: Option<String>,
    pub overlay_base64: Option<String>,
    pub heatmap_only_base64: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionResponse {
    pub status: String,
    pub request_id: Option<String>,
    #[serde(default)]
    pub inference_ms: i32,
    #[serde(default)]
    pub is_normal: bool,
    #[serde(default)]
    pub no_finding_confidence: f32,
    pub overall_risk: String,
    pub clinical_summary: String,
    pub patient_meta: Option<PatientMeta>,
    #[serde(default)]
    pub positive_findings: Vec<PathologyItem>,
    #[serde(default)]
    pub top_findings: Vec<PathologyItem>,
    #[serde(default)]
    pub all_findings: Vec<PathologyItem>,
    pub gradcam: Option<GradcamData>,
    pub disclaimer: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    #[serde(default)]
    pub device: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub labels_count: usize,
    #[serde(default)]
    pub version: String,
}

