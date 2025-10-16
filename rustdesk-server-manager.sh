#!/bin/bash

# RustDesk Server Manager
# 用于管理 RustDesk 服务器的启动、停止、重启和状态检查

set -e

# 配置变量
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_BINARY="hbbs"
RELAY_BINARY="hbbr"
BUILD_DIR="target/release"
PID_FILE="/tmp/rustdesk-server.pid"
LOG_FILE="/tmp/rustdesk-server.log"
CUSTOM_KEYS_FILE="custom_keys.json"
CONFIG_FILE=""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

# 显示帮助信息
show_help() {
    cat << EOF
RustDesk Server Manager

用法: $0 [命令] [选项]

命令:
    start       启动服务器
    stop        停止服务器
    restart     重启服务器
    status      检查服务器状态
    build       编译服务器
    logs        查看日志
    clean       清理日志和PID文件
    help        显示此帮助信息

选项:
    --keys-file <file>    指定自定义密钥文件 (默认: custom_keys.json)
    --config <file>       指定配置文件
    --relay-only         只启动中继服务器
    --rendezvous-only    只启动信令服务器
    --background         后台运行
    --no-build           不自动编译

示例:
    $0 start                              # 启动服务器
    $0 start --keys-file my_keys.json     # 使用自定义密钥文件启动
    $0 start --relay-only                 # 只启动中继服务器
    $0 restart --background               # 后台重启服务器
    $0 status                             # 检查服务器状态
    $0 logs                               # 查看日志

EOF
}

# 检查服务器是否运行
is_running() {
    # 首先检查PID文件
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if ps -p "$pid" > /dev/null 2>&1; then
            return 0
        else
            rm -f "$PID_FILE"
        fi
    fi
    
    # 如果PID文件不存在或无效，检查进程
    if pgrep -f "$SERVER_BINARY" > /dev/null 2>&1 || pgrep -f "$RELAY_BINARY" > /dev/null 2>&1; then
        return 0
    fi
    
    return 1
}

# 获取服务器PID
get_pid() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if ps -p "$pid" > /dev/null 2>&1; then
            echo "$pid"
            return
        fi
    fi
    
    # 从进程列表中获取PID
    local server_pid=$(pgrep -f "$SERVER_BINARY" 2>/dev/null | head -1)
    local relay_pid=$(pgrep -f "$RELAY_BINARY" 2>/dev/null | head -1)
    
    if [ -n "$server_pid" ]; then
        echo "$server_pid"
    elif [ -n "$relay_pid" ]; then
        echo "$relay_pid"
    else
        echo ""
    fi
}

# 编译服务器
build_server() {
    log_info "开始编译 RustDesk 服务器..."
    
    if [ ! -f "Cargo.toml" ]; then
        log_error "未找到 Cargo.toml 文件，请确保在项目根目录运行"
        exit 1
    fi
    
    # 检查 Rust 环境
    if ! command -v cargo &> /dev/null; then
        log_error "未找到 cargo 命令，请先安装 Rust"
        exit 1
    fi
    
    # 编译
    cargo build --release
    
    if [ $? -eq 0 ]; then
        log_info "编译成功"
    else
        log_error "编译失败"
        exit 1
    fi
}

# 启动服务器
start_server() {
    local background=false
    local relay_only=false
    local rendezvous_only=false
    local no_build=false
    
    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --keys-file)
                CUSTOM_KEYS_FILE="$2"
                shift 2
                ;;
            --config)
                CONFIG_FILE="$2"
                shift 2
                ;;
            --relay-only)
                relay_only=true
                shift
                ;;
            --rendezvous-only)
                rendezvous_only=true
                shift
                ;;
            --background)
                background=true
                shift
                ;;
            --no-build)
                no_build=true
                shift
                ;;
            *)
                log_error "未知参数: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # 检查是否已在运行
    if is_running; then
        log_warn "服务器已在运行 (PID: $(get_pid))"
        return 0
    fi
    
    # 编译服务器
    if [ "$no_build" = false ]; then
        if [ ! -f "$BUILD_DIR/$SERVER_BINARY" ] || [ ! -f "$BUILD_DIR/$RELAY_BINARY" ]; then
            build_server
        fi
    fi
    
    # 检查密钥文件
    if [ -n "$CUSTOM_KEYS_FILE" ] && [ ! -f "$CUSTOM_KEYS_FILE" ]; then
        log_warn "自定义密钥文件不存在: $CUSTOM_KEYS_FILE"
        log_info "将使用默认密钥"
        CUSTOM_KEYS_FILE=""
    fi
    
    # 构建启动命令
    local server_cmd_args=""
    local relay_cmd_args=""
    
    # 信令服务器参数（支持自定义密钥文件）
    if [ -n "$CUSTOM_KEYS_FILE" ]; then
        server_cmd_args="$server_cmd_args --custom-keys-file $CUSTOM_KEYS_FILE"
    fi
    if [ -n "$CONFIG_FILE" ]; then
        server_cmd_args="$server_cmd_args --config $CONFIG_FILE"
    fi
    
    # 中继服务器参数（不支持自定义密钥文件）
    if [ -n "$CONFIG_FILE" ]; then
        relay_cmd_args="$relay_cmd_args --config $CONFIG_FILE"
    fi
    
    # 启动服务器
    log_info "启动 RustDesk 服务器..."
    
    if [ "$background" = true ]; then
        # 后台启动
        if [ "$relay_only" = true ]; then
            nohup "$BUILD_DIR/$RELAY_BINARY" $relay_cmd_args > "$LOG_FILE" 2>&1 &
        elif [ "$rendezvous_only" = true ]; then
            nohup "$BUILD_DIR/$SERVER_BINARY" $server_cmd_args > "$LOG_FILE" 2>&1 &
        else
            # 启动两个服务器
            nohup "$BUILD_DIR/$SERVER_BINARY" $server_cmd_args > "$LOG_FILE" 2>&1 &
            local server_pid=$!
            echo $server_pid > "$PID_FILE"
            sleep 2
            nohup "$BUILD_DIR/$RELAY_BINARY" $relay_cmd_args >> "$LOG_FILE" 2>&1 &
        fi
        
        sleep 2
        if is_running; then
            log_info "服务器已在后台启动 (PID: $(get_pid))"
            log_info "日志文件: $LOG_FILE"
        else
            log_error "服务器启动失败，请检查日志: $LOG_FILE"
            exit 1
        fi
    else
        # 前台启动
        if [ "$relay_only" = true ]; then
            exec "$BUILD_DIR/$RELAY_BINARY" $relay_cmd_args
        elif [ "$rendezvous_only" = true ]; then
            exec "$BUILD_DIR/$SERVER_BINARY" $server_cmd_args
        else
            log_info "启动信令服务器和中继服务器..."
            log_warn "前台模式下，两个服务器将同时运行"
            log_warn "使用 Ctrl+C 停止服务器"
            
            # 启动信令服务器
            "$BUILD_DIR/$SERVER_BINARY" $server_cmd_args &
            local server_pid=$!
            echo $server_pid > "$PID_FILE"
            
            # 等待一下再启动中继服务器
            sleep 2
            
            # 启动中继服务器
            "$BUILD_DIR/$RELAY_BINARY" $relay_cmd_args &
            local relay_pid=$!
            
            # 等待进程结束
            wait $server_pid $relay_pid
        fi
    fi
}

# 停止服务器
stop_server() {
    if ! is_running; then
        log_warn "服务器未运行"
        return 0
    fi
    
    local pid=$(get_pid)
    log_info "停止服务器 (PID: $pid)..."
    
    # 发送 TERM 信号
    kill -TERM "$pid" 2>/dev/null || true
    
    # 等待进程结束
    local count=0
    while ps -p "$pid" > /dev/null 2>&1 && [ $count -lt 10 ]; do
        sleep 1
        count=$((count + 1))
    done
    
    # 如果还在运行，强制杀死
    if ps -p "$pid" > /dev/null 2>&1; then
        log_warn "强制停止服务器..."
        kill -KILL "$pid" 2>/dev/null || true
        sleep 1
    fi
    
    # 清理PID文件
    rm -f "$PID_FILE"
    
    # 停止所有相关进程
    pkill -f "$SERVER_BINARY" 2>/dev/null || true
    pkill -f "$RELAY_BINARY" 2>/dev/null || true
    
    log_info "服务器已停止"
}

# 重启服务器
restart_server() {
    log_info "重启服务器..."
    stop_server
    sleep 2
    start_server "$@"
}

# 检查服务器状态
check_status() {
    if is_running; then
        local pid=$(get_pid)
        log_info "服务器正在运行 (PID: $pid)"
        
        # 显示进程信息
        if command -v ps &> /dev/null; then
            echo
            log_info "进程信息:"
            ps -p "$pid" -o pid,ppid,command,etime,pcpu,pmem
        fi
        
        # 检查端口
        echo
        log_info "端口检查:"
        for port in 21115 21116 21117 21118 21119; do
            if lsof -i :$port > /dev/null 2>&1; then
                log_info "  端口 $port: 正在监听"
            else
                log_warn "  端口 $port: 未监听"
            fi
        done
        
        return 0
    else
        log_warn "服务器未运行"
        return 1
    fi
}

# 查看日志
view_logs() {
    if [ -f "$LOG_FILE" ]; then
        log_info "显示服务器日志 (最后50行):"
        echo "----------------------------------------"
        tail -n 50 "$LOG_FILE"
        echo "----------------------------------------"
        log_info "完整日志文件: $LOG_FILE"
    else
        log_warn "日志文件不存在: $LOG_FILE"
    fi
}

# 清理
cleanup() {
    log_info "清理日志和PID文件..."
    rm -f "$PID_FILE"
    rm -f "$LOG_FILE"
    log_info "清理完成"
}

# 主函数
main() {
    case "${1:-help}" in
        start)
            shift
            start_server "$@"
            ;;
        stop)
            stop_server
            ;;
        restart)
            shift
            restart_server "$@"
            ;;
        status)
            check_status
            ;;
        build)
            build_server
            ;;
        logs)
            view_logs
            ;;
        clean)
            cleanup
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "未知命令: $1"
            echo
            show_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"
