import { defineConfig } from "vitepress";

export default defineConfig({
  // 重点：项目站 base 必须是 /仓库名/
  // 例：仓库是 shipfe-rust → base: "/shipfe-rust/"
  base: "/shipfe-rust/",
  title: "Shipfe",
  locales: {
    root: {
      label: 'English',
      lang: 'en',
      title: 'Shipfe',
      description: 'A powerful, free, Rust-based web app deployment tool',
      themeConfig: {
        nav: [
          { text: "Quick Start", link: "/quick-start" },
          { text: "Config", link: "/config/overview" },
          { text: "GitHub", link: "https://github.com/Master-Jian/shipfe-rust" }
        ],
        sidebar: [
          {
            text: "Getting Started",
            items: [
              { text: "Introduction", link: "/" },
              { text: "Quick Start", link: "/quick-start" },
              { text: "Installation", link: "/install" }
            ]
          },
          {
            text: "Commands",
            items: [
              { text: "init", link: "/commands/init" },
              { text: "deploy", link: "/commands/deploy" },
              { text: "rollback", link: "/commands/rollback" }
            ]
          },
          {
            text: "Configuration",
            items: [
              { text: "Config Overview", link: "/config/overview" },
              { text: "Config Schema", link: "/config/schema" },
              { text: "Authentication", link: "/config/auth" },
              { text: "Sub-environments", link: "/config/sub-env" }
            ]
          },
          {
            text: "Features",
            items: [
              { text: "Atomic Deployment", link: "/features/atomic" },
              { text: "Shared Assets", link: "/features/shared-assets" },
              { text: "Snapshots", link: "/features/snapshot" },
              { text: "Cleanup", link: "/features/cleanup" }
            ]
          },
          {
            text: "Operations",
            items: [
              { text: "Troubleshooting", link: "/troubleshooting" }
            ]
          }
        ]
      }
    },
    zh: {
      label: '中文',
      lang: 'zh-CN',
      title: 'Shipfe',
      description: '一个强大的、免费的、基于 Rust 的 Web 应用部署工具',
      themeConfig: {
        nav: [
          { text: "快速入门", link: "/zh/quick-start" },
          { text: "配置", link: "/zh/config/overview" },
          { text: "GitHub", link: "https://github.com/Master-Jian/shipfe-rust" }
        ],
        sidebar: [
          {
            text: "开始",
            items: [
              { text: "简介", link: "/zh/" },
              { text: "快速入门", link: "/zh/quick-start" },
              { text: "安装", link: "/zh/install" }
            ]
          },
          {
            text: "命令",
            items: [
              { text: "init", link: "/zh/commands/init" },
              { text: "deploy", link: "/zh/commands/deploy" },
              { text: "rollback", link: "/zh/commands/rollback" }
            ]
          },
          {
            text: "配置",
            items: [
              { text: "配置概览", link: "/zh/config/overview" },
              { text: "配置项详解", link: "/zh/config/schema" },
              { text: "认证", link: "/zh/config/auth" },
              { text: "子环境", link: "/zh/config/sub-env" }
            ]
          },
          {
            text: "功能",
            items: [
              { text: "原子部署", link: "/zh/features/atomic" },
              { text: "共享资源管理", link: "/zh/features/shared-assets" },
              { text: "资源快照", link: "/zh/features/snapshot" },
              { text: "自动清理", link: "/zh/features/cleanup" }
            ]
          },
          {
            text: "运维",
            items: [
              { text: "故障排除", link: "/zh/troubleshooting" }
            ]
          }
        ]
      }
    }
  }
});