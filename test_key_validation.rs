use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

// 简化的自定义密钥管理器用于测试
struct TestKeyManager {
    keys: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl TestKeyManager {
    async fn new(config_path: &str) -> Self {
        let keys = Arc::new(RwLock::new(HashMap::new()));
        let manager = Self { keys };
        manager.load_keys(config_path).await;
        manager
    }

    async fn load_keys(&self, config_path: &str) {
        if !Path::new(config_path).exists() {
            println!("Config file {} not found", config_path);
            return;
        }

        let content = match fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(e) => {
                println!("Failed to read config file: {}", e);
                return;
            }
        };

        let config: serde_json::Value = match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                println!("Failed to parse config file: {}", e);
                return;
            }
        };

        let mut keys = self.keys.write().await;
        keys.clear();

        if let Some(keys_array) = config.get("keys").and_then(|v| v.as_array()) {
            for key_obj in keys_array {
                if let (Some(key), Some(expired_str)) = (
                    key_obj.get("key").and_then(|v| v.as_str()),
                    key_obj.get("expired").and_then(|v| v.as_str()),
                ) {
                    if let Ok(expired) = DateTime::parse_from_rfc3339(expired_str) {
                        keys.insert(key.to_string(), expired.with_timezone(&Utc));
                        println!("Loaded key: {} (expires: {})", key, expired_str);
                    }
                }
            }
        }
    }

    async fn is_valid_key(&self, key: &str) -> bool {
        let keys = self.keys.read().await;
        if let Some(expired) = keys.get(key) {
            let now = Utc::now();
            let is_valid = now < *expired;
            println!("Key '{}' is {} (expires: {}, now: {})", 
                key, 
                if is_valid { "valid" } else { "expired" },
                expired.format("%Y-%m-%d %H:%M:%S UTC"),
                now.format("%Y-%m-%d %H:%M:%S UTC")
            );
            is_valid
        } else {
            println!("Key '{}' not found", key);
            false
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Testing Custom Key Manager...");
    
    // Create test config
    let config_content = r#"{
  "keys": [
    {
      "key": "test-key-123",
      "expired": "2025-12-31T23:59:59Z"
    },
    {
      "key": "expired-key-456",
      "expired": "2020-01-01T00:00:00Z"
    },
    {
      "key": "valid-key-789",
      "expired": "2025-06-30T12:00:00Z"
    }
  ]
}"#;
    
    fs::write("test_custom_keys.json", config_content).expect("Failed to write test config");
    
    // Create key manager
    let manager = TestKeyManager::new("test_custom_keys.json").await;
    
    // Test cases
    println!("\n=== Testing Key Validation ===");
    
    // Test valid key
    println!("\n1. Testing valid key 'test-key-123':");
    let is_valid = manager.is_valid_key("test-key-123").await;
    println!("   Result: {}", is_valid);
    
    // Test expired key
    println!("\n2. Testing expired key 'expired-key-456':");
    let is_valid = manager.is_valid_key("expired-key-456").await;
    println!("   Result: {}", is_valid);
    
    // Test valid key
    println!("\n3. Testing valid key 'valid-key-789':");
    let is_valid = manager.is_valid_key("valid-key-789").await;
    println!("   Result: {}", is_valid);
    
    // Test non-existent key
    println!("\n4. Testing non-existent key 'non-existent':");
    let is_valid = manager.is_valid_key("non-existent").await;
    println!("   Result: {}", is_valid);
    
    println!("\n=== Test Completed ===");
}
