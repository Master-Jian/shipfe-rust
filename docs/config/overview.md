# Configuration Overview

Shipfe uses the `shipfe.config.json` file for configuration.

## File Structure

```json
{
  "enable_shared": false,
  "hashed_asset_patterns": [],
  "keep_releases": 10,
  "environments": {
    "prod": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "remote_tmp": "/tmp",
      "servers": [
        {
          "host": "your-server.com",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/prod"
        }
      ]
    }
  }
}
```

## Global Configuration Items

- `enable_shared`: Whether to enable shared resource management
- `hashed_asset_patterns`: Patterns for files to be hashed
- `keep_releases`: Number of releases to keep

## Environment Configuration

Each environment contains:

- `build_command`: Local build command
- `local_dist_path`: Local build output directory
- `remote_tmp`: Remote temporary directory
- `servers`: List of servers

## Web Server Configuration

For serving deployed applications, configure your web server to point to the deployment directory. See [Nginx Configuration](nginx.md) for detailed setup instructions.

## Minimal Configuration Example

```json
{
  "environments": {
    "prod": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "servers": [
        {
          "host": "example.com",
          "username": "deploy",
          "remote_deploy_path": "/var/www/app"
        }
      ]
    }
  }
}
```