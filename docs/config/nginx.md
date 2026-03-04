# Nginx Configuration

Shipfe uses symbolic links and shared assets that require specific Nginx configuration.

## Basic Configuration

Configure Nginx to serve from the `current` symbolic link:

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

## Shared Assets Configuration

When `enable_shared` is enabled, configure Nginx to serve shared assets with proper caching:

```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /var/www/prod/current;
    index index.html;
    
    location / {
        try_files $uri $uri/ =404;
    }
    
    # Shared assets - long-term caching based on hash
    location /shared/assets/ {
        alias /var/www/prod/shared/assets/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

## Directory Structure

Shipfe creates this structure:
```
remote_deploy_path/
├── current -> releases/20260304_120000  # Symbolic link to current release
├── shared/
│   └── assets/                          # Shared hashed files
│       ├── abc123def456.js
│       └── def789ghi012.css
└── releases/
    ├── 20260304_120000/                 # Release directories
    └── 20260304_120100/
```

The `current` link always points to the latest successful deployment. Shared assets are referenced via hard links from release directories.