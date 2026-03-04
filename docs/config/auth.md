# Authentication

Shipfe supports multiple SSH authentication methods for each server.

## Authentication Priority Order

Authentication methods are tried in this order:

1. **Password**: If `password` is set for the server
2. **SSH Private Key from Environment**: If `SSH_PRIVATE_KEY` environment variable is set (applies to all servers)
3. **SSH Key File**: If `key_path` is set for the server

## Configuration Examples

### Using Password Authentication

```json
{
  "servers": [
    {
      "host": "web1.prod.com",
      "username": "deploy",
      "password": "web1_password",
      "remote_deploy_path": "/var/www/prod"
    }
  ]
}
```

### Using SSH Key File

```json
{
  "servers": [
    {
      "host": "web2.prod.com",
      "username": "deploy",
      "key_path": "/home/user/.ssh/web2_key",
      "remote_deploy_path": "/var/www/prod"
    }
  ]
}
```

### Using Environment Variable

```bash
export SSH_PRIVATE_KEY="$(cat ~/.ssh/prod_key)"
shipfe deploy --profile prod
```

### Mixed Authentication Methods

```json
{
  "servers": [
    {
      "host": "web1.prod.com",
      "username": "deploy",
      "password": "web1_password",
      "remote_deploy_path": "/var/www/prod"
    },
    {
      "host": "web2.prod.com",
      "username": "deploy",
      "key_path": "/home/user/.ssh/web2_key",
      "remote_deploy_path": "/var/www/prod"
    },
    {
      "host": "web3.prod.com",
      "username": "deploy",
      "remote_deploy_path": "/var/www/prod"
    }
  ]
}
```

## SSH Key Setup

### Generate SSH Key Pairs

```bash
# Server 1
ssh-keygen -t rsa -b 4096 -f ~/.ssh/server1_key -C "server1"

# Server 2
ssh-keygen -t rsa -b 4096 -f ~/.ssh/server2_key -C "server2"
```

### Copy Public Keys to Servers

```bash
ssh-copy-id -i ~/.ssh/server1_key.pub user@server1.com
ssh-copy-id -i ~/.ssh/server2_key.pub user@server2.com
```

### Configure Server-Specific Keys

```json
{
  "servers": [
    {
      "host": "server1.com",
      "key_path": "~/.ssh/server1_key"
    },
    {
      "host": "server2.com",
      "key_path": "~/.ssh/server2_key"
    }
  ]
}
```