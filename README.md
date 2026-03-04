# Shipfe

[![npm version](https://img.shields.io/npm/v/shipfe.svg)](https://www.npmjs.com/package/shipfe)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-Master--Jian/shipfe--rust-blue.svg)](https://github.com/Master-Jian/shipfe-rust)

一个强大的、**免费**、**基于 Rust** 的 Web 应用部署工具，**不请求网络**，实现**一键前端静态部署包上传到服务器**。支持多环境和子环境部署，具有零停机原子部署功能。

## 安装

```bash
npm install -g shipfe
```

## 快速开始

1. 初始化项目：
```bash
shipfe init
```

2. 在 `shipfe.config.json` 中配置部署

3. 部署：
```bash
shipfe deploy --profile prod
```

## 文档

📖 [完整文档](https://master-jian.github.io/shipfe-rust/)

## 常用命令

- `shipfe deploy --profile <env>` - 部署到指定环境
- `shipfe deploy --atomic` - 原子部署
- `shipfe rollback --profile <env> --to <timestamp>` - 回滚到指定版本

## 许可证

MIT