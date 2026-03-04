# Sub-environments

Sub-environments allow deploying multiple applications or different configurations on the same server.

## Configuration Structure

```json
{
  "environments": {
    "dev": {
      "build_command": "npm run build",
      "local_dist_path": "./dist",
      "servers": [...],
      "sub_environments": {
        "admin": {
          "build_command": "npm run build:admin",
          "remote_deploy_path": "/var/www/dev/admin"
        },
        "shop": {
          "build_command": "npm run build:shop",
          "remote_deploy_path": "/var/www/dev/shop"
        }
      }
    }
  }
}
```

## Inheritance Rules

Sub-environments inherit all settings from their parent environment but can override:

- `build_command`
- `local_dist_path`
- `remote_deploy_path`

## Deploying Sub-environments

```bash
# Deploy single sub-environment
shipfe deploy --profile dev-admin
shipfe deploy --profile dev-shop

# Deploy all sub-environments
shipfe deploy --profile dev --all-sub
```

## Directory Structure

Parent and sub-environments are deployed to different directories:

```
/var/www/dev/          # Parent environment
/var/www/dev/admin/    # Sub-environment admin
/var/www/dev/shop/     # Sub-environment shop
```

## Rolling Back Sub-environments

```bash
shipfe rollback --profile dev-admin --to 20260303_034945
```