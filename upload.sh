#!/bin/bash

# RustDesk服务器上传脚本
# 将当前目录打包并上传到远程服务器

set -e

# 配置
REMOTE_HOST="36.134.76.205"
REMOTE_PORT="14805"
REMOTE_USER="root"
REMOTE_DIR="/opt/rustdesk-server"
LOCAL_DIR="."

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    if ! command -v tar &> /dev/null; then
        log_error "tar 命令未找到，请安装 tar"
        exit 1
    fi
    
    if ! command -v scp &> /dev/null; then
        log_error "scp 命令未找到，请安装 openssh-client"
        exit 1
    fi
    
    if ! command -v ssh &> /dev/null; then
        log_error "ssh 命令未找到，请安装 openssh-client"
        exit 1
    fi
    
    log_success "依赖检查通过"
}

# 创建临时目录
TEMP_DIR=$(mktemp -d)
log_info "创建临时目录: $TEMP_DIR"

# 清理函数
cleanup() {
    log_info "清理临时文件..."
    rm -rf "$TEMP_DIR"
    if [ -f "rustdesk-server.tar.gz" ]; then
        rm -f "rustdesk-server.tar.gz"
    fi
}

# 设置退出时清理
trap cleanup EXIT

# 获取当前目录名
CURRENT_DIR_NAME=$(basename "$(pwd)")
log_info "当前目录: $CURRENT_DIR_NAME"

# 创建压缩包
create_archive() {
    log_info "创建压缩包..."
    
    # 排除不需要的文件
    EXCLUDE_PATTERNS=(
        ".git"
        "target"
        "*.log"
        "*.tmp"
        "*.swp"
        "*.swo"
        "*~"
        ".DS_Store"
        "Thumbs.db"
        "node_modules"
        ".vscode"
        ".idea"
        "*.tar.gz"
        "*.zip"
    )
    
    # 构建tar排除参数
    EXCLUDE_ARGS=""
    for pattern in "${EXCLUDE_PATTERNS[@]}"; do
        EXCLUDE_ARGS="$EXCLUDE_ARGS --exclude=$pattern"
    done
    
    # 创建压缩包
    tar -czf "rustdesk-server.tar.gz" $EXCLUDE_ARGS -C "$(dirname "$(pwd)")" "$CURRENT_DIR_NAME"
    
    if [ $? -eq 0 ]; then
        log_success "压缩包创建成功: rustdesk-server.tar.gz"
        log_info "压缩包大小: $(du -h rustdesk-server.tar.gz | cut -f1)"
    else
        log_error "压缩包创建失败"
        exit 1
    fi
}

# 测试SSH连接
test_ssh_connection() {
    log_info "测试SSH连接..."
    
    if ssh -p "$REMOTE_PORT" -o ConnectTimeout=10 -o BatchMode=yes "$REMOTE_USER@$REMOTE_HOST" "echo 'SSH连接测试成功'" 2>/dev/null; then
        log_success "SSH连接测试成功"
    else
        log_warning "SSH连接测试失败，可能需要密码或密钥认证"
        log_info "请确保可以手动连接到: ssh -p $REMOTE_PORT $REMOTE_USER@$REMOTE_HOST"
        read -p "是否继续? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "用户取消操作"
            exit 0
        fi
    fi
}

# 上传文件
upload_file() {
    log_info "上传文件到远程服务器..."
    
    if scp -P "$REMOTE_PORT" "rustdesk-server.tar.gz" "$REMOTE_USER@$REMOTE_HOST:/tmp/"; then
        log_success "文件上传成功"
    else
        log_error "文件上传失败"
        exit 1
    fi
}

# 在远程服务器上解压和部署
deploy_on_remote() {
    log_info "在远程服务器上部署..."
    
    ssh -p "$REMOTE_PORT" "$REMOTE_USER@$REMOTE_HOST" << EOF
        set -e
        
        echo "=== 远程部署开始 ==="
        
        # 创建目标目录
        echo "创建目标目录: $REMOTE_DIR"
        sudo mkdir -p "$REMOTE_DIR"
        
        # 备份现有目录（如果存在）
        if [ -d "$REMOTE_DIR/rustdesk-server" ]; then
            echo "备份现有目录..."
            sudo mv "$REMOTE_DIR/rustdesk-server" "$REMOTE_DIR/rustdesk-server.backup.\$(date +%Y%m%d_%H%M%S)"
        fi
        
        # 解压到临时目录
        echo "解压文件..."
        cd /tmp
        tar -xzf rustdesk-server.tar.gz
        
        # 移动到目标目录
        echo "移动到目标目录..."
        sudo mv "$CURRENT_DIR_NAME" "$REMOTE_DIR/"
        
        # 设置权限
        echo "设置权限..."
        sudo chown -R root:root "$REMOTE_DIR/$CURRENT_DIR_NAME"
        sudo chmod -R 755 "$REMOTE_DIR/$CURRENT_DIR_NAME"
        
        # 清理临时文件
        echo "清理临时文件..."
        rm -f /tmp/rustdesk-server.tar.gz
        
        echo "=== 部署完成 ==="
        echo "部署路径: $REMOTE_DIR/$CURRENT_DIR_NAME"
        echo "目录内容:"
        ls -la "$REMOTE_DIR/$CURRENT_DIR_NAME"
EOF

    if [ $? -eq 0 ]; then
        log_success "远程部署成功"
    else
        log_error "远程部署失败"
        exit 1
    fi
}

# 显示部署信息
show_deployment_info() {
    log_info "=== 部署信息 ==="
    echo "远程服务器: $REMOTE_USER@$REMOTE_HOST:$REMOTE_PORT"
    echo "部署路径: $REMOTE_DIR/$CURRENT_DIR_NAME"
    echo ""
    echo "连接命令:"
    echo "ssh -p $REMOTE_PORT $REMOTE_USER@$REMOTE_HOST"
    echo ""
    echo "进入部署目录:"
    echo "cd $REMOTE_DIR/$CURRENT_DIR_NAME"
    echo ""
    echo "启动服务器:"
    echo "sudo ./target/debug/hbbs -k _ -r 21116"
    echo "sudo ./target/debug/hbbr -k _ -r 21116"
}

# 主函数
main() {
    log_info "=== RustDesk服务器上传脚本 ==="
    log_info "目标服务器: $REMOTE_USER@$REMOTE_HOST:$REMOTE_PORT"
    log_info "目标目录: $REMOTE_DIR"
    echo ""
    
    check_dependencies
    create_archive
    test_ssh_connection
    upload_file
    deploy_on_remote
    show_deployment_info
    
    log_success "=== 上传完成 ==="
}

# 运行主函数
main "$@"
