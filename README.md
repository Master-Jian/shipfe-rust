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
- 🗑️ **Shared Asset Reset**: Automatically clear all shared assets on deployment for clean state

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
          "remote_deploy_path": "/var/www/prod"
        },
        {
          "host": "web2.prod.com",
          "port": 22,
          "username": "deploy",
          "key_path": "/home/user/.ssh/web2_key",
          "remote_deploy_path": "/var/www/prod"
        },
        {
          "host": "web3.prod.com",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/prod"
        }
      ],
      "remote_tmp": "/tmp",
      "delete_old": false
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

### Configuration Details

Here are all available configuration options in `shipfe.config.json`:

#### Global Configuration Options

- **`enable_shared`** (boolean, default: false)
  - Enable shared asset management
  - When enabled, matching files are hashed and stored in a shared directory to avoid re-uploading duplicates
  - Suitable for applications with frequent deployments and many static assets

- **`hashed_asset_patterns`** (array of strings, default: [])
  - File patterns for assets that should be hashed and shared
  - Supports glob patterns like `"**/*.js"`, `"**/*.css"`, `"**/*.{png,jpg,svg}"`
  - Only matching files will be hashed and shared; others are uploaded fresh each deployment

- **`keep_releases`** (number, default: 10)
  - Number of release versions to keep
  - Older releases beyond this number will be automatically cleaned up
  - Set to 0 to disable automatic cleanup

#### Environment Configuration Options

Each environment (like `dev`, `prod`) can contain these options:

- **`build_command`** (string, required)
  - Local build command, e.g., `"npm run build"` or `"yarn build"`

- **`local_dist_path`** (string, required)
  - Local build output directory path, e.g., `"./dist"` or `"./build"`

- **`servers`** (array of objects, required)
  - List of servers, each containing:
    - **`host`** (string, required): Server hostname or IP address
    - **`port`** (number, default: 22): SSH port
    - **`username`** (string, required): SSH username
    - **`password`** (string, optional): SSH password (not recommended, use keys instead)
    - **`key_path`** (string, optional): Path to SSH private key file
    - **`remote_deploy_path`** (string, required): Deployment directory path on server

- **`remote_tmp`** (string, default: "/tmp")
  - Temporary directory path on server for file uploads

- **`sub_environments`** (object, optional)
  - Sub-environment configurations, keyed by sub-environment name
  - Sub-environments inherit parent settings but can override `build_command`, `local_dist_path`, `remote_deploy_path`

- **`delete_old`** (boolean, default: false)
  - Delete all old releases after each deployment
  - Keep only the current deployment
  - Overrides `keep_releases` setting

#### Shared Assets and Hashing Configuration Explained

**How Shared Assets Work:**

1. **File Matching**: Files are matched against `hashed_asset_patterns`
2. **Hash Calculation**: SHA256 hashes are computed for matching files
3. **Storage Strategy**:
   - Identical files are stored only once in the `shared/assets/` directory
   - Filename format: `{hash}.{ext}`, e.g., `abc123def456.js`
4. **Link Creation**: Hard links are created in release directories pointing to shared files
5. **Cleanup**: Shared files no longer referenced by any kept release are removed

**Configuration Example with Explanations:**

```json
{
  "enable_shared": true,
  "hashed_asset_patterns": [
    "**/*.js",           // Match all JS files
    "**/*.css",          // Match all CSS files
    "**/*.{png,jpg}",    // Match PNG and JPG files
    "!**/vendor/**"      // Exclude vendor directory (if not needed for sharing)
  ],
  "keep_releases": 5
}
```

**When to Use:**
- **Enable sharing**: Large applications, frequent deployments, many static assets
- **Disable sharing**: Small applications, infrequent deployments, or need for simple debugging

**Important Notes:**
- Shared assets require filesystem support for hard links
- On first enable, all matching files will be hashed
- Disabling sharing doesn't remove existing shared files (manual cleanup required)

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

**How it works:**
1. **Asset Detection**: Automatically detects hashed files based on:
   - User-defined patterns (e.g., `["assets/", "static/"]`)
   - Filename format: `name-hash.ext` (e.g., `app-abc123.js`, `style-def456.css`)

2. **Deduplication Process**:
   - First deployment: Copies hashed assets to `shared/assets/`
   - Subsequent deployments: Creates hard links to existing shared assets
   - Same hash = same file, no duplication

3. **Shared Asset Reset**: At the start of each deployment, all existing assets in the `shared/` directory are cleared to ensure a clean state, then `shared/assets/` is recreated.

4. **Automatic Cleanup**: Removes unused assets when no longer referenced by any kept release

**Benefits:**
- **Disk Space**: Saves storage by eliminating duplicate hashed files
- **Bandwidth**: Faster deployments with smaller transfer sizes
- **Performance**: Hard links provide instant access with minimal overhead

**Example Directory Structure:**
```
remote_deploy_path/
├── releases/
│   ├── 20260304_120000/
│   │   ├── index.html
│   │   └── assets/
│   │       ├── app.js -> ../../../shared/assets/app-abc123.js
│   │       └── style.css -> ../../../shared/assets/style-def456.css
│   └── 20260304_120100/
│       ├── index.html
│       └── assets/
│           ├── app.js -> ../../../shared/assets/app-abc123.js  (same file)
│           └── style.css -> ../../../shared/assets/style-ghi789.css  (new)
├── shared/
│   └── assets/
│       ├── app-abc123.js
│       ├── style-def456.css
│       └── style-ghi789.css
└── current -> releases/20260304_120100
```

### Automatic Cleanup

Shipfe provides configurable automatic cleanup of old releases and unused shared assets to manage disk usage efficiently.

**Configuration Options:**

1. **`keep_releases`** (recommended):
   ```json
   {
     "keep_releases": 5,
     "delete_old": false
   }
   ```
   - Keeps the 5 most recent releases
   - Automatically removes older releases
   - Works with shared assets cleanup

2. **`delete_old`** (legacy):
   ```json
   {
     "delete_old": true
   }
   ```
   - Removes ALL old releases after each deployment
   - Only keeps the current deployment
   - Overrides `keep_releases` setting

**Cleanup Process:**

1. **Release Cleanup**:
   - Sorts releases by modification time (newest first)
   - Keeps specified number of recent releases
   - Removes older release directories completely

2. **Shared Assets Cleanup** (when `enable_shared: true`):
   - Scans all remaining release snapshots
   - Collects all currently referenced hashed assets
   - Removes unreferenced files from `shared/assets/`

**Example Cleanup Behavior:**

**Before cleanup (7 releases):**
```
releases/
├── 20260301_100000/  (oldest)
├── 20260302_100000/
├── 20260303_100000/
├── 20260304_100000/
├── 20260305_100000/
├── 20260306_100000/
└── 20260307_100000/  (newest, current)
```

**After cleanup (`keep_releases: 3`):**
```
releases/
├── 20260305_100000/  (kept)
├── 20260306_100000/  (kept)
└── 20260307_100000/  (kept, current)
```
*Older releases automatically removed*

**Monitoring Cleanup:**
```bash
# Check current releases
ls -la releases/

# View cleanup logs in shipfe.log
tail -f shipfe.log | grep -i "cleanup\|remove"
```

## Configuration Best Practices

### For Production Deployments

**Recommended Configuration:**
```json
{
  "enable_shared": true,
  "keep_releases": 10,
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

**Why this works:**
- Shared assets reduce disk usage by 60-80%
- 10 releases provide rollback capability
- Hashed patterns cover common static assets
- Automatic cleanup prevents disk space issues

### For Development/Staging

**Recommended Configuration:**
```json
{
  "enable_shared": false,
  "keep_releases": 3,
  "delete_old": false
}
```

**Why this works:**
- Faster deployments (no asset hashing)
- Fewer releases to manage
- Easier debugging without shared assets complexity

### For Memory-Constrained Servers

**Recommended Configuration:**
```json
{
  "enable_shared": true,
  "keep_releases": 2,
  "hashed_asset_patterns": ["**/*.{js,css}"]
}
```

**Why this works:**
- Minimal releases reduce storage
- Only essential assets shared
- Balances performance and disk usage

## Troubleshooting

### Common Issues

**1. Permission Denied Errors**
```
Error: Permission denied (publickey)
```
**Solutions:**
- Verify SSH key is added to `~/.ssh/authorized_keys` on server
- Check SSH key permissions: `chmod 600 ~/.ssh/id_rsa`
- Test SSH connection: `ssh user@host`

**2. Shared Assets Not Working**
```
Warning: Failed to create hard link for shared asset
```
**Solutions:**
- Ensure `enable_shared: true` in config
- Check server filesystem supports hard links
- Verify write permissions on shared directory

**3. Cleanup Not Working**
```
Warning: Failed to remove old release
```
**Solutions:**
- Check file permissions on releases directory
- Ensure no processes are using old release files
- Verify `keep_releases` is set correctly

**4. Snapshot Creation Fails**
```
Error: Failed to create snapshot
```
**Solutions:**
- Check available disk space
- Verify write permissions on releases directory
- Ensure tar command is available on server

### Debug Mode

Enable detailed logging:
```bash
shipfe deploy --debug
```

Check logs:
```bash
tail -f shipfe.log
```

### Performance Optimization

**Slow Deployments:**
- Enable shared assets for large static files
- Use `hashed_asset_patterns` to target specific files
- Consider excluding large non-changing files from patterns

**High Disk Usage:**
- Reduce `keep_releases` count
- Enable shared assets
- Use `delete_old: true` for single-release deployments

**Network Issues:**
- Compress large files before deployment
- Use faster SSH ciphers if supported
- Consider deploying during off-peak hours

### Resource Snapshots

Each deployment generates a `shipfe.snapshot.json` file containing the complete manifest of deployed files and hashed assets. This snapshot provides full visibility into what was deployed and enables various operational capabilities.

**Example snapshot:**
```json
{
  "id": "20260303_035045",
  "timestamp": "2026-03-03T03:50:45Z",
  "files": [
    "index.html",
    "manifest.json",
    "assets/app-abc123.js",
    "assets/style-def456.css",
    "assets/logo.png"
  ],
  "hashed_assets": [
    "assets/app-abc123.js",
    "assets/style-def456.css"
  ]
}
```

**Snapshot Fields:**
- **`id`**: Unique deployment identifier (timestamp-based)
- **`timestamp`**: ISO 8601 timestamp of deployment
- **`files`**: Complete list of all deployed files
- **`hashed_assets`**: Subset of files identified as hashed/cacheable assets

**Use Cases:**

1. **Deployment Verification**:
   ```bash
   # Check what was deployed in a specific release
   cat releases/20260303_035045/shipfe.snapshot.json
   ```

2. **Asset Inventory**:
   ```bash
   # List all hashed assets across all releases
   find releases/ -name "shipfe.snapshot.json" -exec jq -r '.hashed_assets[]' {} \; | sort | uniq
   ```

3. **Rollback Validation**:
   ```bash
   # Verify rollback target has expected assets
   jq '.files[]' releases/20260303_035045/shipfe.snapshot.json
   ```

4. **Storage Analysis**:
   ```bash
   # Calculate deployment sizes
   find releases/20260303_035045/ -type f -exec ls -lh {} \; | awk '{sum += $5} END {print sum}'
   ```

**Integration with CI/CD:**
Snapshots enable automated verification in deployment pipelines:
- Compare deployed files against build artifacts
- Validate asset integrity post-deployment
- Generate deployment reports and changelogs

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