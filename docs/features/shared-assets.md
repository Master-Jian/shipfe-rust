# Shared Assets

Shared assets reduce disk usage and network transmission through file hash deduplication.

## Enable Shared Assets

```json
{
  "enable_shared": true,
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

## How It Works

1. **File Matching**: Matches files according to `hashed_asset_patterns`
2. **Hash Calculation**: Calculates SHA256 hash for matching files
3. **Storage Strategy**:
   - Same hash files are stored only once in `shared/assets/` directory
   - Filename format: `{hash}.{ext}`, e.g., `abc123def456.js`
4. **Link Creation**: Creates hard links in release directory pointing to shared files
5. **Cleanup Mechanism**: Deletes shared files no longer referenced by any release

## Shared Assets Reset

At the start of each deployment, Shipfe automatically clears all existing resources in the `shared/` directory and recreates the `shared/assets/` directory. This ensures each deployment starts from a clean state, avoiding accumulation of old resources and potential conflicts.

## Directory Structure Example

```
remote_deploy_path/
├── shared/
│   └── assets/
│       ├── abc123def456.js
│       ├── def789ghi012.css
│       └── hij345klm678.png
├── releases/
│   ├── 20260304_120000/
│   │   ├── index.html
│   │   ├── app.js -> ../../shared/assets/abc123def456.js
│   │   └── styles.css -> ../../shared/assets/def789ghi012.css
│   └── 20260304_120100/
│       ├── index.html
│       ├── app.js -> ../../shared/assets/abc123def456.js
│       └── styles.css -> ../../shared/assets/def789ghi012.css
└── current -> releases/20260304_120100
```

## Benefits

- **Reduced Disk Usage**: Same files stored only once
- **Faster Deployments**: Unchanged files don't need re-uploading
- **Bandwidth Savings**: Only new or changed files are transmitted
- **Atomic Updates**: All hard links created simultaneously for consistency

## Notes

- Shared assets require hard link support on the server filesystem
- First enablement hashes all matching files
- Disabling shared doesn't delete existing shared files; manual cleanup required

## Use Cases

- **Enable Shared**: Large apps with frequent deployments and many static resources
- **Disable Shared**: Small apps with infrequent deployments or need for simple debugging