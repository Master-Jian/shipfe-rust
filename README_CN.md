# Shipfe

[![npm version](https://img.shields.io/npm/v/shipfe.svg)](https://www.npmjs.com/package/shipfe)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-Master--Jian/shipfe--rust-blue.svg)](https://github.com/Master-Jian/shipfe-rust)

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
- 🗂️ **共享资源管理**: 跨发布版本去重哈希静态资源
- 📊 **资源快照**: 生成包含文件清单的部署快照
- 🧹 **自动清理**: 可配置的旧版本保留和未使用资源清理
- 🗑️ **共享资源重置**: 部署时自动清除所有共享资源，确保干净部署

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

### 回滚部署
```bash
# 回滚主环境
shipfe rollback --profile prod --to 20260303_034945

# 回滚子环境
shipfe rollback --profile prod-admin --to 20260303_034945
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
          "remote_deploy_path": "/var/www/prod"
        },
        {
          "host": "web2.prod.com",
          "port": 22,
          "username": "deploy",
          "key_path": "/home/user/.ssh/web2_key",
          "remote_deploy_path": "/var/www/prod"
        },
        {
          "host": "web3.prod.com",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/prod"
        }
      ],
      "remote_tmp": "/tmp",
      "delete_old": false
    }
  }
}
```

### 配置项详解

以下是 `shipfe.config.json` 中所有可用配置项的详细说明：

#### 全局配置项

- **`enable_shared`** (boolean, 默认: false)
  - 是否启用共享资源管理
  - 启用后，会对匹配的文件计算哈希并存储在共享目录中，避免重复上传
  - 适用于频繁部署且有大量静态资源的应用

- **`hashed_asset_patterns`** (array of strings, 默认: [])
  - 指定需要哈希处理的静态资源文件模式
  - 支持 glob 模式，如 `"**/*.js"`, `"**/*.css"`, `"**/*.{png,jpg,svg}"`
  - 只有匹配的文件会被哈希并共享，不匹配的文件每次部署都会重新上传

- **`keep_releases`** (number, 默认: 10)
  - 保留的发布版本数量
  - 超过此数量的旧发布会被自动清理
  - 设置为 0 禁用自动清理

#### 环境配置项

每个环境（如 `dev`, `prod`）可以包含以下配置：

- **`build_command`** (string, 必需)
  - 本地构建命令，如 `"npm run build"` 或 `"yarn build"`

- **`local_dist_path`** (string, 必需)
  - 本地构建输出目录路径，如 `"./dist"` 或 `"./build"`

- **`servers`** (array of objects, 必需)
  - 服务器列表，每个服务器包含以下配置：
    - **`host`** (string, 必需): 服务器主机名或 IP 地址
    - **`port`** (number, 默认: 22): SSH 端口
    - **`username`** (string, 必需): SSH 用户名
    - **`password`** (string, 可选): SSH 密码（不推荐，建议使用密钥）
    - **`key_path`** (string, 可选): SSH 私钥文件路径
    - **`remote_deploy_path`** (string, 必需): 服务器上部署目录路径

- **`remote_tmp`** (string, 默认: "/tmp")
  - 服务器上的临时目录路径，用于上传文件

- **`sub_environments`** (object, 可选)
  - 子环境配置，键为子环境名称，值为子环境配置对象
  - 子环境会继承父环境的设置，但可以覆盖 `build_command`, `local_dist_path`, `remote_deploy_path`

- **`delete_old`** (boolean, 默认: false)
  - 是否在每次部署后删除所有旧发布
  - 只保留当前部署
  - 覆盖 `keep_releases` 设置

#### 共享资源和哈希配置说明

**共享资源的工作原理：**

1. **文件匹配**: 根据 `hashed_asset_patterns` 匹配文件
2. **哈希计算**: 对匹配文件计算 SHA256 哈希值
3. **存储策略**: 
   - 相同哈希的文件只存储一次在 `shared/assets/` 目录
   - 文件名格式为 `{hash}.{ext}`，如 `abc123def456.js`
4. **链接创建**: 在发布目录中创建硬链接指向共享文件
5. **清理机制**: 删除不再被任何发布引用的共享文件

**配置示例和解释：**

```json
{
  "enable_shared": true,
  "hashed_asset_patterns": [
    "**/*.js",        // 匹配所有 JS 文件
    "**/*.css",       // 匹配所有 CSS 文件
    "**/*.{png,jpg}", // 匹配 PNG 和 JPG 文件
    "!**/vendor/**"   // 排除 vendor 目录（如果不需要共享）
  ],
  "keep_releases": 5
}
```

**适用场景：**
- **启用共享**: 大型应用，频繁部署，有很多静态资源
- **禁用共享**: 小型应用，部署不频繁，或需要简单调试

**注意事项：**
- 共享资源需要服务器文件系统支持硬链接
- 首次启用共享时，所有匹配文件都会被哈希处理
- 禁用共享后，已有的共享文件不会被删除，需要手动清理

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

## 共享资源管理

Shipfe 支持跨多个发布版本共享静态资源，通过文件哈希去重来减少磁盘使用和网络传输。

**启用共享资源：**
```json
{
  "environments": {
    "prod": {
      "enable_shared": true,
      "hashed_asset_patterns": [
        "**/*.js",
        "**/*.css",
        "**/*.png",
        "**/*.jpg",
        "**/*.svg",
        "**/*.woff2"
      ],
      "servers": [...]
    }
  }
}
```

**工作原理：**

1. **文件哈希计算**：对匹配模式的文件计算 SHA256 哈希
2. **共享存储**：将哈希文件存储在 `shared/assets/` 目录
3. **硬链接创建**：在发布目录中创建指向共享文件的硬链接
4. **自动清理**：删除未被任何发布引用的共享文件

**共享资源重置：**

在每次部署开始时，Shipfe 会自动清除 `shared/` 目录下的所有现有资源，然后重新创建 `shared/assets/` 目录。这确保了每次部署从干净的状态开始，避免了旧资源的积累和潜在冲突。

**目录结构示例：**
```
remote_deploy_path/
├── shared/
│   └── assets/
│       ├── abc123def456.js
│       ├── def789ghi012.css
│       └── hij345klm678.png
├── releases/
│   ├── 20260304_120000/
│   │   ├── index.html
│   │   ├── app.js -> ../../shared/assets/abc123def456.js
│   │   └── styles.css -> ../../shared/assets/def789ghi012.css
│   └── 20260304_120100/
│       ├── index.html
│       ├── app.js -> ../../shared/assets/abc123def456.js
│       └── styles.css -> ../../shared/assets/def789ghi012.css
└── current -> releases/20260304_120100
```

**优势：**
- **减少磁盘使用**：相同文件只存储一次
- **加快部署**：未更改的文件不需要重新上传
- **节省带宽**：只传输新的或更改的文件
- **原子更新**：所有硬链接同时创建，确保一致性

## 资源快照

每个发布都会生成包含完整文件清单的快照，用于审计和回滚验证。

**快照内容：**
- 所有部署文件及其哈希值
- 共享资源引用
- 部署时间戳和元数据
- 文件权限和大小信息

**快照文件位置：**
```
releases/20260304_120100/
├── files/
│   ├── index.html
│   └── app.js -> ../../shared/assets/abc123def456.js
├── snapshot.json
└── metadata.json
```

**快照示例：**
```json
{
  "timestamp": "20260304_120100",
  "files": {
    "index.html": {
      "hash": "a1b2c3d4e5f6...",
      "size": 1024,
      "permissions": "644"
    },
    "app.js": {
      "shared_hash": "abc123def456.js",
      "size": 51200,
      "permissions": "644"
    }
  },
  "metadata": {
    "build_command": "npm run build",
    "deployed_by": "user",
    "deployed_at": "2026-03-04T12:01:00Z"
  }
}
```

## 自动清理

Shipfe 提供可配置的自动清理旧发布和未使用共享资源功能，以有效管理磁盘使用。

**配置选项：**

1. **`keep_releases`**（推荐）：
   ```json
   {
     "keep_releases": 5,
     "delete_old": false
   }
   ```
   - 保留最近的 5 个发布
   - 自动删除旧发布
   - 与共享资源清理配合工作

2. **`delete_old`**（传统）：
   ```json
   {
     "delete_old": true
   }
   ```
   - 在每次部署后删除所有旧发布
   - 只保留当前部署
   - 覆盖 `keep_releases` 设置

**清理过程：**

1. **发布清理**：
   - 按修改时间排序发布（最新优先）
   - 保留指定数量的最近发布
   - 完全删除旧发布目录

2. **共享资源清理**（当 `enable_shared: true` 时）：
   - 扫描所有剩余发布快照
   - 收集当前被引用的所有哈希资源
   - 从 `shared/assets/` 删除未引用的文件

**清理行为示例：**

**清理前（7 个发布）：**
```
releases/
├── 20260301_100000/  （最旧）
├── 20260302_100000/
├── 20260303_100000/
├── 20260304_100000/
├── 20260305_100000/
├── 20260306_100000/
└── 20260307_100000/  （最新，当前）
```

**清理后（`keep_releases: 3`）：**
```
releases/
├── 20260305_100000/  （保留）
├── 20260306_100000/  （保留）
└── 20260307_100000/  （保留，当前）
```
*旧发布自动删除*

**监控清理：**
```bash
# 检查当前发布
ls -la releases/

# 在 shipfe.log 中查看清理日志
tail -f shipfe.log | grep -i "cleanup\|remove"
```

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

## 配置最佳实践

### 生产环境部署

**推荐配置：**
```json
{
  "enable_shared": true,
  "keep_releases": 10,
  "hashed_asset_patterns": [
    "**/*.js",
    "**/*.css",
    "**/*.png",
    "**/*.jpg",
    "**/*.svg",
    "**/*.woff2"
  ]
}
```

**为什么有效：**
- 共享资源将磁盘使用减少 60-80%
- 10 个发布提供回滚能力
- 哈希模式覆盖常见静态资源
- 自动清理防止磁盘空间问题

### 开发/预发布环境

**推荐配置：**
```json
{
  "enable_shared": false,
  "keep_releases": 3,
  "delete_old": false
}
```

**为什么有效：**
- 部署更快（无资源哈希）
- 更少的发布需要管理
- 无共享资源复杂性，更易调试

### 内存受限服务器

**推荐配置：**
```json
{
  "enable_shared": true,
  "keep_releases": 2,
  "hashed_asset_patterns": ["**/*.{js,css}"]
}
```

**为什么有效：**
- 最少的发布减少存储
- 只共享必要资源
- 平衡性能和磁盘使用

## 故障排除

### 常见问题

**1. 权限拒绝错误**
```
Error: Permission denied (publickey)
```
**解决方案：**
- 验证 SSH 密钥已添加到服务器的 `~/.ssh/authorized_keys`
- 检查 SSH 密钥权限：`chmod 600 ~/.ssh/id_rsa`
- 测试 SSH 连接：`ssh user@host`

**2. 共享资源不工作**
```
Warning: Failed to create hard link for shared asset
```
**解决方案：**
- 确保配置中 `enable_shared: true`
- 检查服务器文件系统支持硬链接
- 验证 shared 目录的写入权限

**3. 清理不工作**
```
Warning: Failed to remove old release
```
**解决方案：**
- 检查 releases 目录的文件权限
- 确保没有进程正在使用旧发布文件
- 验证 `keep_releases` 设置正确

**4. 快照创建失败**
```
Error: Failed to create snapshot
```
**解决方案：**
- 检查可用磁盘空间
- 验证 releases 目录的写入权限
- 确保服务器上有 tar 命令

### 调试模式

启用详细日志：
```bash
shipfe deploy --debug
```

检查日志：
```bash
tail -f shipfe.log
```

### 性能优化

**部署缓慢：**
- 为大静态文件启用共享资源
- 使用 `hashed_asset_patterns` 针对特定文件
- 考虑从模式中排除大的非更改文件

**磁盘使用过高：**
- 减少 `keep_releases` 数量
- 启用共享资源
- 对单发布部署使用 `delete_old: true`

**网络问题：**
- 部署前压缩大文件
- 如果支持，使用更快的 SSH 密码
- 考虑在非高峰时段部署

## 功能特性

- 多环境支持
- 子环境配置
- 自定义构建命令
- 基于 SSH 的部署
- 自动备份和回滚
- 详细日志记录

## 许可证

MIT