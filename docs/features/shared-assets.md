# Shared Assets

Shared assets allow you to separate **where files are served from** (release directory) from **where they are physically stored** (`shared/`), so that frequently reused assets don't need to be re-uploaded every time.

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

- `enable_shared`: turns shared asset management on for the environment.
- `hashed_asset_patterns`: glob patterns for files that should be treated as shared assets (typically files with content hashes in their names, such as `app-abc123.js`).

## How It Works

1. **File Matching**
   - After build, Shipfe scans `local_dist_path` and matches files by `hashed_asset_patterns`.
   - All matched files are recorded as `hashed_assets` in `shipfe.snapshot.json` using their **relative paths**, e.g. `assets/app-abc123.js`, `assets/images/logo-xyz999.png`.

2. **Normal Release Extraction**
   - The full `dist` directory is always uploaded and extracted into:

     ```
     remote_deploy_path/
       releases/<timestamp>/...   # exactly the same structure as local dist
     ```

   - The web server serves from `remote_deploy_path/current`, so the URL structure matches `dist`.

3. **Shared Storage Strategy**
   - For each path `p` in `hashed_assets`:
     - Source: `releases/<timestamp>/{p}`
     - Target: `shared/{p}`
   - Shipfe moves the file and creates a hard link back:

     ```bash
     mv -f releases/<timestamp>/{p} shared/{p}
     ln -f shared/{p} releases/<timestamp>/{p}
     ```

   - This means:
     - **Release directory structure is unchanged** (paths in `current/` are identical to `dist`).
     - Shared directory mirrors the same relative paths under `shared/`.
     - If a file with the same path already exists in `shared/`, it is overwritten (`mv -f`).

4. **Link Creation**
   - Requests still hit `current/...` as usual.
   - The filesystem ensures those files physically live under `shared/...` via hard links.

5. **Cleanup**
   - Current implementation does **not** automatically delete old files under `shared/`.
   - If you want to drop history and reclaim disk space, you can manually clean `shared/` when no deployments are running.

## Directory Structure Example

```
remote_deploy_path/
├── shared/
│   └── assets/
│       ├── app-abc123.js
│       └── images/
│           └── logo-xyz999.png
├── releases/
│   └── 20260304_120100/
│       ├── index.html
│       ├── assets/
│       │   ├── app-abc123.js -> ../../../shared/assets/app-abc123.js
│       │   └── images/
│       │       └── logo-xyz999.png -> ../../../../shared/assets/images/logo-xyz999.png
│       └── ...
└── current -> releases/20260304_120100
```

## Example Patterns

- Minimal: only JS/CSS bundles

  ```json
  "hashed_asset_patterns": [
    "assets/**/*.js",
    "assets/**/*.css"
  ]
  ```

- Full assets directory (including images, fonts, etc.)

  ```json
  "hashed_asset_patterns": [
    "assets/**"
  ]
  ```

In all cases, file paths under `shared/` follow the same relative layout as under `dist`.

## Notes

- Requires filesystem support for hard links on the server.
- `hashed_asset_patterns` only controls **which files** participate in the shared mechanism; other files remain as normal files in each release.
- Shared assets are an optimization; if you prefer simpler behavior, set `enable_shared` to `false` for the environment.