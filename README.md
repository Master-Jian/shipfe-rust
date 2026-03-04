# Shipfe

[![npm version](https://img.shields.io/npm/v/shipfe.svg)](https://www.npmjs.com/package/shipfe)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-Master--Jian/shipfe--rust-blue.svg)](https://github.com/Master-Jian/shipfe-rust)

A powerful, **free**, **Rust-based** deployment tool for web applications that **never requests network** and enables **one-click static frontend deployment** to servers. Supports multiple environments and sub-environments with zero-downtime atomic deployments.

## Key Features

- 🚀 **Free and Open Source**: No hidden costs, MIT licensed
- 🦀 **Built with Rust**: Fast, reliable, and memory-safe
- 🔒 **No Network Requests**: Works completely offline, ensuring security and privacy
- ⚡ **One-Click Deployment**: Upload static frontend packages to servers instantly
- 🔄 **Atomic Deployments**: Zero-downtime deployments with automatic rollback
- 🌍 **Multi-Environment Support**: Configure different environments (dev, staging, prod)
- 📦 **Sub-Environment Support**: Deploy multiple apps to the same server
- 🔑 **Flexible Authentication**: SSH key, password, or environment variable authentication
- 📝 **Detailed Logging**: Comprehensive deployment logs for troubleshooting
- 🗂️ **Shared Asset Management**: Deduplicate hashed static assets across releases
- 📊 **Resource Snapshot**: Generate deployment snapshots with file manifests
- 🧹 **Automatic Cleanup**: Configurable retention of old releases and unused assets

## Installation

```bash
npm install -g shipfe
```

## Quick Start

1. Initialize your project:
```bash
shipfe init
```

2. Configure your deployment in `shipfe.config.json`

3. Deploy:
```bash
shipfe deploy --profile prod
```

## Usage

### Initialize project
```bash
shipfe init
```

### Deploy to environment
```bash
# Deploy to default environment
shipfe deploy

# Deploy to specific environment
shipfe deploy --profile dev

# Deploy to sub-environment
shipfe deploy --profile dev-admin

# Deploy to all sub-environments
shipfe deploy --profile dev --all-sub

# Atomic deployment (creates releases/timestamp and updates current symlink)
shipfe deploy --atomic
```

### Rollback Deployment
```bash
# Rollback main environment
shipfe rollback --profile prod --to 20260303_034945

# Rollback sub-environment
shipfe rollback --profile prod-admin --to 20260303_034945
```

### Configuration

Edit `shipfe.config.json` to configure your deployment settings:

```json
{
  "environments": {
    "dev": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "servers": [
        {
          "host": "dev.example.com",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/dev"
        }
      ],
      "remote_tmp": "/tmp",
      "hashed_asset_patterns": ["assets/"],
      "enable_shared": true,
      "keep_releases": 5,
      "delete_old": false
    }
  }
}
```

#### Authentication Options

Each server can have its own authentication method. Shipfe tries authentication methods in this order:

1. **Password** (if `password` is set in server config)
2. **SSH Private Key from environment** (if `SSH_PRIVATE_KEY` env var is set)
3. **SSH Key file** (if `key_path` is set in server config)

**Example with multiple servers using different auth methods:**

```json
{
  "environments": {
    "prod": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "servers": [
        {
          "host": "web1.prod.com",
          "port": 22,
          "username": "deploy",
          "password": "web1_password",
          "remote_deploy_path": "/var/www/prod",
          "delete_old": false
        },
        {
          "host": "web2.prod.com",
          "port": 22,
          "username": "deploy",
          "key_path": "/home/user/.ssh/web2_key",
          "remote_deploy_path": "/var/www/prod",
          "delete_old": false
        },
        {
          "host": "web3.prod.com",
          "port": 22,
          "username": "deploy",
          "key_path": "/home/user/.ssh/web3_key",
          "remote_deploy_path": "/var/www/prod",
          "delete_old": false
        }
      ],
      "remote_tmp": "/tmp"
    }
  }
}

### Sub-environments

For deploying multiple applications or different configurations to the same server, use sub-environments:

```json
{
  "environments": {
    "dev": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "servers": [
        {
          "host": "dev.example.com",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/dev"
        }
      ],
      "remote_tmp": "/tmp",
      "hashed_asset_patterns": ["assets/"],
      "enable_shared": true,
      "keep_releases": 5,
      "delete_old": false,
      "sub_environments": {
        "admin": {
          "build_command": "npm run build:admin",
          "remote_deploy_path": "/var/www/dev/admin"
        },
        "shop": {
          "build_command": "npm run build:shop",
          "remote_deploy_path": "/var/www/dev/shop"
        },
        "cu": {
          "build_command": "npm run build:cu",
          "remote_deploy_path": "/var/www/dev/cu"
        }
      }
    }
  }
}
```

Deploy to sub-environments:
```bash
shipfe deploy --profile dev-admin
shipfe deploy --profile dev-shop
shipfe deploy --profile dev-cu

# Deploy to all sub-environments at once
shipfe deploy --profile dev --all-sub
```

### Shared Assets Management

Shipfe supports deduplication of hashed static assets across releases to save disk space and bandwidth. When `enable_shared` is set to `true`, hashed assets are stored in a shared directory and hard-linked in each release.

**Configuration:**
```json
{
  "environments": {
    "prod": {
      "enable_shared": true,
      "hashed_asset_patterns": ["assets/"],
      "keep_releases": 5,
      "delete_old": false
    }
  }
}
```

**Features:**
- **Automatic Detection**: Detects hashed files based on patterns or filename format (`-hash.ext`)
- **Deduplication**: Same hashed assets are shared across releases
- **Cleanup**: Unused assets are automatically removed when no longer referenced
- **Retention**: Configurable number of releases to keep (`keep_releases`)

### Resource Snapshots

Each deployment generates a `shipfe.snapshot.json` file containing the complete manifest of deployed files and hashed assets.

**Example snapshot:**
```json
{
  "id": "20260303_035045",
  "timestamp": "2026-03-03T03:50:45Z",
  "files": ["index.html", "assets/app-abc123.js", "assets/style-def456.css"],
  "hashed_assets": ["assets/app-abc123.js", "assets/style-def456.css"]
}
```

This enables:
- **Asset tracking**: Know exactly what files are deployed
- **Integrity verification**: Verify deployed files match build output
- **Rollback validation**: Ensure rollback targets have correct assets

### Atomic Deployment

Shipfe supports atomic deployment to minimize downtime. When using `--atomic`, the deployment creates a timestamped release directory and updates a `current` symlink for zero-downtime switching.

```bash
# Atomic deployment to default environment
shipfe deploy --atomic

# Atomic deployment to specific environment
shipfe deploy --profile prod --atomic
```

**Directory Structure:**
```
remote_deploy_path/
├── releases/
│   ├── 20260303_034945/
│   │   ├── index.html
│   │   ├── assets/
│   │   │   └── app.js -> ../../../shared/assets/app-abc123.js
│   │   └── shipfe.snapshot.json
│   ├── 20260303_035012/
│   └── 20260303_035045/
├── shared/
│   └── assets/
│       ├── app-abc123.js
│       └── style-def456.css
└── current -> releases/20260303_035045
```

Your web server should serve from `remote_deploy_path/current`.

**Or use environment variable for all servers:**
```bash
export SSH_PRIVATE_KEY="$(cat ~/.ssh/prod_key)"
shipfe deploy --profile prod
```

### Authentication

Shipfe supports multiple SSH authentication methods for each server individually. For each server, authentication methods are tried in this order:

1. **Password authentication**: If `password` is set in that server's config
2. **SSH Private Key from environment**: If `SSH_PRIVATE_KEY` environment variable is set (applies to all servers)
3. **SSH Key file**: If `key_path` is set in that server's config

#### Usage Examples:

```bash
# Each server can use different authentication
shipfe deploy --profile prod

# Or override with environment variable for all servers
export SSH_PRIVATE_KEY="$(cat ~/.ssh/prod_key)"
shipfe deploy --profile prod
```

#### SSH Key Setup for Individual Servers:

1. **Generate SSH key pairs** for each server:
   ```bash
   # For server 1
   ssh-keygen -t rsa -b 4096 -f ~/.ssh/server1_key -C "server1"

   # For server 2
   ssh-keygen -t rsa -b 4096 -f ~/.ssh/server2_key -C "server2"
   ```

2. **Copy public keys to respective servers**:
   ```bash
   ssh-copy-id -i ~/.ssh/server1_key.pub user@server1.com
   ssh-copy-id -i ~/.ssh/server2_key.pub user@server2.com
   ```

3. **Configure shipfe** with server-specific keys:
   ```json
   {
     "servers": [
       {
         "host": "server1.com",
         "key_path": "~/.ssh/server1_key"
       },
       {
         "host": "server2.com",
         "key_path": "~/.ssh/server2_key"
       }
     ]
   }
   ```

## Features

- Multiple environment support
- Sub-environment configuration
- Custom build commands
- SSH-based deployment
- Automatic backup and rollback
- Detailed logging
- Shared asset deduplication
- Resource snapshot generation
- Configurable release retention
- Automatic cleanup of unused assets

## License

MIT