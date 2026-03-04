# Resource Snapshots

Each deployment generates a snapshot containing a complete file manifest for auditing and rollback verification.

## Snapshot Contents

- All deployed files and their hashes
- Shared resource references
- Deployment timestamp and metadata
- File permissions and size information

## Snapshot File Location

```
releases/20260304_120100/
├── files/
│   ├── index.html
│   └── app.js -> ../../shared/assets/abc123def456.js
├── snapshot.json
└── metadata.json
```

## snapshot.json Example

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

## metadata.json Example

```json
{
  "build_command": "npm run build",
  "deployed_by": "user",
  "deployed_at": "2026-03-04T12:01:00Z"
}
```

## 用途

- **审计**：验证部署内容
- **回滚验证**：确保回滚到正确版本
- **清理**：确定哪些共享资源可以删除
- **调试**：排查部署问题