---
home: true
heroImage: /logo.png
heroText: Shipfe
tagline: 现代化的零停机部署工具
actionText: 快速开始 →
actionLink: /zh/quick-start
features:
- title: 零停机部署
  details: 原子化部署确保应用在部署过程中保持可用性，无缝切换到新版本。
- title: 智能回滚
  details: 一键回滚到之前的稳定版本，包含完整的文件快照和元数据。
- title: 资源共享
  details: 通过文件哈希去重减少磁盘使用和网络传输，提升部署效率。
- title: 自动清理
  details: 可配置的自动清理机制，保持服务器整洁并节省存储空间。
footer: MIT Licensed | Copyright © 2024 Master-Jian
---

## 快速导航

<div class="features">
  <div class="feature">
    <h3>🚀 快速开始</h3>
    <p>5分钟内完成安装和首次部署</p>
    <a href="/zh/quick-start">开始使用</a>
  </div>
  <div class="feature">
    <h3>⚙️ 配置指南</h3>
    <p>了解所有配置选项和最佳实践</p>
    <a href="/zh/config/overview">配置文档</a>
  </div>
  <div class="feature">
    <h3>📚 命令参考</h3>
    <p>完整的命令行工具使用说明</p>
    <a href="/zh/commands/init">命令文档</a>
  </div>
</div>

## 核心特性

### 原子化部署
每次部署都是原子操作，要么完全成功，要么完全失败，确保应用稳定性。

### 智能资源管理
- **共享资源**：相同文件只存储一次，通过硬链接节省磁盘空间
- **增量部署**：只传输变更的文件，减少网络开销
- **自动清理**：可配置的清理策略，防止磁盘空间浪费

### 完善的回滚机制
- **快照记录**：每个部署生成完整文件清单和元数据
- **一键回滚**：快速恢复到任意历史版本
- **状态验证**：确保回滚后的应用状态正确

## 适用场景

- **Web应用**：前端应用、API服务、微服务
- **静态站点**：博客、文档站点、营销页面
- **混合部署**：包含静态资源和动态服务的应用

## 系统要求

- **服务器**：Linux/macOS/Windows，支持SSH访问
- **本地环境**：Node.js 16+，支持SSH密钥认证
- **网络**：稳定的SSH连接，支持文件传输