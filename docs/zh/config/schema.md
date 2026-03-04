# 配置架构

完整的Shipfe配置文件架构说明。

## 必需配置

### remote_host

服务器主机名或IP地址。

```json
{
  "remote_host": "your-server.com"
}
```

### remote_user

SSH登录用户名。

```json
{
  "remote_user": "deploy"
}
```

### remote_path

服务器上的部署根目录。

```json
{
  "remote_path": "/var/www/myapp"
}
```

### local_path

本地构建输出目录。

```json
{
  "local_path": "./dist"
}
```

## 可选配置

### build_command

部署前执行的构建命令。

```json
{
  "build_command": "npm run build"
}
```

### keep_releases

保留的发布版本数量。默认值：5

```json
{
  "keep_releases": 10
}
```

### delete_old

是否只保留最新版本，删除所有旧版本。默认值：false

```json
{
  "delete_old": true
}
```

### enable_shared

启用共享资源功能。默认值：false

```json
{
  "enable_shared": true
}
```

### hashed_asset_patterns

共享资源的文件匹配模式。默认值：常见的静态资源类型

```json
{
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

### exclude_patterns

部署时排除的文件模式。

```json
{
  "exclude_patterns": [
    "**/*.map",
    "**/*.log",
    "**/.DS_Store"
  ]
}
```

### ssh_key_path

SSH私钥路径。默认值：`~/.ssh/id_rsa`

```json
{
  "ssh_key_path": "~/.ssh/deploy_key"
}
```

### ssh_port

SSH连接端口。默认值：22

```json
{
  "ssh_port": 2222
}
```

### pre_deploy_commands

部署前在服务器上执行的命令数组。

```json
{
  "pre_deploy_commands": [
    "sudo systemctl stop nginx",
    "cd /var/www/myapp && php artisan down"
  ]
}
```

### post_deploy_commands

部署后在服务器上执行的命令数组。

```json
{
  "post_deploy_commands": [
    "cd /var/www/myapp/current && php artisan migrate",
    "sudo systemctl start nginx",
    "cd /var/www/myapp/current && php artisan up"
  ]
}
```

## 配置类型

### 字符串类型

- `remote_host`
- `remote_user`
- `remote_path`
- `local_path`
- `build_command`
- `ssh_key_path`

### 数字类型

- `keep_releases`
- `ssh_port`

### 布尔类型

- `delete_old`
- `enable_shared`

### 数组类型

- `hashed_asset_patterns`: 字符串数组
- `exclude_patterns`: 字符串数组
- `pre_deploy_commands`: 字符串数组
- `post_deploy_commands`: 字符串数组

## 模式语法

文件匹配模式使用 glob 语法：

- `*` - 匹配任意字符（不含路径分隔符）
- `**` - 匹配任意字符（包含路径分隔符）
- `?` - 匹配单个字符
- `[abc]` - 匹配括号内的任意字符
- `{a,b,c}` - 匹配大括号内的任意模式

### 示例

```json
{
  "hashed_asset_patterns": [
    "**/*.js",        // 所有JS文件
    "**/*.css",       // 所有CSS文件
    "assets/**/*",    // assets目录下的所有文件
    "**/*.{png,jpg}"  // 所有PNG和JPG文件
  ],
  "exclude_patterns": [
    "**/*.map",       // 排除源码映射文件
    "**/*.log",       // 排除日志文件
    ".git/**",        // 排除Git目录
    "node_modules/**" // 排除依赖目录
  ]
}
```

## 环境变量

配置文件支持环境变量替换：

```json
{
  "remote_host": "$DEPLOY_HOST",
  "remote_user": "$DEPLOY_USER",
  "remote_path": "/var/www/$APP_NAME"
}
```

使用时设置环境变量：

```bash
export DEPLOY_HOST=production.example.com
export DEPLOY_USER=deploy
export APP_NAME=myapp
shipfe deploy
```

## 验证规则

Shipfe 会验证配置的正确性：

- **必需字段**：`remote_host`, `remote_user`, `remote_path`, `local_path`
- **路径存在**：`local_path` 必须存在
- **SSH连接**：能够建立SSH连接
- **权限检查**：服务器目录具有写入权限
- **模式有效性**：glob模式语法正确