# rollback Command

The `shipfe rollback` command is used to rollback to a previous deployment version.

## Syntax

```bash
shipfe rollback --profile <name> --to <timestamp>
```

## Options

- `--profile <name>`: Specify environment configuration (required)
- `--to <timestamp>`: Target version timestamp (required)

## View Available Versions

Timestamps come from the `releases/` directory names on the server. You can view them like this:

```bash
# View on server
ls -la /path/to/deploy/releases/
```

Example output:
```
drwxr-xr-x  2 deploy deploy 4096 Mar  4 12:01 20260304_120100
drwxr-xr-x  2 deploy deploy 4096 Mar  3 15:49 20260303_154945
```

## Examples

```bash
# Rollback main environment
shipfe rollback --profile prod --to 20260303_034945

# Rollback sub-environment
shipfe rollback --profile prod-admin --to 20260303_034945
```

## Notes

- Ensure the target version exists
- Rollback updates the `current` symbolic link to point to the specified version
- No release versions are deleted, only the current pointer is changed