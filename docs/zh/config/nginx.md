# Nginx 配置

Shipfe 使用符号链接和共享资源，需要特定的 Nginx 配置。

## 基本配置

配置 Nginx 从 `current` 符号链接提供服务：

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

## 共享资源配置

启用 `enable_shared` 时，配置 Nginx 以适当的缓存提供共享资源：

```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /var/www/prod/current;
    index index.html;
    
    location / {
        try_files $uri $uri/ =404;
    }
    
    # 共享资源 - 基于哈希的长期缓存
    location /shared/assets/ {
        alias /var/www/prod/shared/assets/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

## 目录结构

Shipfe 创建以下结构：
```
remote_deploy_path/
├── current -> releases/20260304_120000  # 指向当前发布版本的符号链接
├── shared/
│   └── assets/                          # 共享哈希文件
│       ├── abc123def456.js
│       └── def789ghi012.css
└── releases/
    ├── 20260304_120000/                 # 发布目录
    └── 20260304_120100/
```

`current` 链接始终指向最新的成功部署。共享资源通过发布目录中的硬链接引用。