# deploy Command

The `shipfe deploy` command is used to deploy applications to servers.

## Syntax

```bash
shipfe deploy [options]
```

## Options

- `--profile <name>`: Specify environment configuration (e.g., `prod`, `dev`)
- `--all-sub`: Deploy to all sub-environments

## Examples

```bash
# Deploy to default environment
shipfe deploy

# Deploy to specified environment
shipfe deploy --profile dev

# Deploy to sub-environment
shipfe deploy --profile dev-admin

# Deploy to all sub-environments
shipfe deploy --profile dev --all-sub
```

## Workflow

1. Execute local build command (`build_command`)
2. Upload build artifacts to server
3. Create new release directory on server
4. Update symbolic link (for atomic deployment)
5. Clean up old versions (based on configuration)