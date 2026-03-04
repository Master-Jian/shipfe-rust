# 子环境配置

为不同的部署环境配置Shipfe。

## 环境配置文件

为每个环境创建单独的配置文件：

```
project/
├── shipfe.config.json          # 默认配置
├── shipfe.staging.json         # 预发布环境
├── shipfe.production.json      # 生产环境
└── shipfe.test.json           # 测试环境
```

## 环境配置示例

### 开发环境

```json
// shipfe.dev.json
{
  "remote_host": "dev.example.com",
  "remote_user": "dev",
  "remote_path": "/var/www/dev/myapp",
  "local_path": "./dist",
  "keep_releases": 3,
  "build_command": "npm run build:dev",
  "delete_old": false
}
```

### 预发布环境

```json
// shipfe.staging.json
{
  "remote_host": "staging.example.com",
  "remote_user": "deploy",
  "remote_path": "/var/www/staging/myapp",
  "local_path": "./dist",
  "keep_releases": 5,
  "build_command": "npm run build:staging",
  "enable_shared": true,
  "pre_deploy_commands": [
    "cd /var/www/staging/myapp && php artisan down"
  ],
  "post_deploy_commands": [
    "cd /var/www/staging/myapp/current && php artisan migrate",
    "cd /var/www/staging/myapp/current && php artisan up"
  ]
}
```

### 生产环境

```json
// shipfe.production.json
{
  "remote_host": "production.example.com",
  "remote_user": "deploy",
  "remote_path": "/var/www/production/myapp",
  "local_path": "./dist",
  "keep_releases": 10,
  "build_command": "npm run build:production",
  "enable_shared": true,
  "hashed_asset_patterns": [
    "**/*.js",
    "**/*.css",
    "**/*.png",
    "**/*.jpg",
    "**/*.svg",
    "**/*.woff2",
    "**/*.woff"
  ],
  "exclude_patterns": [
    "**/*.map",
    "**/*.log",
    "**/.*"
  ],
  "pre_deploy_commands": [
    "sudo systemctl stop nginx",
    "cd /var/www/production/myapp && php artisan down"
  ],
  "post_deploy_commands": [
    "cd /var/www/production/myapp/current && php artisan migrate --force",
    "cd /var/www/production/myapp/current && php artisan config:cache",
    "cd /var/www/production/myapp/current && php artisan route:cache",
    "cd /var/www/production/myapp/current && php artisan view:cache",
    "sudo systemctl start nginx",
    "cd /var/www/production/myapp/current && php artisan up"
  ]
}
```

## 使用环境配置

### 指定配置文件

```bash
# 部署到预发布环境
shipfe deploy --config shipfe.staging.json

# 部署到生产环境
shipfe deploy --config shipfe.production.json

# 回滚预发布环境
shipfe rollback --config shipfe.staging.json
```

### 环境变量

使用环境变量简化配置：

```json
// shipfe.config.json
{
  "remote_host": "$DEPLOY_HOST",
  "remote_user": "$DEPLOY_USER",
  "remote_path": "$DEPLOY_PATH",
  "build_command": "$BUILD_COMMAND"
}
```

```bash
# 设置环境变量
export DEPLOY_HOST=staging.example.com
export DEPLOY_USER=deploy
export DEPLOY_PATH=/var/www/staging/myapp
export BUILD_COMMAND="npm run build:staging"

shipfe deploy
```

### npm脚本

在 `package.json` 中定义部署脚本：

```json
{
  "scripts": {
    "deploy:dev": "shipfe deploy --config shipfe.dev.json",
    "deploy:staging": "shipfe deploy --config shipfe.staging.json",
    "deploy:prod": "shipfe deploy --config shipfe.production.json",
    "rollback:staging": "shipfe rollback --config shipfe.staging.json",
    "rollback:prod": "shipfe rollback --config shipfe.production.json"
  }
}
```

使用：

```bash
npm run deploy:staging
npm run deploy:prod
```

## 环境差异配置

### 构建命令差异

不同环境可能需要不同的构建配置：

```json
// 开发环境 - 快速构建，无优化
{
  "build_command": "npm run build:dev"
}

// 生产环境 - 完整构建，优化
{
  "build_command": "npm run build:production"
}
```

### 资源处理差异

```json
// 预发布环境 - 启用源码映射
{
  "exclude_patterns": []
}

// 生产环境 - 排除源码映射
{
  "exclude_patterns": [
    "**/*.map"
  ]
}
```

### 部署钩子差异

```json
// 开发环境 - 简单重启
{
  "post_deploy_commands": [
    "touch current/tmp/restart.txt"
  ]
}

// 生产环境 - 完整部署流程
{
  "pre_deploy_commands": [
    "sudo systemctl stop nginx"
  ],
  "post_deploy_commands": [
    "sudo systemctl start nginx"
  ]
}
```

## 配置继承

使用配置继承减少重复：

```javascript
// config.js
const baseConfig = {
  local_path: "./dist",
  keep_releases: 5,
  enable_shared: true
};

const environments = {
  dev: {
    ...baseConfig,
    remote_host: "dev.example.com",
    build_command: "npm run build:dev"
  },
  staging: {
    ...baseConfig,
    remote_host: "staging.example.com",
    build_command: "npm run build:staging"
  },
  production: {
    ...baseConfig,
    remote_host: "production.example.com",
    build_command: "npm run build:production",
    keep_releases: 10
  }
};

module.exports = environments;
```

然后生成JSON配置文件：

```bash
node -e "console.log(JSON.stringify(require('./config.js').production, null, 2))" > shipfe.production.json
```

## 安全考虑

### 敏感信息

不要在配置文件中存储敏感信息：

```json
// ❌ 不安全
{
  "database_password": "secret123"
}

// ✅ 使用环境变量
{
  "database_password": "$DB_PASSWORD"
}
```

### 权限控制

为配置文件设置适当权限：

```bash
# 配置文件权限
chmod 600 shipfe.*.json

# 环境变量文件权限
chmod 600 .env
```

### 密钥管理

为不同环境使用不同的SSH密钥：

```json
// 生产环境使用专用密钥
{
  "ssh_key_path": "~/.ssh/production_deploy_key"
}
```