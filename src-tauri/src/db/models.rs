use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageEntry {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub description: String,
    pub pinyin_full: String,
    pub pinyin_initials: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistory {
    pub id: i64,
    pub page_id: i64,
    pub last_searched_at: String,
    pub search_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub key: String,
    pub value: String,
}
