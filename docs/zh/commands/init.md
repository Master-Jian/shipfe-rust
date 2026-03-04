# init 命令

初始化Shipfe配置文件。

## 语法

```bash
shipfe init [options]
```

## 描述

`init` 命令在当前目录创建 `shipfe.config.json` 配置文件。这个文件包含了部署所需的所有配置信息。

## 选项

| 选项 | 描述 |
|------|------|
| `-f, --force` | 强制覆盖已存在的配置文件 |
| `-h, --help` | 显示帮助信息 |

## 示例

### 基本初始化

```bash
shipfe init
```

这将在当前目录创建 `shipfe.config.json` 文件。

### 强制覆盖

```bash
shipfe init --force
```

如果配置文件已存在，将被覆盖。

## 生成的配置文件

`init` 命令会生成一个包含默认值的配置文件：

```json
{
  "remote_host": "",
  "remote_user": "",
  "remote_path": "",
  "local_path": "./dist",
  "keep_releases": 5,
  "build_command": "",
  "delete_old": false,
  "enable_shared": false,
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

## 下一步

创建配置文件后，您需要：

1. 编辑配置文件，填入您的服务器信息
2. 配置SSH密钥认证
3. 运行 `shipfe deploy` 进行首次部署

## 相关命令

- [`deploy`](/zh/commands/deploy) - 执行部署
- [`rollback`](/zh/commands/rollback) - 回滚到上一版本