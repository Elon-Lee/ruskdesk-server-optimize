#!/bin/bash

# RustDesk Server Manager 使用示例
# 演示如何使用管理脚本

echo "=== RustDesk Server Manager 使用示例 ==="
echo

# 1. 查看帮助
echo "1. 查看帮助信息:"
echo "   ./rustdesk-server-manager.sh help"
echo

# 2. 检查状态
echo "2. 检查服务器状态:"
echo "   ./rustdesk-server-manager.sh status"
echo

# 3. 编译服务器
echo "3. 编译服务器:"
echo "   ./rustdesk-server-manager.sh build"
echo

# 4. 启动服务器
echo "4. 启动服务器:"
echo "   # 前台启动（调试模式）"
echo "   ./rustdesk-server-manager.sh start"
echo
echo "   # 后台启动"
echo "   ./rustdesk-server-manager.sh start --background"
echo
echo "   # 使用自定义密钥文件"
echo "   ./rustdesk-server-manager.sh start --keys-file my_keys.json --background"
echo
echo "   # 只启动信令服务器"
echo "   ./rustdesk-server-manager.sh start --rendezvous-only --background"
echo

# 5. 查看日志
echo "5. 查看日志:"
echo "   ./rustdesk-server-manager.sh logs"
echo

# 6. 重启服务器
echo "6. 重启服务器:"
echo "   ./rustdesk-server-manager.sh restart --background"
echo

# 7. 停止服务器
echo "7. 停止服务器:"
echo "   ./rustdesk-server-manager.sh stop"
echo

# 8. 清理
echo "8. 清理日志和PID文件:"
echo "   ./rustdesk-server-manager.sh clean"
echo

echo "=== 实际演示 ==="
echo

# 演示基本操作
echo "演示基本操作..."

# 检查状态
echo "检查当前状态:"
./rustdesk-server-manager.sh status
echo

# 启动服务器
echo "启动服务器（后台模式）:"
./rustdesk-server-manager.sh start --rendezvous-only --background
echo

# 等待一下
sleep 3

# 检查状态
echo "检查启动后的状态:"
./rustdesk-server-manager.sh status
echo

# 查看日志
echo "查看日志:"
./rustdesk-server-manager.sh logs
echo

# 停止服务器
echo "停止服务器:"
./rustdesk-server-manager.sh stop
echo

# 最终状态
echo "最终状态:"
./rustdesk-server-manager.sh status
echo

echo "=== 示例完成 ==="
