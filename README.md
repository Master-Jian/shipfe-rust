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

## Features

- Multiple environment support
- Sub-environment configuration
- Custom build commands
- SSH-based deployment
- Automatic backup and rollback
- Detailed logging

## License

MIT