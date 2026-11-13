use egui::Color32;

#[derive(Clone, Debug)]
pub struct PathologyMetadata {
    pub index: usize,
    pub label: &'static str,
    pub name_uz: &'static str,
    pub name_en: &'static str,
    pub description_uz: &'static str,
    pub threshold: f32,
}

pub const ALL_PATHOLOGIES: [PathologyMetadata; 14] = [
    PathologyMetadata {
        index: 0,
        label: "Atelectasis",
        name_uz: "Atelektaz",
        name_en: "Atelectasis",
        description_uz: "O'pka to'qimasining kollapsi yoki qulashi, havosizlanishi.",
        threshold: 0.44,
    },
    PathologyMetadata {
        index: 1,
        label: "Cardiomegaly",
        name_uz: "Kardiomegaliya",
        name_en: "Cardiomegaly",
        description_uz: "Yurak o'lchamlarining me'yordan sezilarli kattalashishi (KTI > 0.50).",
        threshold: 0.29,
    },
    PathologyMetadata {
        index: 2,
        label: "Effusion",
        name_uz: "Plevral effuziya",
        name_en: "Pleural Effusion",
        description_uz: "Plevra bo'shlig'ida patologik suyuqlik to'planishi, sinuslar berkilishi.",
        threshold: 0.48,
    },
    PathologyMetadata {
        index: 3,
        label: "Infiltration",
        name_uz: "Infiltratsiya",
        name_en: "Infiltration",
        description_uz: "O'pka parenximasida yallig'lanish yoki oqsil/hujayra to'planishi.",
        threshold: 0.45,
    },
    PathologyMetadata {
        index: 4,
        label: "Mass",
        name_uz: "Hajmli hosila (Mass)",
        name_en: "Mass (>3cm)",
        description_uz: "O'pka sohasida 3 sm dan katta patologik zich hosila.",
        threshold: 0.39,
    },
    PathologyMetadata {
        index: 5,
        label: "Nodule",
        name_uz: "Tugun (Nodule)",
        name_en: "Nodule (≤3cm)",
        description_uz: "O'pka maydonida 3 sm gacha bo'lgan aylana/tuxumsimon zichlik.",
        threshold: 0.42,
    },
    PathologyMetadata {
        index: 6,
        label: "Pneumonia",
        name_uz: "Pnevmoniya",
        name_en: "Pneumonia",
        description_uz: "O'pka to'qimasining o'tkir yuqumli yallig'lanish o'chog'i.",
        threshold: 0.25,
    },
    PathologyMetadata {
        index: 7,
        label: "Pneumothorax",
        name_uz: "Pnevmotoraks",
        name_en: "Pneumothorax",
        description_uz: "Plevra bo'shlig'ida erkin havo to'planishi va o'pka siqilishi.",
        threshold: 0.41,
    },
    PathologyMetadata {
        index: 8,
        label: "Consolidation",
        name_uz: "Konsolidatsiya",
        name_en: "Consolidation",
        description_uz: "Alveolalarning havo o'rniga ekssudat bilan to'lishi (zichlashuv).",
        threshold: 0.40,
    },
    PathologyMetadata {
        index: 9,
        label: "Edema",
        name_uz: "O'pka shishi",
        name_en: "Pulmonary Edema",
        description_uz: "O'pka qon tomirlaridan to'qimalarga suyuqlik sizib chiqishi (dimlanish).",
        threshold: 0.33,
    },
    PathologyMetadata {
        index: 10,
        label: "Emphysema",
        name_uz: "Emfizema",
        name_en: "Emphysema",
        description_uz: "Alveola devorlarining yemirilishi va o'pkaning me'yordan ortiq shishishi.",
        threshold: 0.46,
    },
    PathologyMetadata {
        index: 11,
        label: "Fibrosis",
        name_uz: "Fibroz",
        name_en: "Fibrosis",
        description_uz: "O'pka to'qimasining chandiqlanishi va elastikligining pasayishi.",
        threshold: 0.28,
    },
    PathologyMetadata {
        index: 12,
        label: "Pleural_Thickening",
        name_uz: "Plevra qalinlashuvi",
        name_en: "Pleural Thickening",
        description_uz: "Plevra varaqlarining fibroz yoki yallig'lanish tufayli qalinlashishi.",
        threshold: 0.36,
    },
    PathologyMetadata {
        index: 13,
        label: "Hernia",
        name_uz: "Diafragma churrasi",
        name_en: "Hiatal / Diaphragmatic Hernia",
        description_uz: "Qorin bo'shlig'i a'zolarining ko'krak qafasiga surilishi.",
        threshold: 0.13,
    },
];

pub fn find_pathology(label: &str) -> Option<&'static PathologyMetadata> {
    ALL_PATHOLOGIES
        .iter()
        .find(|p| p.label.eq_ignore_ascii_case(label))
}

pub struct MedicalColors;

impl MedicalColors {
    pub const BACKGROUND: Color32 = Color32::from_rgb(0x09, 0x0E, 0x17);
    pub const SURFACE: Color32 = Color32::from_rgb(0x10, 0x19, 0x28);
    pub const SURFACE_ELEVATED: Color32 = Color32::from_rgb(0x16, 0x23, 0x3B);
    pub const BORDER: Color32 = Color32::from_rgb(0x1F, 0x32, 0x54);
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(0x2C, 0x44, 0x6F);

    pub const PRIMARY_CYAN: Color32 = Color32::from_rgb(0x00, 0xE5, 0xFF);
    pub const CYAN_DIM: Color32 = Color32::from_rgb(0x00, 0x96, 0xB0);
    pub const PRIMARY_TEAL: Color32 = Color32::from_rgb(0x00, 0xB4, 0xD8);
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(0x2D, 0x7F, 0xF9);

    pub const RISK_HIGH: Color32 = Color32::from_rgb(0xFF, 0x2A, 0x55);
    pub const RISK_MODERATE_HIGH: Color32 = Color32::from_rgb(0xFF, 0x8C, 0x00);
    pub const RISK_MODERATE_LOW: Color32 = Color32::from_rgb(0xFF, 0xC1, 0x07);
    pub const RISK_LOW: Color32 = Color32::from_rgb(0x00, 0xC8, 0x96);
    pub const RISK_NORMAL: Color32 = Color32::from_rgb(0x00, 0xE6, 0x76);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF0, 0xF4, 0xFA);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x64, 0x74, 0x8B);

    pub fn for_risk(risk_code: Option<&str>, is_positive: bool) -> Color32 {
        if is_positive {
            match risk_code {
                Some("HIGH") => Self::RISK_HIGH,
                Some("MODERATE_HIGH") => Self::RISK_MODERATE_HIGH,
                _ => Self::RISK_HIGH,
            }
        } else {
            match risk_code {
                Some("MODERATE_LOW") => Self::RISK_MODERATE_LOW,
                Some("LOW") => Self::RISK_LOW,
                _ => Self::RISK_NORMAL,
            }
        }
    }
}

pub struct CdssDisclaimer;

impl CdssDisclaimer {
    pub const APP_TITLE: &'static str = "BMI CXR-AI v2";
    pub const SYSTEM_VERSION: &'static str =
        "BMI_CXR_multilabel_v2 (DenseNet-121 Multimodal Macro-AUROC 0.777)";

    pub const DIALOG_TITLE_UZ: &'static str =
        "Tibbiy Ogohlantirish va Mas'uliyatdan Ozod Bo'lish";

    pub const FULL_DISCLAIMER_UZ: &'static str =
        "DIQQAT: \"BMI CXR-AI v2\" ilovasi ko'krak qafasi rentgenogrammalarini chuqur tahlil \
qiluvchi sun'iy intellektga asoslangan Klinik Qaror Qabul Qilishga Ko'maklashish \
Tizimi (Clinical Decision Support System - CDSS / Second-Opinion Tool) hisoblanadi.\n\n\
- Ushbu ilova faqat malakali shifokorlar, rentgenologlar va tibbiyot mutaxassislari uchun yordamchi vositadir.\n\
- Tizim yakuniy tashxis qo'ymaydi va sertifikatlangan rentgenolog shifokor xulosasini almashtirmaydi.\n\
- Grad-CAM++ issiqlik xaritasi (heatmap) model e'tibor qaratgan sohalarni vizual ko'rsatadi, ammo morfologik patologiyani 100% kafolatlamaydi.\n\
- Bemorga davo choralarini belgilashdan oldin barcha klinik, laborator va instrumental tekshiruvlar to'liq baholanishi shart.\n\n\
Ilovadan foydalanish orqali siz ushbu shartlarga rozilik bildirasiz.";

    pub const SHORT_FOOTER_DISCLAIMER_UZ: &'static str =
        "Eslatma: Ushbu tizim faqat klinik qaror qabul qilishga ko'maklashish (CDSS) uchun mo'ljallangan bo'lib, \
mustaqil klinik tashxis hisoblanmaydi. Yakuniy tashxis sertifikatlangan shifokor-rentgenolog tomonidan tasdiqlanishi shart.";

}

