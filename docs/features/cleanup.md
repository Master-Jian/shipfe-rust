# Automatic Cleanup

Shipfe provides configurable automatic cleanup of old releases and unused shared resources.

## Configuration Options

### `keep_releases` (Recommended)

```json
{
  "keep_releases": 5,
  "delete_old": false
}
```

- Keep the most recent 5 releases
- Automatically delete old releases
- Works with shared resource cleanup

### `delete_old` (Legacy)

```json
{
  "delete_old": true
}
```

- Delete all old releases after each deployment
- Keep only the current deployment
- Overrides `keep_releases` setting

## Cleanup Process

### Release Cleanup

1. Sort releases by modification time (newest first)
2. Keep the specified number of recent releases
3. Completely delete old release directories

### Shared Resource Cleanup (when `enable_shared: true`)

1. Scan all remaining release snapshots
2. Collect all currently referenced hashed resources
3. Delete unreferenced files from `shared/assets/`

## Cleanup Behavior Example

### Before Cleanup (7 releases)

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

### After Cleanup (`keep_releases: 3`)

```
releases/
├── 20260305_100000/  (kept)
├── 20260306_100000/  (kept)
└── 20260307_100000/  (kept, current)
```

*Old releases automatically deleted*

## Monitoring Cleanup

```bash
# Check current releases
ls -la releases/

# View cleanup logs in shipfe.log
tail -f shipfe.log | grep -i "cleanup\|remove"
```