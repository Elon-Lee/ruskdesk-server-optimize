#!/bin/bash

# 测试客户端连接脚本
# 模拟客户端使用key "123" 连接服务器

echo "=== 测试客户端连接 ==="
echo

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 服务器信息
SERVER_HOST="127.0.0.1"
SERVER_PORT="21115"
TEST_KEY="123"

echo -e "${BLUE}测试参数:${NC}"
echo "  服务器地址: $SERVER_HOST:$SERVER_PORT"
echo "  测试密钥: $TEST_KEY"
echo

# 检查服务器状态
echo -e "${BLUE}检查服务器状态...${NC}"
./rustdesk-server-manager.sh status
echo

# 清空日志
echo -e "${BLUE}清空日志文件...${NC}"
echo "" > /tmp/rustdesk-server.log
echo

# 使用netcat测试连接
echo -e "${BLUE}测试TCP连接...${NC}"
echo "测试连接到 $SERVER_HOST:$SERVER_PORT"
timeout 5 nc -v $SERVER_HOST $SERVER_PORT 2>&1 || echo "连接测试完成"
echo

# 等待一下让日志写入
sleep 2

# 显示相关日志
echo -e "${BLUE}服务器日志 (最后20行):${NC}"
echo "----------------------------------------"
tail -20 /tmp/rustdesk-server.log
echo "----------------------------------------"
echo

# 检查是否有授权相关的日志
echo -e "${BLUE}搜索授权相关日志:${NC}"
grep -i "key\|auth\|licence\|punch\|register" /tmp/rustdesk-server.log | tail -10
echo

echo -e "${GREEN}测试完成！${NC}"
echo
echo "请检查上面的日志输出，查看是否有以下信息："
echo "1. 客户端连接请求"
echo "2. 密钥验证过程"
echo "3. 授权成功或失败的原因"
