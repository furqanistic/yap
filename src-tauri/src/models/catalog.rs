//! The speech models Yap can download.
//!
//! Files are the official whisper.cpp ggml builds on Hugging Face. Sizes and
//! SHA-256 values come from Hugging Face's LFS metadata (`X-Linked-Size` and
//! `X-Linked-ETag`) and were checked against a full download of Tiny. If you
//! add a model, verify its numbers the same way; never copy them from docs.
//!
//! Tiny and Base use full precision because they're already small and lose
//! the most accuracy when quantized. Small uses `q8_0` (half the size, about
//! the same accuracy) and Large Turbo uses `q5_0` (a third of the size).

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Languages {
    EnglishOnly,
    Multilingual,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    /// Stable id, also stored in settings (`model`).
    pub id: &'static str,
    pub label: &'static str,
    /// One short sentence for the UI.
    pub description: &'static str,
    pub languages: Languages,
    pub recommended: bool,
    pub size_bytes: u64,
    pub url: &'static str,
    #[serde(skip)]
    pub sha256: &'static str,
    #[serde(skip)]
    pub file_name: &'static str,
}

#[cfg(test)]
const BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/";

macro_rules! hf_url {
    ($file:literal) => {
        concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/",
            $file
        )
    };
}

pub const CATALOG: &[ModelInfo] = &[
    ModelInfo {
        id: "whisper-tiny",
        label: "Whisper Tiny",
        description: "Near-instant results. Best for short, clear notes.",
        languages: Languages::Multilingual,
        recommended: false,
        size_bytes: 77_691_713,
        url: hf_url!("ggml-tiny.bin"),
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        file_name: "ggml-tiny.bin",
    },
    ModelInfo {
        id: "whisper-tiny-en",
        label: "Whisper Tiny (English)",
        description: "Near-instant results, a little more accurate for English.",
        languages: Languages::EnglishOnly,
        recommended: false,
        size_bytes: 77_704_715,
        url: hf_url!("ggml-tiny.en.bin"),
        sha256: "921e4cf8686fdd993dcd081a5da5b6c365bfde1162e72b08d75ac75289920b1f",
        file_name: "ggml-tiny.en.bin",
    },
    ModelInfo {
        id: "whisper-base",
        label: "Whisper Base",
        description: "Quick and light. Good on older machines.",
        languages: Languages::Multilingual,
        recommended: false,
        size_bytes: 147_951_465,
        url: hf_url!("ggml-base.bin"),
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        file_name: "ggml-base.bin",
    },
    ModelInfo {
        id: "whisper-base-en",
        label: "Whisper Base (English)",
        description: "Quick and light, a little more accurate for English.",
        languages: Languages::EnglishOnly,
        recommended: false,
        size_bytes: 147_964_211,
        url: hf_url!("ggml-base.en.bin"),
        sha256: "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002",
        file_name: "ggml-base.en.bin",
    },
    ModelInfo {
        id: "whisper-small",
        label: "Whisper Small",
        description: "A good balance of speed and accuracy for everyday use.",
        languages: Languages::Multilingual,
        recommended: true,
        size_bytes: 264_464_607,
        url: hf_url!("ggml-small-q8_0.bin"),
        sha256: "49c8fb02b65e6049d5fa6c04f81f53b867b5ec9540406812c643f177317f779f",
        file_name: "ggml-small-q8_0.bin",
    },
    ModelInfo {
        id: "whisper-small-en",
        label: "Whisper Small (English)",
        description: "Balanced speed and accuracy, tuned for English.",
        languages: Languages::EnglishOnly,
        recommended: false,
        size_bytes: 264_477_561,
        url: hf_url!("ggml-small.en-q8_0.bin"),
        sha256: "67a179f608ea6114bd3fdb9060e762b588a3fb3bd00c4387971be4d177958067",
        file_name: "ggml-small.en-q8_0.bin",
    },
    ModelInfo {
        id: "whisper-large-turbo",
        label: "Whisper Large Turbo",
        description: "Highest accuracy. Needs a fast machine.",
        languages: Languages::Multilingual,
        recommended: false,
        size_bytes: 574_041_195,
        url: hf_url!("ggml-large-v3-turbo-q5_0.bin"),
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        file_name: "ggml-large-v3-turbo-q5_0.bin",
    },
];

pub fn find(id: &str) -> Option<&'static ModelInfo> {
    CATALOG.iter().find(|model| model.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::download::UrlPolicy;
    use std::collections::HashSet;

    #[test]
    fn ids_and_file_names_are_unique() {
        let ids: HashSet<_> = CATALOG.iter().map(|m| m.id).collect();
        let files: HashSet<_> = CATALOG.iter().map(|m| m.file_name).collect();
        assert_eq!(ids.len(), CATALOG.len());
        assert_eq!(files.len(), CATALOG.len());
    }

    #[test]
    fn entries_are_well_formed() {
        let policy = UrlPolicy::huggingface();
        for model in CATALOG {
            assert_eq!(model.url, format!("{BASE_URL}{}", model.file_name));
            assert!(policy.allows(&model.url.parse().unwrap()), "{}", model.id);
            assert_eq!(model.sha256.len(), 64, "{}", model.id);
            assert!(model.sha256.chars().all(|c| c.is_ascii_hexdigit()));
            assert!(model.size_bytes > 10_000_000);
        }
    }

    #[test]
    fn exactly_one_recommended_model() {
        assert_eq!(CATALOG.iter().filter(|m| m.recommended).count(), 1);
    }

    #[test]
    fn default_setting_is_in_the_catalog() {
        let default_model = crate::store::settings::Settings::default().model;
        assert!(find(&default_model).is_some());
    }
}
