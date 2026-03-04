# deploy 命令

执行应用部署到远程服务器。

## 语法

```bash
shipfe deploy [options]
```

## 描述

`deploy` 命令执行完整的部署流程：
1. 执行构建命令（如果配置了）
2. 创建新的发布目录
3. 上传文件到服务器
4. 原子化切换到新版本
5. 清理旧版本（根据配置）

## 选项

| 选项 | 描述 |
|------|------|
| `-c, --config <file>` | 指定配置文件路径 |
| `-d, --debug` | 启用调试模式，显示详细日志 |
| `--dry-run` | 预览模式，不实际执行部署 |
| `--skip-build` | 跳过构建步骤 |
| `-h, --help` | 显示帮助信息 |

## 示例

### 基本部署

```bash
shipfe deploy
```

使用默认配置文件执行完整部署。

### 指定配置文件

```bash
shipfe deploy --config production.json
```

使用特定的配置文件进行部署。

### 调试模式

```bash
shipfe deploy --debug
```

启用详细日志输出，帮助诊断问题。

### 预览部署

```bash
shipfe deploy --dry-run
```

显示将要执行的操作，但不实际部署。

### 跳过构建

```bash
shipfe deploy --skip-build
```

如果构建已手动完成，可以跳过构建步骤。

## 部署流程

### 1. 构建阶段

如果配置了 `build_command`，Shipfe会执行它：

```bash
npm run build
# 或其他构建命令
```

### 2. 准备阶段

- 创建时间戳目录：`releases/YYYYMMDD_HHMMSS`
- 生成部署快照

### 3. 上传阶段

- 上传构建产物到新发布目录
- 处理共享资源（如果启用）

### 4. 切换阶段

- 原子化切换 `current` 符号链接
- 确保零停机部署

### 5. 清理阶段

- 删除超出 `keep_releases` 限制的旧版本
- 清理未使用的共享资源

## 部署目录结构

```
remote_path/
├── current -> releases/20260304_120000  # 当前版本
├── releases/
│   ├── 20260304_120000/                # 最新版本
│   ├── 20260304_110000/                # 上一版本
│   └── 20260304_100000/                # 更早版本
└── shared/                             # 共享资源（可选）
    └── assets/
```

## 错误处理

部署过程中如果出现错误：

- 自动清理失败的发布目录
- 不影响当前运行的版本
- 详细的错误日志输出

## 相关命令

- [`init`](/zh/commands/init) - 初始化配置文件
- [`rollback`](/zh/commands/rollback) - 回滚部署