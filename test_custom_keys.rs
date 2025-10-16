use hbbs::CustomKeyManager;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();
    
    println!("Testing Custom Key Manager...");
    
    // Create a custom key manager with test config
    let mut manager = CustomKeyManager::new("test_custom_keys.json").await;
    
    // Test valid key
    println!("Testing valid key 'test-key-123':");
    let is_valid = manager.is_valid_key("test-key-123").await;
    println!("Result: {}", is_valid);
    
    // Test expired key
    println!("Testing expired key 'expired-key-456':");
    let is_valid = manager.is_valid_key("expired-key-456").await;
    println!("Result: {}", is_valid);
    
    // Test valid key
    println!("Testing valid key 'valid-key-789':");
    let is_valid = manager.is_valid_key("valid-key-789").await;
    println!("Result: {}", is_valid);
    
    // Test non-existent key
    println!("Testing non-existent key 'non-existent':");
    let is_valid = manager.is_valid_key("non-existent").await;
    println!("Result: {}", is_valid);
    
    // Test cleanup
    println!("Running cleanup...");
    manager.cleanup_expired_keys().await;
    
    // Test again after cleanup
    println!("Testing expired key after cleanup 'expired-key-456':");
    let is_valid = manager.is_valid_key("expired-key-456").await;
    println!("Result: {}", is_valid);
    
    println!("Test completed!");
}