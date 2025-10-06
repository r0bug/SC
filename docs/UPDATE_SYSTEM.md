# Update System

SagensContact Alpha includes an automatic update system that allows remote installations to fetch and apply updates from GitHub releases.

## Architecture

The update system consists of:

1. **Update Checker** (`update_system.rs`) - Core logic for version comparison and GitHub API integration
2. **Update Routes** (`update_routes.rs`) - REST API endpoints for update operations
3. **GitHub Releases** - Source of truth for available versions
4. **Build Metadata** - Version tracking via `build.rs`

## Features

- ✅ Automatic version checking via GitHub releases API
- ✅ Semantic version comparison
- ✅ Platform-specific binary downloads
- ✅ SHA256 checksum verification
- ✅ Automatic backup before update
- ✅ Rollback capability
- ✅ Configurable auto-update behavior

## API Endpoints

### Check Current Version
```bash
GET /api/system/version
```

Response:
```json
{
  "version": "0.1.0",
  "build_date": "2025-10-06 10:30:00 UTC",
  "commit_hash": "abc1234"
}
```

### Check for Updates
```bash
GET /api/system/updates/check
```

Response:
```json
{
  "current_version": "0.1.0",
  "latest_version": "0.2.0",
  "update_available": true,
  "release_url": "https://github.com/sagenscontact/alpha/releases/tag/v0.2.0",
  "release_notes": "## What's New\n- Feature 1\n- Bug fix 2",
  "download_url": "https://github.com/sagenscontact/alpha/releases/download/v0.2.0/sagenscontact-ubuntu-latest",
  "published_at": "2025-10-06T10:00:00Z"
}
```

### Get Cached Update Info
```bash
GET /api/system/updates/info
```

Returns the last checked update information without making a new GitHub API call.

### Get Update Configuration
```bash
GET /api/system/updates/config
```

Response:
```json
{
  "auto_check": true,
  "auto_download": false,
  "auto_install": false,
  "check_interval_hours": 24
}
```

### Update Configuration
```bash
PUT /api/system/updates/config
Content-Type: application/json

{
  "auto_check": true,
  "auto_download": true,
  "check_interval_hours": 12
}
```

### Download Update
```bash
POST /api/system/updates/download
```

Downloads the latest available update to a temporary location.

### Apply Update
```bash
POST /api/system/updates/apply
```

⚠️ **This endpoint requires the service to restart.**

## Configuration

Update behavior can be configured via the `/api/system/updates/config` endpoint:

- **auto_check**: Automatically check for updates (default: true)
- **auto_download**: Automatically download updates when available (default: false)
- **auto_install**: Automatically install downloaded updates (default: false)
- **check_interval_hours**: How often to check for updates (1-168 hours, default: 24)

## Release Process

### Creating a Release

1. **Update Version** in `Cargo.toml`:
```toml
[workspace.package]
version = "0.2.0"
```

2. **Commit Changes**:
```bash
git add .
git commit -m "chore: bump version to 0.2.0"
git push
```

3. **Create Git Tag**:
```bash
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

4. **GitHub Actions** will automatically:
   - Build release binaries for all platforms
   - Create GitHub release
   - Upload artifacts

5. **Verify Release**:
   - Check https://github.com/sagenscontact/alpha/releases
   - Ensure all platform binaries are attached

### Manual Release (if needed)

```bash
# Build release binaries
cargo build --release --bin sync_service
cargo build --release --bin sagenscontact

# Create release on GitHub
gh release create v0.2.0 \
  --title "v0.2.0" \
  --notes "Release notes here" \
  target/release/sync_service \
  target/release/sagenscontact
```

## Update Flow

### Automatic Update (when enabled)

1. Service checks GitHub API every `check_interval_hours`
2. If new version found and `auto_download` enabled:
   - Downloads binary to temp directory
   - Verifies checksum
3. If `auto_install` enabled:
   - Creates backup of current binary
   - Replaces current binary with update
   - Schedules restart

### Manual Update

1. User checks for updates via API or UI
2. User initiates download
3. User confirms installation
4. Service applies update and restarts

## Security

- **HTTPS Only**: All downloads use HTTPS
- **Checksum Verification**: SHA256 checksums verify file integrity
- **Backup Creation**: Previous version kept as `.bak` file
- **Rollback Capability**: Can revert to previous version

## Troubleshooting

### Update Check Fails

```bash
# Check network connectivity
curl -I https://api.github.com

# Check GitHub API rate limits
curl https://api.github.com/rate_limit
```

### Update Download Fails

- Ensure sufficient disk space in temp directory
- Check firewall settings allow HTTPS to GitHub
- Verify GitHub release has platform-specific artifacts

### Update Won't Apply

- Check file permissions on binary
- Ensure no other processes are using the binary
- Review logs: `/var/log/sagenscontact/sync_service.log`

### Rollback

If an update causes issues:

```bash
# Via API
POST /api/system/updates/rollback

# Manual
mv sync_service.bak sync_service
chmod +x sync_service
systemctl restart sagenscontact
```

## Testing

### Test Update Check

```bash
# Check current version
curl http://localhost:3002/api/system/version

# Check for updates
curl http://localhost:3002/api/system/updates/check | jq
```

### Test with Mock Release

For testing, you can point to a test repository:

1. Edit `GITHUB_REPO` in `update_system.rs`
2. Create test releases in your fork
3. Test update flow

## Environment Variables

```bash
# Override GitHub repository
SAGENSCONTACT_UPDATE_REPO="your-org/your-repo"

# Disable update checks
SAGENSCONTACT_DISABLE_UPDATES="true"
```

## Future Enhancements

- [ ] Delta updates (only download changes)
- [ ] Signature verification (GPG)
- [ ] Update scheduling (maintenance windows)
- [ ] Staged rollouts (canary deployments)
- [ ] Update notifications via email/webhook
- [ ] Bandwidth throttling for downloads
- [ ] Resume interrupted downloads
- [ ] Pre-update health checks
- [ ] Post-update validation

## Related Files

- `crates/sync_service/src/update_system.rs` - Core update logic
- `crates/sync_service/src/update_routes.rs` - API endpoints
- `crates/sync_service/build.rs` - Build metadata injection
- `.github/workflows/ci.yml` - Release automation
