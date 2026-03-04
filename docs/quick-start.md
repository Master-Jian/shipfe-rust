# Quick Start

This page aims to complete a full deployment cycle (init → config → deploy → verify → rollback) with minimal configuration.

## 1. Installation

```bash
npm install -g shipfe
shipfe --version
```

## 2. Initialize Configuration

Run in your project root directory:

```bash
shipfe init
```

This will generate (example):

```
shipfe.config.json
```

## 3. Write a Minimal Viable Configuration (Single Server, Single Environment)

Edit `shipfe.config.json`:

```json
{
  "keep_releases": 5,
  "environments": {
    "prod": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "remote_tmp": "/tmp",
      "servers": [
        {
          "host": "YOUR_SERVER_IP",
          "port": 22,
          "username": "deploy",
          "remote_deploy_path": "/var/www/shipfe-demo"
        }
      ]
    }
  }
}
```

## 4. First Deployment (Atomic Deployment Recommended)

```bash
shipfe deploy --profile prod --atomic
```

After deployment, the server directory will typically be:

```
/var/www/shipfe-demo/
  releases/
    20260304_120100/
  current -> releases/20260304_120100
```

Your Nginx/static server should serve from:

```
/var/www/shipfe-demo/current
```

## 5. Verify Deployment

Check on the server:

```bash
ls -la /var/www/shipfe-demo
ls -la /var/www/shipfe-demo/current
```

Or directly access your domain/server address.

## 6. Rollback to a Specific Version

Rollback to a specific timestamp version (example):

```bash
shipfe rollback --profile prod --to 20260303_034945
```

Timestamps come from the `releases/` directory names.

## Next Steps

- Config details: [/config/overview](/config/overview)
- Deploy command parameters: [/commands/deploy](/commands/deploy)
- Atomic deployment principles: [/features/atomic](/features/atomic)
- Shared asset management: [/features/shared-assets](/features/shared-assets)