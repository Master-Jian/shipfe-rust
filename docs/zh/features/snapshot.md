# 资源快照

每个发布都会生成包含完整文件清单的快照，用于审计和回滚验证。

## 快照内容

- 所有部署文件及其哈希值
- 共享资源引用
- 部署时间戳和元数据
- 文件权限和大小信息

## 快照文件位置

```
releases/20260304_120100/
├── files/
│   ├── index.html
│   └── app.js -> ../../shared/assets/abc123def456.js
├── snapshot.json
└── metadata.json
```

## snapshot.json 示例

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
  }
}
```

## metadata.json 示例

```json
{
  "build_command": "npm run build",
  "deployed_by": "user",
  "deployed_at": "2026-03-04T12:01:00Z"
}
```

## 快照用途

### 审计追踪

- 记录每次部署的文件清单
- 验证文件完整性
- 追踪部署历史

### 回滚验证

- 确保回滚到的版本文件完整
- 验证共享资源引用正确
- 检查文件权限设置

### 问题诊断

- 比较不同版本的文件差异
- 识别部署问题
- 分析性能变化

## 快照生成时机

- **部署时**：每次成功部署后生成
- **回滚时**：验证目标版本的快照
- **清理时**：检查要删除版本的快照

## 快照验证

Shipfe 会验证快照的完整性：

- 检查文件是否存在
- 验证哈希值匹配
- 确认权限设置正确
- 验证共享资源链接

## 快照存储

快照文件存储在每个发布目录中：

- `snapshot.json` - 文件清单和哈希
- `metadata.json` - 部署元数据

## 查看快照

### 本地查看

```bash
# SSH到服务器查看快照
ssh user@server "cat /var/www/myapp/releases/20260304_120100/snapshot.json"
```

### 编程访问

```javascript
const fs = require('fs');
const snapshot = JSON.parse(fs.readFileSync('snapshot.json', 'utf8'));

console.log('Files in this release:');
Object.entries(snapshot.files).forEach(([file, info]) => {
  console.log(`${file}: ${info.hash} (${info.size} bytes)`);
});
```

## 快照与共享资源

当启用共享资源时，快照包含特殊处理：

```json
{
  "files": {
    "app.js": {
      "shared_hash": "abc123def456.js",
      "size": 51200,
      "permissions": "644"
    }
  }
}
```

- `shared_hash` 字段标识共享资源
- 实际文件通过硬链接引用共享存储

## 性能考虑

- 快照文件很小，通常只有几KB
- 哈希计算只对实际文件进行
- 快照验证快速且轻量

## 故障排除

### 快照创建失败

```
Error: Failed to create snapshot
```

**原因**：
- 磁盘空间不足
- 文件权限问题
- 哈希计算失败

**解决方案**：
- 检查磁盘空间
- 验证文件权限
- 查看详细错误日志

### 快照验证失败

```
Warning: Snapshot validation failed
```

**原因**：
- 文件被修改
- 权限被改变
- 共享资源链接损坏

**解决方案**：
- 检查文件完整性
- 修复文件权限
- 重新创建共享资源链接

### 快照文件丢失

如果快照文件丢失：

- 无法验证版本完整性
- 回滚可能失败
- 共享资源清理受影响

**恢复**：
- 从备份恢复
- 重新生成快照（如果可能）
- 手动验证文件