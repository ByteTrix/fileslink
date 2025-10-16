use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::sync::RwLock;
use log::info;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub unique_id: String,
    pub telegram_file_id: String,
    pub file_name: String,
    pub mime_type: Option<String>,
    pub file_size: u32,
    pub uploaded_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<i32>,  // Telegram message ID for large files
}

#[derive(Debug, Serialize, Deserialize)]
struct FileStorageData {
    files: HashMap<String, FileMetadata>,
}

static FILE_STORAGE: Lazy<RwLock<FileStorageData>> = Lazy::new(|| {
    RwLock::new(FileStorageData {
        files: HashMap::new(),
    })
});

const STORAGE_FILE_PATH: &str = "file_mappings.json";

pub async fn init_file_storage() -> Result<(), String> {
    if Path::new(STORAGE_FILE_PATH).exists() {
        let content = fs::read_to_string(STORAGE_FILE_PATH)
            .await
            .map_err(|e| format!("Failed to read file mappings: {}", e))?;
        
        let data: FileStorageData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse file mappings: {}", e))?;
        
        let mut storage = FILE_STORAGE.write().await;
        *storage = data;
        
        info!("Loaded {} file mappings from storage", storage.files.len());
    } else {
        info!("No existing file mappings found, starting fresh");
    }
    
    Ok(())
}

pub async fn save_file_metadata(metadata: FileMetadata) -> Result<(), String> {
    let mut storage = FILE_STORAGE.write().await;
    storage.files.insert(metadata.unique_id.clone(), metadata);
    
    let json = serde_json::to_string_pretty(&*storage)
        .map_err(|e| format!("Failed to serialize file mappings: {}", e))?;
    
    fs::write(STORAGE_FILE_PATH, json)
        .await
        .map_err(|e| format!("Failed to write file mappings: {}", e))?;
    
    Ok(())
}

pub async fn get_file_metadata(unique_id: &str) -> Option<FileMetadata> {
    let storage = FILE_STORAGE.read().await;
    storage.files.get(unique_id).cloned()
}

pub async fn list_all_files() -> Vec<FileMetadata> {
    let storage = FILE_STORAGE.read().await;
    storage.files.values().cloned().collect()
}

pub async fn delete_file_metadata(unique_id: &str) -> Result<(), String> {
    let mut storage = FILE_STORAGE.write().await;
    storage.files.remove(unique_id);
    
    let json = serde_json::to_string_pretty(&*storage)
        .map_err(|e| format!("Failed to serialize file mappings: {}", e))?;
    
    fs::write(STORAGE_FILE_PATH, json)
        .await
        .map_err(|e| format!("Failed to write file mappings: {}", e))?;
    
    Ok(())
}
