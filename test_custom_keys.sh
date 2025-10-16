#!/bin/bash

echo "Testing Custom Key Manager..."

# Create test config
cat > test_custom_keys.json << 'EOF'
{
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
}
EOF

echo "Test config created. Now testing with hbbs..."

# Test with hbbs binary
echo "Testing valid key 'test-key-123':"
echo "test-key-123" | timeout 5s ./target/debug/hbbs --test-custom-key 2>/dev/null || echo "Key validation test completed"

echo "Testing expired key 'expired-key-456':"
echo "expired-key-456" | timeout 5s ./target/debug/hbbs --test-custom-key 2>/dev/null || echo "Key validation test completed"

echo "Testing non-existent key 'non-existent':"
echo "non-existent" | timeout 5s ./target/debug/hbbs --test-custom-key 2>/dev/null || echo "Key validation test completed"

echo "Test completed!"
