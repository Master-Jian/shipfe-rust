# Atomic Deployment

Atomic deployment enables zero-downtime deployment through symbolic link switching for consistency.

## How It Works

When using `--atomic`, deployment:

1. Creates a timestamped release directory: `releases/20260303_034945/`
2. Uploads files to the new directory
3. Updates the `current` symbolic link to point to the new directory

## Directory Structure

```
remote_deploy_path/
├── releases/
│   ├── 20260303_034945/
│   ├── 20260303_035012/
│   └── 20260303_035045/
└── current -> releases/20260303_035045
```

## Web Server Configuration

Your web server should serve from `remote_deploy_path/current`.

### Nginx Example

```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /var/www/prod/current;
    index index.html;
    
    location / {
        try_files $uri $uri/ =404;
    }
}
```

### Apache Example

```apache
<VirtualHost *:80>
    ServerName your-domain.com
    DocumentRoot /var/www/prod/current
</VirtualHost>
```

## Benefits

- **Zero Downtime**: Switching happens instantly
- **Atomicity**: Either everything succeeds or everything fails
- **Instant Rollback**: Just change the symbolic link
- **Version Isolation**: Each release is stored independently

## Usage

```bash
# Atomic deployment to default environment
shipfe deploy --atomic

# Atomic deployment to specified environment
shipfe deploy --profile prod --atomic
```