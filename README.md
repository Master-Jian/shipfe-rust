# Shipfe

A powerful deployment tool for web applications with support for multiple environments and sub-environments.

## Installation

```bash
npm install -g shipfe
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
          "remote_deploy_path": "/var/www/dev",
          "delete_old": false
        }
      ],
      "remote_tmp": "/tmp"
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
          "remote_deploy_path": "/var/www/dev",
          "delete_old": false
        }
      ],
      "remote_tmp": "/tmp",
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
```

Sub-environments inherit settings from the parent environment and can override `build_command`, `local_dist_path`, and `remote_deploy_path`.

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

## License

MIT