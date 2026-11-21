//! The module's categories and their danger levels (design, section 5): code, since a category id is a
//! `&'static str`; `stores.toml` names them by id and is checked against this list.

use fresher_core::DangerLevel;

/// Every category, in the order the module lists them: what only a downloader writes (Low) first, then LM Studio's
/// models and Ollama's, which may be imported or made ones that do not come back by download (Medium).
pub(crate) const CATEGORIES: [(&str, DangerLevel); 4] = [
    ("local-ai-models.hugging-face-models", DangerLevel::Low),
    ("local-ai-models.hugging-face-datasets", DangerLevel::Low),
    ("local-ai-models.lm-studio", DangerLevel::Medium),
    ("local-ai-models.ollama", DangerLevel::Medium),
];
