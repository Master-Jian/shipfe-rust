# Shipfe

一个强大的、**免费**、**基于 Rust** 的 Web 应用部署工具，**不请求网络**，实现**一键前端静态部署包上传到服务器**。支持多环境和子环境部署，具有零停机原子部署功能。

## 主要特性

- 🚀 **免费开源**: 无隐藏费用，MIT 许可证
- 🦀 **基于 Rust**: 快速、可靠、内存安全
- 🔒 **不请求网络**: 完全离线工作，确保安全和隐私
- ⚡ **一键部署**: 即时上传静态前端包到服务器
- 🔄 **原子部署**: 零停机部署，自动回滚
- 🌍 **多环境支持**: 配置不同环境（开发、预发布、生产）
- 📦 **子环境支持**: 在同一服务器部署多个应用
- 🔑 **灵活认证**: SSH 密钥、密码或环境变量认证
- 📝 **详细日志**: 全面的部署日志用于故障排除

## 安装

```bash
npm install -g shipfe
```

## 快速开始

1. 初始化项目：
```bash
shipfe init
```

2. 在 `shipfe.config.json` 中配置部署

3. 部署：
```bash
shipfe deploy --profile prod
```

## 使用方法

### 初始化项目
```bash
shipfe init
```

### 部署到环境
```bash
# 部署到默认环境
shipfe deploy

# 部署到指定环境
shipfe deploy --profile dev

# 部署到子环境
shipfe deploy --profile dev-admin

# 部署到所有子环境
shipfe deploy --profile dev --all-sub

# 原子部署（创建 releases/时间戳 并更新 current 符号链接）
shipfe deploy --atomic
```

### 激活许可证（高级功能）
```bash
shipfe activate --profile prod --file license.key
```

### 回滚部署
```bash
shipfe rollback --profile prod --to 20260303_034945
```

## 配置

编辑 `shipfe.config.json` 来配置您的部署设置：

```json
{
  "environments": {
    "dev": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "servers": [
        {
          "host": "dev.example.com",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/dev",
          "delete_old": false
        }
      ],
      "remote_tmp": "/tmp"
    }
  }
}
```

### 认证选项

每个服务器可以有自己的认证方法。Shipfe 按以下顺序尝试认证方法：

1. **密码**（如果在服务器配置中设置了 `password`）
2. **环境变量中的 SSH 私钥**（如果设置了 `SSH_PRIVATE_KEY` 环境变量）
3. **SSH 密钥文件**（如果在服务器配置中设置了 `key_path`）

**使用不同认证方法的多个服务器示例：**

```json
{
  "environments": {
    "prod": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "servers": [
        {
          "host": "web1.prod.com",
          "port": 22,
          "username": "deploy",
          "password": "web1_password",
          "remote_deploy_path": "/var/www/prod",
          "delete_old": false
        },
        {
          "host": "web2.prod.com",
          "port": 22,
          "username": "deploy",
          "key_path": "/home/user/.ssh/web2_key",
          "remote_deploy_path": "/var/www/prod",
          "delete_old": false
        },
        {
          "host": "web3.prod.com",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/prod",
          "delete_old": false
        }
      ],
      "remote_tmp": "/tmp"
    }
  }
}
```

### 子环境

对于在同一服务器部署多个应用或不同配置，使用子环境：

```json
{
  "environments": {
    "dev": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "servers": [
        {
          "host": "dev.example.com",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/dev",
          "delete_old": false
        }
      ],
      "remote_tmp": "/tmp",
      "sub_environments": {
        "admin": {
          "build_command": "npm run build:admin",
          "remote_deploy_path": "/var/www/dev/admin"
        },
        "shop": {
          "build_command": "npm run build:shop",
          "remote_deploy_path": "/var/www/dev/shop"
        },
        "cu": {
          "build_command": "npm run build:cu",
          "remote_deploy_path": "/var/www/dev/cu"
        }
      }
    }
  }
}
```

部署到子环境：
```bash
shipfe deploy --profile dev-admin
shipfe deploy --profile dev-shop
shipfe deploy --profile dev-cu

# 一次部署到所有子环境
shipfe deploy --profile dev --all-sub
```

### 一次部署所有子环境
```bash
shipfe deploy --profile dev --all-sub
```

这将按顺序部署到所有子环境（admin、shop、cu）。

子环境继承父环境的设置，可以覆盖 `build_command`、`local_dist_path` 和 `remote_deploy_path`。

### 原子部署

Shipfe 支持原子部署以最小化停机时间。使用 `--atomic` 时，部署会创建一个带时间戳的发布目录，并更新 `current` 符号链接，实现零停机切换。

```bash
# 原子部署到默认环境
shipfe deploy --atomic

# 原子部署到指定环境
shipfe deploy --profile prod --atomic
```

**目录结构：**
```
remote_deploy_path/
├── releases/
│   ├── 20260303_034945/
│   ├── 20260303_035012/
│   └── 20260303_035045/
└── current -> releases/20260303_035045
```

您的 Web 服务器应从 `remote_deploy_path/current` 提供服务。

**或为所有服务器使用环境变量：**
```bash
export SSH_PRIVATE_KEY="$(cat ~/.ssh/prod_key)"
shipfe deploy --profile prod
```

### 认证

Shipfe 支持为每个服务器单独设置多种 SSH 认证方法。对于每个服务器，按以下顺序尝试认证方法：

1. **密码认证**：如果该服务器配置中设置了 `password`
2. **环境变量中的 SSH 私钥**：如果设置了 `SSH_PRIVATE_KEY` 环境变量（适用于所有服务器）
3. **SSH 密钥文件**：如果该服务器配置中设置了 `key_path`

#### 使用示例：

```bash
# 每个服务器可以使用不同的认证
shipfe deploy --profile prod

# 或为所有服务器使用环境变量覆盖
export SSH_PRIVATE_KEY="$(cat ~/.ssh/prod_key)"
shipfe deploy --profile prod
```

#### 为单个服务器设置 SSH 密钥：

1. **为每个服务器生成 SSH 密钥对**：
   ```bash
   # 服务器 1
   ssh-keygen -t rsa -b 4096 -f ~/.ssh/server1_key -C "server1"

   # 服务器 2
   ssh-keygen -t rsa -b 4096 -f ~/.ssh/server2_key -C "server2"
   ```

2. **将公钥复制到相应服务器**：
   ```bash
   ssh-copy-id -i ~/.ssh/server1_key.pub user@server1.com
   ssh-copy-id -i ~/.ssh/server2_key.pub user@server2.com
   ```

3. **使用服务器特定密钥配置 shipfe**：
   ```json
   {
     "servers": [
       {
         "host": "server1.com",
         "key_path": "~/.ssh/server1_key"
       },
       {
         "host": "server2.com",
         "key_path": "~/.ssh/server2_key"
       }
     ]
   }
   ```

## 功能特性

- 多环境支持
- 子环境配置
- 自定义构建命令
- 基于 SSH 的部署
- 自动备份和回滚
- 详细日志记录

## 许可证

MIT