# Troubleshooting

## Common Issues

### 1. Permission Denied Error

```
Error: Permission denied (publickey)
```

**Solutions:**
- Verify SSH key is added to server's `~/.ssh/authorized_keys`
- Check SSH key permissions: `chmod 600 ~/.ssh/id_rsa`
- Test SSH connection: `ssh user@host`

### 2. Shared Resources Not Working

```
Warning: Failed to create hard link for shared asset
```

**Solutions:**
- Ensure `enable_shared: true` in configuration
- Check if server filesystem supports hard links
- Verify write permissions for shared directory

### 3. Cleanup Not Working

```
Warning: Failed to remove old release
```

**Solutions:**
- Check file permissions in releases directory
- Ensure no processes are using old release files
- Verify `keep_releases` setting is correct

### 4. Snapshot Creation Failed

```
Error: Failed to create snapshot
```

**Solutions:**
- Check available disk space
- Verify write permissions for releases directory
- Ensure tar command is available on server

## Debug Mode

Enable verbose logging:

```bash
shipfe deploy --debug
```

## Log Paths

Shipfe log files are typically located at:

- Local: `shipfe.log`
- Server: `shipfe.log` in deployment directory

Check logs:

```bash
tail -f shipfe.log
```

## Performance Optimization

### Slow Deployments

- Enable shared resources for large static files
- Use `hashed_asset_patterns` for specific files
- Consider excluding large non-changing files from patterns

### High Disk Usage

- Reduce `keep_releases` count
- Enable shared resources
- Use `delete_old: true` for single release deployments

### Network Issues

- Compress large files before deployment
- Use faster SSH ciphers if supported
- Consider deploying during off-peak hours