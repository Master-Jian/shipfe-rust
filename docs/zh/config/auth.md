# 认证配置

配置SSH认证以安全连接到远程服务器。

## SSH密钥认证（推荐）

Shipfe 使用SSH密钥认证，这是最安全和最方便的方式。

### 生成SSH密钥

如果还没有SSH密钥：

```bash
# 生成RSA密钥（推荐）
ssh-keygen -t rsa -b 4096 -C "your-email@example.com"

# 或生成Ed25519密钥（更现代）
ssh-keygen -t ed25519 -C "your-email@example.com"
```

### 配置密钥路径

在 `shipfe.config.json` 中指定私钥路径：

```json
{
  "ssh_key_path": "~/.ssh/id_rsa"
}
```

默认路径是 `~/.ssh/id_rsa`。

### 添加公钥到服务器

将公钥添加到服务器的 `authorized_keys`：

```bash
# 自动添加（推荐）
ssh-copy-id user@your-server.com

# 或手动添加
cat ~/.ssh/id_rsa.pub | ssh user@server "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"
```

### 验证SSH连接

测试SSH连接是否正常：

```bash
ssh -i ~/.ssh/id_rsa user@your-server.com
```

## SSH端口配置

如果SSH服务使用非标准端口：

```json
{
  "ssh_port": 2222
}
```

## 密钥权限

确保SSH密钥文件权限正确：

```bash
# 私钥权限
chmod 600 ~/.ssh/id_rsa

# 公钥权限
chmod 644 ~/.ssh/id_rsa.pub

# .ssh目录权限
chmod 700 ~/.ssh
```

## 使用不同的密钥

为不同项目或环境使用不同的SSH密钥：

```json
{
  "ssh_key_path": "~/.ssh/project_deploy_key"
}
```

## SSH代理转发

如果使用SSH代理，可以配置代理转发：

```bash
# 在本地启用代理
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_rsa

# 配置转发（在shipfe配置中不需要特殊设置）
```

## 密码认证（不推荐）

虽然不推荐，但也可以使用密码认证：

**注意**：密码认证不够安全，建议使用SSH密钥。

如果必须使用密码，Shipfe会提示输入密码。

## 多服务器部署

为多个服务器配置不同的认证：

```json
// staging.json
{
  "remote_host": "staging.example.com",
  "remote_user": "deploy",
  "ssh_key_path": "~/.ssh/staging_key"
}

// production.json
{
  "remote_host": "production.example.com",
  "remote_user": "deploy",
  "ssh_key_path": "~/.ssh/production_key"
}
```

## 故障排除

### 权限拒绝错误

```
Permission denied (publickey)
```

**解决方案**：

1. 检查私钥路径是否正确
2. 验证公钥是否已添加到服务器
3. 检查密钥文件权限
4. 测试直接SSH连接

### 主机密钥验证失败

```
Host key verification failed
```

**解决方案**：

```bash
# 添加主机密钥到known_hosts
ssh-keyscan your-server.com >> ~/.ssh/known_hosts

# 或禁用严格检查（不推荐）
# 在~/.ssh/config中添加：
# Host your-server.com
#   StrictHostKeyChecking no
```

### SSH连接超时

```
Connection timed out
```

**解决方案**：

1. 检查服务器是否可达
2. 验证防火墙设置
3. 确认SSH服务正在运行
4. 检查网络连接

### 调试SSH连接

启用详细SSH调试：

```bash
ssh -v -i ~/.ssh/id_rsa user@your-server.com
```

或在Shipfe中使用调试模式：

```bash
shipfe deploy --debug
```

## 安全最佳实践

1. **使用强密钥**：使用4096位RSA或Ed25519密钥
2. **定期轮换密钥**：定期更换SSH密钥
3. **限制密钥使用**：为特定服务器限制密钥使用
4. **禁用密码认证**：在服务器上禁用SSH密码认证
5. **使用SSH代理**：使用ssh-agent管理密钥
6. **监控访问**：监控SSH登录日志