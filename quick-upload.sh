#!/bin/bash

# 快速上传脚本 - 简化版本
set -e

REMOTE_HOST="36.134.76.205"
REMOTE_PORT="14805"
REMOTE_USER="root"
REMOTE_DIR="/opt/rustdesk-server"

echo "=== 快速上传 RustDesk 服务器 ==="

# 创建压缩包（排除不需要的文件）
echo "创建压缩包..."
tar -czf rustdesk-server.tar.gz \
    --exclude=.git \
    --exclude=target \
    --exclude="*.log" \
    --exclude="*.tmp" \
    --exclude="*.swp" \
    --exclude="*.swo" \
    --exclude="*~" \
    --exclude=".DS_Store" \
    --exclude="Thumbs.db" \
    --exclude="node_modules" \
    --exclude=".vscode" \
    --exclude=".idea" \
    --exclude="*.tar.gz" \
    --exclude="*.zip" \
    -C .. rustdesk-server-master

echo "压缩包大小: $(du -h rustdesk-server.tar.gz | cut -f1)"

# 上传并部署
echo "上传并部署到远程服务器..."
scp -P $REMOTE_PORT rustdesk-server.tar.gz $REMOTE_USER@$REMOTE_HOST:/tmp/ && \
ssh -p $REMOTE_PORT $REMOTE_USER@$REMOTE_HOST "
    sudo mkdir -p $REMOTE_DIR
    cd /tmp
    tar -xzf rustdesk-server.tar.gz
    sudo mv rustdesk-server-master $REMOTE_DIR/
    sudo chown -R root:root $REMOTE_DIR/rustdesk-server-master
    sudo chmod -R 755 $REMOTE_DIR/rustdesk-server-master
    rm -f rustdesk-server.tar.gz
    echo '部署完成!'
    echo '部署路径: $REMOTE_DIR/rustdesk-server-master'
    ls -la $REMOTE_DIR/rustdesk-server-master
"

# 清理本地压缩包
rm -f rustdesk-server.tar.gz

echo "=== 上传完成 ==="
echo "连接命令: ssh -p $REMOTE_PORT $REMOTE_USER@$REMOTE_HOST"
echo "进入目录: cd $REMOTE_DIR/rustdesk-server-master"
