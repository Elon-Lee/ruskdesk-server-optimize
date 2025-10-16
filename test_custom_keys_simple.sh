#!/bin/bash

echo "=== RustDesk Custom Keys Test ==="

# 创建测试密钥文件
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

echo "✓ Created test custom keys file"

# 编译项目
echo "Compiling project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "✗ Compilation failed"
    exit 1
fi

echo "✓ Compilation successful"

# 测试帮助信息
echo ""
echo "=== Testing help information ==="
./target/release/hbbs --help | grep -i "custom-keys-file"

if [ $? -eq 0 ]; then
    echo "✓ Custom keys file option found in help"
else
    echo "✗ Custom keys file option not found in help"
fi

# 测试启动服务器（后台运行）
echo ""
echo "=== Testing server startup with custom keys ==="
./target/release/hbbs --port 21116 --custom-keys-file test_custom_keys.json --key test-key-123 &
SERVER_PID=$!

# 等待服务器启动
sleep 3

# 检查服务器是否正在运行
if ps -p $SERVER_PID > /dev/null; then
    echo "✓ Server started successfully with custom keys file"
    
    # 停止服务器
    kill $SERVER_PID
    wait $SERVER_PID 2>/dev/null
    echo "✓ Server stopped"
else
    echo "✗ Server failed to start"
fi

# 清理
rm -f test_custom_keys.json

echo ""
echo "=== Test completed ==="
