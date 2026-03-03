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
          "password": "your_password",
          "remote_deploy_path": "/var/www/dev",
          "delete_old": false
        }
      ],
      "remote_tmp": "/tmp",
      "sub_environments": {
        "admin": {
          "build_command": "npm run build:admin",
          "remote_deploy_path": "/var/www/dev/admin"
        }
      }
    }
  }
}
```

### Authentication

Shipfe supports multiple SSH authentication methods (tried in order):

1. **Password authentication**: Set `password` in server config
2. **SSH Private Key from environment**: Set `SSH_PRIVATE_KEY` environment variable
3. **SSH Key file**: Set `key_path` in server config

```bash
# Using password (in config)
shipfe deploy --profile dev

# Using SSH private key from environment
export SSH_PRIVATE_KEY="$(cat ~/.ssh/id_rsa)"
shipfe deploy --profile dev

# Using SSH key file (in config)
# Set "key_path": "/path/to/private/key" in server config
shipfe deploy --profile dev
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