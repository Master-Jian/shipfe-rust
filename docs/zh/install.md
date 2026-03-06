# 安装

Shipfe 支持多种安装方式，选择最适合您的方式。

## 系统要求

- **操作系统**: Linux, macOS, Windows
- **Node.js**: 版本 16 或更高
- **SSH**: 支持密钥认证
- **服务器**: 支持SSH访问的Linux/macOS服务器

## 方式一：npm安装（推荐）

### 项目级安装（推荐）

在你的前端项目根目录中安装为开发依赖：

```bash
npm install --save-dev shipfe
```

推荐直接使用 `npx` 调用：

```bash
npx shipfe --version
npx shipfe --help
```

或者在 `package.json` 中添加脚本：

```json
{
  "scripts": {
    "shipfe:init": "shipfe init",
    "shipfe:deploy": "shipfe deploy",
    "shipfe:rollback": "shipfe rollback"
  }
}
```

然后通过 npm 脚本使用：

```bash
npm run shipfe:init
npm run shipfe:deploy -- --profile prod
```

### 全局安装（可选）

如果你更习惯全局 CLI，也可以选择全局安装（切换 Node 版本或环境时需要注意）：

```bash
npm install -g shipfe
```

## 方式二：从源码构建

### 克隆仓库

```bash
git clone https://github.com/Master-Jian/shipfe.git
cd shipfe
```

### 安装Rust

如果还没有安装Rust：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 构建和安装

```bash
# 构建发布版本
cargo build --release

# 安装到系统路径
cp target/release/shipfe /usr/local/bin/shipfe

# 或者安装到用户目录
cp target/release/shipfe ~/.local/bin/shipfe
```

### 验证安装

```bash
shipfe --version
```

## 方式三：下载预构建二进制文件

访问 [GitHub Releases](https://github.com/Master-Jian/shipfe/releases) 下载适合您平台的二进制文件。

```bash
# 下载并安装（以Linux x64为例）
wget https://github.com/Master-Jian/shipfe/releases/download/v1.0.0/shipfe-linux-x64
chmod +x shipfe-linux-x64
sudo mv shipfe-linux-x64 /usr/local/bin/shipfe
```

## 配置自动补全（可选）

### Bash

```bash
# 添加到 ~/.bashrc
echo 'source <(shipfe completion bash)' >> ~/.bashrc
source ~/.bashrc
```

### Zsh

```bash
# 添加到 ~/.zshrc
echo 'source <(shipfe completion zsh)' >> ~/.zshrc
source ~/.zshrc
```

### Fish

```bash
# 添加到 ~/.config/fish/config.fish
echo 'shipfe completion fish | source' >> ~/.config/fish/config.fish
```

## 验证安装

运行以下命令验证安装是否成功：

```bash
# 检查版本
shipfe --version

# 查看帮助
shipfe --help

# 查看所有可用命令
shipfe --help
```

## 故障排除

### 权限问题

使用项目级安装时通常不会遇到权限问题。如果你确实需要全局安装并遇到权限错误，可以：

```bash
# 使用sudo安装（不推荐）
sudo npm install -g shipfe

# 或者安装到用户目录
npm config set prefix ~/.npm-global
export PATH=~/.npm-global/bin:$PATH
```

### 网络问题

如果npm安装失败，尝试使用国内镜像（同样推荐项目级安装）：

```bash
npm config set registry https://registry.npmmirror.com
npm install --save-dev shipfe
```

### Rust编译问题

如果源码构建失败：

```bash
# 更新Rust
rustup update

# 清理并重新构建
cargo clean
cargo build --release
```