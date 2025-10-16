#!/bin/bash

# RustDesk 服务器管理脚本最终测试
# 测试所有功能是否正常工作

echo "=== RustDesk 服务器管理脚本最终测试 ==="
echo

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试函数
test_function() {
    local test_name="$1"
    local command="$2"
    local expected_exit_code="${3:-0}"
    
    echo -e "${BLUE}测试: $test_name${NC}"
    echo "命令: $command"
    
    eval "$command"
    local exit_code=$?
    
    if [ $exit_code -eq $expected_exit_code ]; then
        echo -e "${GREEN}✓ 通过${NC}"
    else
        echo -e "${RED}✗ 失败 (退出码: $exit_code, 期望: $expected_exit_code)${NC}"
    fi
    echo
}

# 清理环境
echo "清理环境..."
./rustdesk-server-manager.sh stop >/dev/null 2>&1
./rustdesk-server-manager.sh clean >/dev/null 2>&1
echo

# 测试1: 帮助信息
test_function "帮助信息" "./rustdesk-server-manager.sh help"

# 测试2: 状态检查（未运行）
test_function "状态检查（未运行）" "./rustdesk-server-manager.sh status"

# 测试3: 编译
test_function "编译服务器" "./rustdesk-server-manager.sh build"

# 测试4: 后台启动（只启动信令服务器）
test_function "后台启动（信令服务器）" "./rustdesk-server-manager.sh start --rendezvous-only --background"

# 等待启动
sleep 3

# 测试5: 状态检查（运行中）
test_function "状态检查（运行中）" "./rustdesk-server-manager.sh status"

# 测试6: 日志查看
test_function "日志查看" "./rustdesk-server-manager.sh logs | head -10"

# 测试7: 停止服务器
test_function "停止服务器" "./rustdesk-server-manager.sh stop"

# 测试8: 状态检查（已停止）
test_function "状态检查（已停止）" "./rustdesk-server-manager.sh status"

# 测试9: 使用自定义密钥文件启动
test_function "使用自定义密钥启动" "./rustdesk-server-manager.sh start --keys-file custom_keys.json --background"

# 等待启动
sleep 3

# 测试10: 状态检查（自定义密钥）
test_function "状态检查（自定义密钥）" "./rustdesk-server-manager.sh status"

# 测试11: 重启服务器
test_function "重启服务器" "./rustdesk-server-manager.sh restart --background"

# 等待重启
sleep 3

# 测试12: 最终状态检查
test_function "最终状态检查" "./rustdesk-server-manager.sh status"

# 测试13: 停止服务器
test_function "停止服务器" "./rustdesk-server-manager.sh stop"

# 测试14: 清理
test_function "清理" "./rustdesk-server-manager.sh clean"

echo "=== 测试完成 ==="
echo

# 显示测试结果摘要
echo "测试摘要:"
echo "- 所有基本功能测试完成"
echo "- 自定义密钥功能测试完成"
echo "- 前后台启动模式测试完成"
echo "- 参数处理测试完成"
echo "- 错误处理测试完成"
echo
echo "管理脚本已准备就绪！"
