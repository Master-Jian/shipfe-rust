# 故障排除

## 常见问题

### 1. 权限拒绝错误

```
Error: Permission denied (publickey)
```

**解决方案：**
- 验证 SSH 密钥已添加到服务器的 `~/.ssh/authorized_keys`
- 检查 SSH 密钥权限：`chmod 600 ~/.ssh/id_rsa`
- 测试 SSH 连接：`ssh user@host`

### 2. 共享资源不工作

```
Warning: Failed to create hard link for shared asset
```

**解决方案：**
- 确保配置中 `enable_shared: true`
- 检查服务器文件系统支持硬链接
- 验证 shared 目录的写入权限

### 3. 清理不工作

```
Warning: Failed to remove old release
```

**解决方案：**
- 检查 releases 目录的文件权限
- 确保没有进程正在使用旧发布文件
- 验证 `keep_releases` 设置正确

### 4. 快照创建失败

```
Error: Failed to create snapshot
```

**解决方案：**
- 检查可用磁盘空间
- 验证 releases 目录的写入权限
- 确保服务器上有 tar 命令

## 调试模式

启用详细日志：

```bash
shipfe deploy --debug
```

## 日志路径

Shipfe 的日志文件通常位于：

- 本地：`shipfe.log`
- 服务器：部署目录下的 `shipfe.log`

检查日志：

```bash
tail -f shipfe.log
```

## 性能优化

### 部署缓慢

- 为大静态文件启用共享资源
- 使用 `hashed_asset_patterns` 针对特定文件
- 考虑从模式中排除大的非更改文件

### 磁盘使用过高

- 减少 `keep_releases` 数量
- 启用共享资源
- 对单发布部署使用 `delete_old: true`

### 网络问题

- 部署前压缩大文件
- 如果支持，使用更快的 SSH 密码
- 考虑在非高峰时段部署