# 配置概览

Shipfe 使用 JSON 格式的配置文件来定义部署行为。

## 配置文件位置

默认情况下，Shipfe 在项目根目录查找 `shipfe.config.json` 文件。

您也可以使用 `-c` 或 `--config` 参数指定自定义配置文件：

```bash
shipfe deploy --config production.json
```

## 基本配置

最小的配置文件包含服务器连接信息：

```json
{
  "remote_host": "your-server.com",
  "remote_user": "deploy",
  "remote_path": "/var/www/myapp",
  "local_path": "./dist"
}
```

## 完整配置示例

```json
{
  "remote_host": "your-server.com",
  "remote_user": "deploy",
  "remote_path": "/var/www/myapp",
  "local_path": "./dist",
  "build_command": "npm run build",
  "keep_releases": 5,
  "delete_old": false,
  "enable_shared": true,
  "hashed_asset_patterns": [
    "**/*.js",
    "**/*.css",
    "**/*.png",
    "**/*.jpg",
    "**/*.svg",
    "**/*.woff2"
  ],
  "exclude_patterns": [
    "**/*.map",
    "**/*.log"
  ],
  "ssh_key_path": "~/.ssh/id_rsa",
  "ssh_port": 22,
  "pre_deploy_commands": [
    "sudo systemctl stop nginx"
  ],
  "post_deploy_commands": [
    "sudo systemctl start nginx"
  ]
}
```

## 配置分类

### 连接配置

- [`remote_host`](/zh/config/auth) - 服务器地址
- [`remote_user`](/zh/config/auth) - SSH用户名
- [`ssh_key_path`](/zh/config/auth) - SSH密钥路径
- [`ssh_port`](/zh/config/auth) - SSH端口

### 路径配置

- `remote_path` - 服务器部署根目录
- `local_path` - 本地构建输出目录

### 构建配置

- `build_command` - 构建命令
- `exclude_patterns` - 排除的文件模式

### 部署策略

- `keep_releases` - 保留的发布数量
- `delete_old` - 是否只保留最新版本

### 高级功能

- [`enable_shared`](/zh/features/shared-assets) - 启用共享资源
- `hashed_asset_patterns` - 共享资源文件模式
- `pre_deploy_commands` - 部署前命令
- `post_deploy_commands` - 部署后命令

## Web 服务器配置

要提供部署的应用服务，请将 Web 服务器配置为指向部署目录。详见 [Nginx 配置](nginx.md) 设置说明。

## 环境特定配置

对于不同的部署环境，可以创建多个配置文件：

```bash
# 生产环境
shipfe deploy --config production.json

# 预发布环境
shipfe deploy --config staging.json

# 测试环境
shipfe deploy --config test.json
```

## 配置验证

Shipfe 在部署前会验证配置的正确性：

- 检查必需字段
- 验证服务器连接
- 检查路径权限
- 验证构建命令

## 最佳实践

1. **使用版本控制**：将配置文件纳入版本控制
2. **环境分离**：为不同环境使用不同配置
3. **敏感信息**：不要在配置文件中存储密码
4. **权限控制**：确保配置文件权限正确（600）
5. **备份配置**：定期备份重要的配置文件

## 故障排除

### 配置无效

```bash
shipfe deploy --debug
```

启用调试模式查看详细的配置验证信息。

### 权限问题

确保配置文件权限正确：

```bash
chmod 600 shipfe.config.json
```

### 路径问题

使用绝对路径避免歧义：

```json
{
  "local_path": "/full/path/to/dist",
  "remote_path": "/var/www/myapp"
}
```