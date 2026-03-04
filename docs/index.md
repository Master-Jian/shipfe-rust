# Shipfe

[![npm version](https://img.shields.io/npm/v/shipfe.svg)](https://www.npmjs.com/package/shipfe)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-Master--Jian/shipfe--rust-blue.svg)](https://github.com/Master-Jian/shipfe-rust)

A powerful, **free**, **Rust-based** web app deployment tool that **doesn't request network**, enabling **one-click static frontend package uploads to servers**. Supports multi-environment and sub-environment deployments with zero-downtime atomic deployment functionality.

## Key Features

- 🚀 **Free & Open Source**: No hidden fees, MIT license
- 🦀 **Rust-Based**: Fast, reliable, memory-safe
- 🔒 **No Network Requests**: Works completely offline, ensuring security and privacy
- ⚡ **One-Click Deployment**: Instantly upload static frontend packages to servers
- 🔄 **Atomic Deployment**: Zero-downtime deployment with automatic rollback
- 🌍 **Multi-Environment Support**: Configure different environments (dev, staging, prod)
- 📦 **Sub-Environment Support**: Deploy multiple apps on the same server
- 🔑 **Flexible Authentication**: SSH keys, passwords, or environment variables
- 📝 **Detailed Logging**: Comprehensive deployment logs for troubleshooting
- 🗂️ **Shared Resource Management**: Cross-release deduplication of hashed static resources
- 📊 **Resource Snapshots**: Generate snapshots with file manifests for deployment auditing
- 🧹 **Automatic Cleanup**: Configurable old release retention and unused resource cleanup
- 🗑️ **Shared Resource Reset**: Automatically clears all shared resources during deployment for clean deployments

## Installation

[Installation Guide](/install)

## Quick Start

1. Initialize project:
```bash
shipfe init
```

2. Configure deployment in `shipfe.config.json`

3. Deploy:
```bash
shipfe deploy --profile prod
```

## Quick Links

- [Quick Start](/quick-start)
- [Config Overview](/config/overview)
- [Deploy Command](/commands/deploy)
- [Rollback Command](/commands/rollback)