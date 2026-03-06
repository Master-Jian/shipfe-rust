# 快速开始

本指南将帮助您在5分钟内完成Shipfe的安装和首次部署。

## 安装

### 使用npm安装（项目级推荐）

在你的前端项目根目录安装为开发依赖，并通过 `npx` 使用：

```bash
npm install --save-dev shipfe
npx shipfe --version
```

### 从源码构建

```bash
git clone https://github.com/Master-Jian/shipfe.git
cd shipfe
cargo build --release
cp target/release/shipfe /usr/local/bin/shipfe
```

## 初始化项目

在您的项目根目录下运行：

```bash
npx shipfe init
```

这将创建 `shipfe.config.json` 配置文件。

## 配置

编辑 `shipfe.config.json`：

```json
{
  "remote_host": "your-server.com",
  "remote_user": "deploy",
  "remote_path": "/var/www/myapp",
  "local_path": "./dist",
  "keep_releases": 5,
  "build_command": "npm run build"
}
```

### 配置说明

- `remote_host`: 服务器地址
- `remote_user`: SSH用户名
- `remote_path`: 服务器部署目录
- `local_path`: 本地构建输出目录
- `keep_releases`: 保留的发布版本数量
- `build_command`: 构建命令

## SSH密钥配置

确保您的SSH密钥已添加到服务器：

```bash
# 本地生成SSH密钥（如果还没有）
ssh-keygen -t rsa -b 4096 -C "your-email@example.com"

# 复制公钥到服务器
ssh-copy-id user@your-server.com
```

## 首次部署

运行构建和部署：

```bash
npx shipfe deploy
```

Shipfe将：
1. 执行构建命令
2. 创建新的发布目录
3. 上传文件到服务器
4. 原子化切换到新版本
5. 清理旧版本

## 验证部署

检查部署状态：

```bash
# 查看服务器上的发布
ssh user@server "ls -la /var/www/myapp/releases/"

# 检查当前符号链接
ssh user@server "ls -la /var/www/myapp/current"
```

## 回滚（如果需要）

如果部署出现问题，可以快速回滚：

```bash
npx shipfe rollback
```

这将自动切换到上一个稳定版本。

## 下一步

- [配置详解](/zh/config/overview) - 了解所有配置选项
- [命令参考](/zh/commands/deploy) - 掌握所有命令
- [故障排除](/zh/troubleshooting) - 解决常见问题