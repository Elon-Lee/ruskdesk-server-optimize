# RustDesk Server Manager 使用说明

这是一个用于管理 RustDesk 服务器的完整管理脚本，支持启动、停止、重启、状态检查等功能。

## 功能特性

- ✅ **自动编译**: 自动检测并编译服务器
- ✅ **进程管理**: 完整的启动/停止/重启功能
- ✅ **状态监控**: 实时检查服务器运行状态
- ✅ **日志管理**: 查看和管理服务器日志
- ✅ **自定义配置**: 支持自定义密钥文件和配置文件
- ✅ **后台运行**: 支持后台运行模式
- ✅ **端口检查**: 自动检查服务器端口状态
- ✅ **错误处理**: 完善的错误处理和日志记录

## 快速开始

### 1. 基本使用

```bash
# 查看帮助
./rustdesk-server-manager.sh help

# 启动服务器（自动编译）
./rustdesk-server-manager.sh start

# 停止服务器
./rustdesk-server-manager.sh stop

# 重启服务器
./rustdesk-server-manager.sh restart

# 检查状态
./rustdesk-server-manager.sh status
```

### 2. 使用自定义密钥

```bash
# 使用自定义密钥文件启动
./rustdesk-server-manager.sh start --keys-file my_keys.json

# 后台运行
./rustdesk-server-manager.sh start --keys-file my_keys.json --background
```

### 3. 只启动特定服务器

```bash
# 只启动信令服务器
./rustdesk-server-manager.sh start --rendezvous-only

# 只启动中继服务器
./rustdesk-server-manager.sh start --relay-only
```

## 详细命令说明

### start - 启动服务器

```bash
./rustdesk-server-manager.sh start [选项]
```

**选项:**
- `--keys-file <file>`: 指定自定义密钥文件
- `--config <file>`: 指定配置文件
- `--relay-only`: 只启动中继服务器
- `--rendezvous-only`: 只启动信令服务器
- `--background`: 后台运行
- `--no-build`: 不自动编译

**示例:**
```bash
# 基本启动
./rustdesk-server-manager.sh start

# 使用自定义密钥后台启动
./rustdesk-server-manager.sh start --keys-file custom_keys.json --background

# 只启动信令服务器
./rustdesk-server-manager.sh start --rendezvous-only
```

### stop - 停止服务器

```bash
./rustdesk-server-manager.sh stop
```

停止所有相关的 RustDesk 服务器进程。

### restart - 重启服务器

```bash
./rustdesk-server-manager.sh restart [选项]
```

重启服务器，选项与 `start` 命令相同。

### status - 检查状态

```bash
./rustdesk-server-manager.sh status
```

检查服务器运行状态，包括：
- 进程信息
- 端口监听状态
- 运行时间

### build - 编译服务器

```bash
./rustdesk-server-manager.sh build
```

手动编译服务器（通常不需要，start 命令会自动编译）。

### logs - 查看日志

```bash
./rustdesk-server-manager.sh logs
```

显示服务器日志的最后50行。

### clean - 清理

```bash
./rustdesk-server-manager.sh clean
```

清理PID文件和日志文件。

## 配置说明

### 环境变量

脚本使用以下默认配置，可以通过修改脚本中的变量来调整：

```bash
SERVER_BINARY="hbbs"                    # 信令服务器二进制文件名
RELAY_BINARY="hbbr"                     # 中继服务器二进制文件名
BUILD_DIR="target/release"              # 编译输出目录
PID_FILE="/tmp/rustdesk-server.pid"     # PID文件路径
LOG_FILE="/tmp/rustdesk-server.log"     # 日志文件路径
CUSTOM_KEYS_FILE="custom_keys.json"     # 默认密钥文件
```

### 自定义密钥文件

创建 `custom_keys.json` 文件：

```json
{
  "keys": [
    {
      "key": "your-custom-key-here",
      "expires_at": "2025-12-31T23:59:59Z"
    }
  ]
}
```

## 使用场景

### 1. 开发环境

```bash
# 前台运行，方便调试
./rustdesk-server-manager.sh start

# 查看日志
./rustdesk-server-manager.sh logs
```

### 2. 生产环境

```bash
# 后台运行
./rustdesk-server-manager.sh start --background

# 定期检查状态
./rustdesk-server-manager.sh status
```

### 3. 测试环境

```bash
# 使用测试密钥
./rustdesk-server-manager.sh start --keys-file test_keys.json

# 只启动信令服务器进行测试
./rustdesk-server-manager.sh start --rendezvous-only
```

## 故障排除

### 1. 服务器启动失败

```bash
# 检查编译是否成功
./rustdesk-server-manager.sh build

# 查看详细日志
./rustdesk-server-manager.sh logs

# 检查端口是否被占用
netstat -tlnp | grep -E ':(21115|21116|21117|21118|21119)'
```

### 2. 权限问题

```bash
# 确保脚本有执行权限
chmod +x rustdesk-server-manager.sh

# 确保有写入临时目录的权限
ls -la /tmp/rustdesk-server.*
```

### 3. 端口冲突

```bash
# 检查端口使用情况
./rustdesk-server-manager.sh status

# 停止冲突的进程
./rustdesk-server-manager.sh stop
```

## 系统服务集成

### 创建 systemd 服务

创建 `/etc/systemd/system/rustdesk-server.service`：

```ini
[Unit]
Description=RustDesk Server
After=network.target

[Service]
Type=forking
User=rustdesk
Group=rustdesk
WorkingDirectory=/path/to/rustdesk-server
ExecStart=/path/to/rustdesk-server/rustdesk-server-manager.sh start --background
ExecStop=/path/to/rustdesk-server/rustdesk-server-manager.sh stop
ExecReload=/path/to/rustdesk-server/rustdesk-server-manager.sh restart
PIDFile=/tmp/rustdesk-server.pid
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

启用服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable rustdesk-server
sudo systemctl start rustdesk-server
```

## 注意事项

1. **权限**: 确保脚本有执行权限
2. **端口**: 确保相关端口（21115-21119）未被占用
3. **依赖**: 确保已安装 Rust 和 cargo
4. **日志**: 定期清理日志文件避免占用过多磁盘空间
5. **密钥**: 妥善保管自定义密钥文件

## 更新日志

- **v1.0.0**: 初始版本，支持基本的启动/停止/重启功能
- 支持自定义密钥文件
- 支持后台运行
- 支持状态检查和日志查看
