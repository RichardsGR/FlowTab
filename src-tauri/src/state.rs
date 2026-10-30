use crate::browsers::OpenTab;
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::Instant;

pub struct TabCache {
    pub browser_key: String,
    pub tabs: Vec<OpenTab>,
    pub fetched_at: Instant,
}

pub struct AppState {
    pub db: Connection,
    pub data_dir: PathBuf,
    pub tab_cache: Option<TabCache>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = dirs_app_data()?;
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("flowtab.db");
        let db = crate::db::init::initialize_database(&db_path)?;

        Ok(Self { db, data_dir, tab_cache: None })
    }
}

fn dirs_app_data() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Use dirs-next logic for macOS
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| {
            // Fallback: try to get home via 'dscl' or just use /tmp
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        });

        Ok(PathBuf::from(format!(
            "{}/Library/Application Support/com.flowtab.app",
            home
        )))
    }

    #[cfg(target_os = "windows")]
    {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOMEDRIVE").and_then(|d| {
                std::env::var("HOMEPATH").map(|p| format!("{}{}", d, p))
            }))
            .unwrap_or_else(|_| "C:".to_string());

        Ok(PathBuf::from(format!(
            "{}\\AppData\\Roaming\\com.flowtab.app",
            home
        )))
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Ok(PathBuf::from(format!(
            "{}/.local/share/com.flowtab.app",
            home
        )))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Ok(PathBuf::from(format!("{}/.flowtab", home)))
    }
}
