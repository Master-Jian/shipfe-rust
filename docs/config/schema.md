# Configuration Schema

## Global Configuration Items

### `enable_shared` (boolean, default: false)

Whether to enable shared resource management. When enabled, matching files are hashed and stored in a shared directory to avoid duplicate uploads.

Suitable for applications that deploy frequently and have many static resources.

### `hashed_asset_patterns` (array of strings, default: [])

Specify file patterns for static resources to be hashed. Supports glob patterns like `"**/*.js"`, `"**/*.css"`, `"**/*.{png,jpg,svg}"`.

Only files matching these patterns will be hashed and shared; non-matching files are uploaded fresh with each deployment.

### `keep_releases` (number, default: 10)

Number of release versions to retain. Older releases beyond this number are automatically cleaned up. Set to 0 to disable automatic cleanup.

### `delete_old` (boolean, default: false)

Whether to delete all old releases after each deployment. Only the current release is kept. Overrides `keep_releases` setting.

## Environment Configuration Items

### `build_command` (string, required)

Local build command, such as `"npm run build"` or `"yarn build"`.

### `local_dist_path` (string, required)

Local build output directory path, such as `"./dist"` or `"./build"`.

### `servers` (array of objects, required)

List of servers, each containing:

- `host` (string, required): Server hostname or IP address
- `port` (number, default: 22): SSH port
- `username` (string, required): SSH username
- `password` (string, optional): SSH password (not recommended, prefer keys)
- `key_path` (string, optional): SSH private key file path
- `remote_deploy_path` (string, required): Deployment directory path on server

### `remote_tmp` (string, default: "/tmp")

Remote temporary directory path for file uploads.

### `sub_environments` (object, optional)

Sub-environment configuration. Keys are sub-environment names, values are sub-environment config objects. Sub-environments inherit parent settings but can override `build_command`, `local_dist_path`, `remote_deploy_path`.