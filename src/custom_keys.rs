use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::Arc,
};
use hbb_common::tokio::sync::RwLock as AsyncRwLock;
use notify::{Watcher, RecursiveMode};
use std::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomKey {
    pub key: String,
    pub expired: String, // ISO 8601 date string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomKeyConfig {
    pub keys: Vec<CustomKey>,
}

impl Default for CustomKeyConfig {
    fn default() -> Self {
        Self {
            keys: vec![
                CustomKey {
                    key: "123".to_string(),
                    expired: "2025-10-19".to_string(),
                }
            ],
        }
    }
}

pub struct CustomKeyManager {
    keys: Arc<AsyncRwLock<HashMap<String, DateTime<Utc>>>>,
    config_path: String,
    _watcher: Option<notify::RecommendedWatcher>,
}

impl CustomKeyManager {
    pub fn new_sync(config_path: &str) -> Self {
        let config_path = config_path.to_string();
        let keys = Arc::new(AsyncRwLock::new(HashMap::new()));
        
        // Create a simple synchronous version for static initialization
        let manager = Self {
            keys: keys.clone(),
            config_path: config_path.clone(),
            _watcher: None,
        };
        
        // Load keys synchronously
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<CustomKeyConfig>(&content) {
                let mut key_map = HashMap::new();
                let now = Utc::now();
                
                for custom_key in config.keys {
                    match DateTime::parse_from_rfc3339(&custom_key.expired) {
                        Ok(expired) => {
                            let expired_utc = expired.with_timezone(&Utc);
                            if expired_utc > now {
                                key_map.insert(custom_key.key, expired_utc);
                            }
                        }
                        Err(_) => {
                            // Skip invalid dates
                        }
                    }
                }
                
                // Use a blocking approach to set the keys
                let rt = hbb_common::tokio::runtime::Handle::current();
                rt.block_on(async {
                    *keys.write().await = key_map;
                });
            }
        }
        
        manager
    }
    
    pub async fn new(config_path: &str) -> Self {
        let keys = Arc::new(AsyncRwLock::new(HashMap::new()));
        let mut manager = Self {
            keys,
            config_path: config_path.to_string(),
            _watcher: None,
        };
        
        // Load initial keys
        manager.load_keys().await;
        
        // Start file watcher
        manager.start_watcher().await;
        
        manager
    }

    pub async fn load_keys(&mut self) {
        let path = Path::new(&self.config_path);
        if !path.exists() {
            // Create default config file
            let default_config = CustomKeyConfig::default();
            if let Ok(content) = serde_json::to_string_pretty(&default_config) {
                if let Err(e) = fs::write(&self.config_path, content) {
                    hbb_common::log::error!("Failed to create default config file: {}", e);
                } else {
                    hbb_common::log::info!("Created default config file: {}", self.config_path);
                }
            }
        }

        match fs::read_to_string(&self.config_path) {
            Ok(content) => {
                match serde_json::from_str::<CustomKeyConfig>(&content) {
                    Ok(config) => {
                        let mut key_map = HashMap::new();
                        let now = Utc::now();
                        
                        for custom_key in config.keys {
                            match DateTime::parse_from_rfc3339(&custom_key.expired) {
                                Ok(expired) => {
                                    let expired_utc = expired.with_timezone(&Utc);
                                    if expired_utc > now {
                        let key = custom_key.key.clone();
                        let expired = custom_key.expired.clone();
                        key_map.insert(custom_key.key, expired_utc);
                        hbb_common::log::info!("Loaded custom key: {} (expires: {})", key, expired);
                                    } else {
                                        hbb_common::log::warn!("Custom key {} has expired: {}", custom_key.key, custom_key.expired);
                                    }
                                }
                                Err(e) => {
                                    hbb_common::log::error!("Invalid date format for key {}: {} - {}", custom_key.key, custom_key.expired, e);
                                }
                            }
                        }
                        
                        let key_count = key_map.len();
                        *self.keys.write().await = key_map;
                        hbb_common::log::info!("Loaded {} valid custom keys", key_count);
                    }
                    Err(e) => {
                        hbb_common::log::error!("Failed to parse config file: {}", e);
                    }
                }
            }
            Err(e) => {
                hbb_common::log::error!("Failed to read config file: {}", e);
            }
        }
    }

    async fn start_watcher(&mut self) {
        let (tx, rx) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                hbb_common::log::error!("Failed to create file watcher: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(Path::new(&self.config_path), RecursiveMode::NonRecursive) {
            hbb_common::log::error!("Failed to watch config file: {}", e);
            return;
        }

        self._watcher = Some(watcher);

        // Spawn watcher task
        let keys = self.keys.clone();
        let config_path = self.config_path.clone();
        hbb_common::tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                match event {
                    Ok(notify::Event { kind: notify::EventKind::Modify(_), .. }) => {
                        hbb_common::log::info!("Config file modified, reloading keys...");
                        Self::reload_keys(&keys, &config_path).await;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        hbb_common::log::error!("File watcher error: {}", e);
                    }
                }
            }
        });
    }

    async fn reload_keys(keys: &Arc<AsyncRwLock<HashMap<String, DateTime<Utc>>>>, config_path: &str) {
        match fs::read_to_string(config_path) {
            Ok(content) => {
                match serde_json::from_str::<CustomKeyConfig>(&content) {
                    Ok(config) => {
                        let mut key_map = HashMap::new();
                        let now = Utc::now();
                        
                        for custom_key in config.keys {
                            match DateTime::parse_from_rfc3339(&custom_key.expired) {
                                Ok(expired) => {
                                    let expired_utc = expired.with_timezone(&Utc);
                                    if expired_utc > now {
                                        key_map.insert(custom_key.key, expired_utc);
                                    }
                                }
                                Err(e) => {
                                    hbb_common::log::error!("Invalid date format for key {}: {} - {}", custom_key.key, custom_key.expired, e);
                                }
                            }
                        }
                        
                        let key_count = key_map.len();
                        *keys.write().await = key_map;
                        hbb_common::log::info!("Reloaded {} valid custom keys", key_count);
                    }
                    Err(e) => {
                        hbb_common::log::error!("Failed to parse config file during reload: {}", e);
                    }
                }
            }
            Err(e) => {
                hbb_common::log::error!("Failed to read config file during reload: {}", e);
            }
        }
    }

    pub async fn is_valid_key(&self, key: &str) -> bool {
        let keys = self.keys.read().await;
        if let Some(expired) = keys.get(key) {
            let now = Utc::now();
            if *expired > now {
                return true;
            } else {
                hbb_common::log::debug!("Key {} has expired", key);
                return false;
            }
        }
        false
    }

    pub async fn get_all_keys(&self) -> Vec<String> {
        let keys = self.keys.read().await;
        keys.keys().cloned().collect()
    }

    pub async fn cleanup_expired_keys(&self) {
        let mut keys = self.keys.write().await;
        let now = Utc::now();
        let initial_count = keys.len();
        
        keys.retain(|_, expired| *expired > now);
        
        let removed_count = initial_count - keys.len();
        if removed_count > 0 {
            hbb_common::log::info!("Cleaned up {} expired keys", removed_count);
        }
    }
}

impl Clone for CustomKeyManager {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            config_path: self.config_path.clone(),
            _watcher: None, // Watcher is not cloned
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_custom_key_manager() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_str().unwrap();
        
        // Create test config
        let config = CustomKeyConfig {
            keys: vec![
                CustomKey {
                    key: "test123".to_string(),
                    expired: "2025-12-31T23:59:59Z".to_string(),
                },
                CustomKey {
                    key: "expired123".to_string(),
                    expired: "2020-01-01T00:00:00Z".to_string(),
                },
            ],
        };
        
        let content = serde_json::to_string_pretty(&config).unwrap();
        fs::write(config_path, content).unwrap();
        
        let manager = CustomKeyManager::new(config_path);
        
        // Test valid key
        assert!(manager.is_valid_key("test123").await);
        
        // Test expired key
        assert!(!manager.is_valid_key("expired123").await);
        
        // Test non-existent key
        assert!(!manager.is_valid_key("nonexistent").await);
    }
}
