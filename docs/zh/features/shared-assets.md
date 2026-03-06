# 共享资源管理

共享资源用于将 **对外访问路径**（发布目录中的文件）与 **实际存储位置**（`shared/` 目录）解耦，这样经常复用的静态资源就不需要每次都完整重新上传。

## 启用共享资源

```json
{
  "enable_shared": true,
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

- `enable_shared`：控制当前环境是否启用共享资源。
- `hashed_asset_patterns`：用来匹配需要作为共享资源处理的文件（通常是带内容哈希的 js/css/image 等静态资源，例如 `app-abc123.js`）。

## 工作原理

1. **文件匹配**
   - 构建完成后，Shipfe 会扫描 `local_dist_path` 下的所有文件。
   - 使用 `hashed_asset_patterns`（glob）匹配出需要共享的文件，将其相对路径（例如 `assets/app-abc123.js`、`assets/images/logo-xyz999.png`）写入 `shipfe.snapshot.json` 的 `hashed_assets` 列表。

2. **正常发布解压**
   - 整个 `dist` 目录会被打包、上传，并解压到：

     ```
     remote_deploy_path/
       releases/<timestamp>/...   # 结构与本地 dist 完全一致
     ```

   - Web 服务器通过 `remote_deploy_path/current` 提供静态资源，对外 URL 与本地 dist 一致。

3. **共享存储策略**
   - 对于 `hashed_assets` 中的每一个路径 `p`：
     - 源文件：`releases/<timestamp>/{p}`
     - 目标文件：`shared/{p}`
   - 远端执行的核心逻辑：

     ```bash
     mv -f releases/<timestamp>/{p} shared/{p}
     ln -f shared/{p} releases/<timestamp>/{p}
     ```

   - 这意味着：
     - **发布目录结构保持不变**，`current/` 中的路径和 dist 完全一致；
     - `shared/` 下面使用同样的相对路径结构（例如 `shared/assets/...`、`shared/assets/images/...`）；
     - 如果 `shared/{p}` 已经存在，新版本会用 `mv -f` 覆盖旧文件。

4. **硬链接创建**
   - 访问仍然发生在 `current/...` 路径下。
   - 文件系统通过硬链接保证这些文件实际物理存储在 `shared/...` 下，一份数据可被多个发布版本复用。

5. **清理策略**
   - 当前实现 **不会自动清空或回收 `shared/` 目录中的旧文件**。
   - 如果磁盘空间需要回收，可在停机或确认无历史版本依赖时手动清理 `shared/` 目录。

## 目录结构示例

假设 dist 结构中存在：

```text
dist/
  index.html
  assets/
    app-abc123.js
    images/
      logo-xyz999.png
```

启用共享后，典型远端目录结构：

```text
remote_deploy_path/
├── shared/
│   └── assets/
│       ├── app-abc123.js
│       └── images/
│           └── logo-xyz999.png
├── releases/
│   └── 20260304_120100/
│       ├── index.html
│       ├── assets/
│       │   ├── app-abc123.js -> ../../../shared/assets/app-abc123.js
│       │   └── images/
│       │       └── logo-xyz999.png -> ../../../../shared/assets/images/logo-xyz999.png
│       └── ...
└── current -> releases/20260304_120100
```

## 常见配置示例

- 只共享打包产出的 js/css：

  ```json
  "hashed_asset_patterns": [
    "assets/**/*.js",
    "assets/**/*.css"
  ]
  ```

- 整个 assets 目录都作为共享资源（包括图片、字体等）：

  ```json
  "hashed_asset_patterns": [
    "assets/**"
  ]
  ```

无论使用哪种匹配规则，`shared/` 下的目录结构都会与 dist 的相对路径保持一致。

## 优势

- **减少重复上传**：多次部署中未变更的静态资源可以重用。
- **节省磁盘空间**：相同路径的文件在 shared 中只保存一份数据。
- **兼容现有路径**：对外访问路径和 dist 完全一致，无需改动 Nginx 或前端构建配置。

## 注意事项

- 服务器文件系统需要支持硬链接。
- `hashed_asset_patterns` 只决定哪些文件参与共享，其它文件仍按普通方式部署到每次发布中。
- 如果不需要这套优化或想简化行为，可以为某个环境关闭 `enable_shared`。