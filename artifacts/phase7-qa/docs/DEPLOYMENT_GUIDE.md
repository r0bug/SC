# SagensContact Phase 7 - Deployment Guide

**Version:** 0.1.0-alpha
**Date:** October 2, 2025
**Status:** QA Ready

---

## Quick Reference

### Service URLs
- **Web UI**: http://localhost:3001
- **API**: http://localhost:3002/api
- **Health Check**: http://localhost:3002/health
- **Detailed Health**: http://localhost:3002/api/health/detailed
- **Metrics**: http://localhost:3002/metrics (restrict in production!)

### Default Ports
- Sync Service: 3002
- Web UI: 3001
- PostgreSQL (if used): 5432

---

## Local Development Setup

### 1. Prerequisites
```bash
# Verify binary permissions
chmod +x binaries/*
chmod +x scripts/*.sh

# Create data directories
mkdir -p data/attachments
mkdir -p data/logs
```

### 2. Start Services
```bash
# Terminal 1 - Start Sync Service
cd scripts
./start_sync_service.sh

# Terminal 2 - Start Web UI
cd scripts
./start_web_ui.sh

# Verify services
curl http://localhost:3002/health    # Should return "OK"
curl http://localhost:3001            # Should return web UI HTML
```

### 3. Environment Variables
```bash
# Sync Service (.env.sync)
export DATABASE_URL="sqlite:./data/contacts.db"
export PORT=3002
export JWT_SECRET="dev-secret-change-in-production"
export LOG_FORMAT="pretty"  # or "json"
export ATTACHMENT_STORAGE_PATH="./data/attachments"

# Web UI (.env.web)
export PORT=3001
export API_URL="http://localhost:3002"
```

---

## Staging Deployment

### Option A: Docker Compose (Recommended)

**1. Create `docker-compose.yml`:**
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: sagenscontact
      POSTGRES_USER: sagenscontact
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  sync_service:
    image: sagenscontact-sync:latest
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgresql://sagenscontact:${DB_PASSWORD}@postgres:5432/sagenscontact
      PORT: 3002
      JWT_SECRET: ${JWT_SECRET}
      LOG_FORMAT: json
    volumes:
      - ./data/attachments:/app/data/attachments
    ports:
      - "3002:3002"

  web:
    image: sagenscontact-web:latest
    depends_on:
      - sync_service
    environment:
      PORT: 3001
      API_URL: http://sync_service:3002
    ports:
      - "3001:3001"

  nginx:
    image: nginx:alpine
    depends_on:
      - web
      - sync_service
    volumes:
      - ../config/nginx.conf:/etc/nginx/nginx.conf:ro
      - /etc/letsencrypt:/etc/letsencrypt:ro
    ports:
      - "80:80"
      - "443:443"

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"

volumes:
  postgres_data:
  prometheus_data:
```

**2. Create `.env` file:**
```bash
DB_PASSWORD=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 32)
```

**3. Start services:**
```bash
docker-compose up -d
docker-compose logs -f  # View logs
```

### Option B: Systemd Services

**1. Install binaries:**
```bash
sudo mkdir -p /opt/sagenscontact/{binaries,web,data/attachments}
sudo cp -r binaries/* /opt/sagenscontact/binaries/
sudo cp -r web/* /opt/sagenscontact/web/
sudo chown -R sagenscontact:sagenscontact /opt/sagenscontact
```

**2. Install systemd services:**
```bash
sudo cp config/systemd/*.service /etc/systemd/system/
sudo systemctl daemon-reload
```

**3. Configure environment:**
```bash
sudo mkdir -p /etc/sagenscontact
sudo tee /etc/sagenscontact/sync_service.env << EOF
DATABASE_URL=postgresql://sagenscontact:PASSWORD@localhost:5432/sagenscontact
PORT=3002
JWT_SECRET=$(openssl rand -base64 32)
LOG_FORMAT=json
ATTACHMENT_STORAGE_PATH=/opt/sagenscontact/data/attachments
EOF

sudo chmod 600 /etc/sagenscontact/sync_service.env
```

**4. Enable and start:**
```bash
sudo systemctl enable --now sagenscontact-sync.service
sudo systemctl enable --now sagenscontact-web.service
sudo systemctl status sagenscontact-sync.service
```

---

## TLS/HTTPS Setup

### Option A: Nginx with Let's Encrypt

**1. Install Certbot:**
```bash
sudo apt install certbot python3-certbot-nginx
```

**2. Obtain certificate:**
```bash
sudo certbot --nginx -d contacts.example.com
```

**3. Configure Nginx:**
Use the provided `config/nginx.conf.example` as a template.

```bash
sudo cp config/nginx.conf.example /etc/nginx/sites-available/sagenscontact
# Edit with your domain
sudo ln -s /etc/nginx/sites-available/sagenscontact /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Option B: Caddy (Simpler)

**1. Install Caddy:**
```bash
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/caddy-stable-archive-keyring.gpg] https://dl.cloudsmith.io/public/caddy/stable/deb/debian any-version main" | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

**2. Create Caddyfile:**
```bash
sudo tee /etc/caddy/Caddyfile << 'EOF'
contacts.example.com {
    # Web UI
    reverse_proxy localhost:3001

    # API
    handle /api/* {
        reverse_proxy localhost:3002
    }

    # WebSocket
    handle /ws/* {
        reverse_proxy localhost:3002
    }

    # Metrics (restrict!)
    handle /metrics {
        @allowed remote_ip 127.0.0.1
        reverse_proxy @allowed localhost:3002
        respond 403
    }
}
EOF

sudo systemctl reload caddy
```

---

## Monitoring Setup

### Prometheus Configuration

**Create `prometheus.yml`:**
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'sagenscontact-sync'
    static_configs:
      - targets: ['localhost:3002']
    metrics_path: '/metrics'
```

### Grafana Dashboard

**Import dashboard JSON** (create custom or use Prometheus defaults):
- HTTP request rate and latency
- Database query performance
- Rate limit hits
- WebSocket connections
- Attachment upload metrics

---

## Security Checklist

### Pre-Production
- [ ] Set strong `JWT_SECRET` (32+ characters random)
- [ ] Enable TLS/HTTPS (Let's Encrypt or purchased cert)
- [ ] Restrict `/metrics` endpoint to localhost/monitoring IPs
- [ ] Configure firewall (ufw/iptables)
- [ ] Set `LOG_FORMAT=json` for structured logging
- [ ] Enable database backups (automated)
- [ ] Review rate limit configurations
- [ ] Test security headers with https://securityheaders.com

### Production Hardening
- [ ] Use PostgreSQL instead of SQLite (for scale)
- [ ] Deploy Redis for distributed rate limiting
- [ ] Configure log aggregation (ELK/Loki)
- [ ] Set up alerting (Prometheus Alertmanager)
- [ ] Implement database connection pooling
- [ ] Enable WAL mode for SQLite (if still using)
- [ ] Configure automated certificate renewal
- [ ] Set up backup restoration testing
- [ ] Implement secrets management (Vault/AWS Secrets Manager)
- [ ] Configure CORS properly for web UI domain

---

## Database Migration

### From SQLite to PostgreSQL

**1. Export SQLite data:**
```bash
sqlite3 data/contacts.db .dump > contacts.sql
```

**2. Create PostgreSQL database:**
```bash
createdb sagenscontact
psql sagenscontact < contacts.sql
```

**3. Update DATABASE_URL:**
```bash
export DATABASE_URL="postgresql://user:pass@localhost/sagenscontact"
```

---

## Benchmarking

### Run benchmark suite:
```bash
cd scripts
./benchmark.sh

# Results will be in:
# - benchmark_results.txt
# - latency_p50, p95, p99
```

### Expected Performance (Alpha baseline):
- **Health endpoint**: < 5ms p95
- **List contacts**: < 50ms p95
- **Search**: < 100ms p95
- **Create contact**: < 20ms p95

---

## Troubleshooting

### Service won't start
```bash
# Check logs
journalctl -u sagenscontact-sync.service -f

# Common issues:
# 1. DATABASE_URL incorrect
# 2. Port already in use (check with: lsof -i :3002)
# 3. Missing data directories
# 4. Permission denied on binaries
```

### Database connection errors
```bash
# Test connection
psql "$DATABASE_URL"

# Check PostgreSQL is running
sudo systemctl status postgresql

# Verify credentials
echo "$DATABASE_URL"
```

### Rate limiting too aggressive
```bash
# Edit rate_limit.rs configurations:
# - Auth: requests_per_second, burst_size
# - Attachments: requests_per_second, burst_size
# - Search: requests_per_second, burst_size

# Rebuild and redeploy
```

### Web UI can't connect to API
```bash
# Verify API is running
curl http://localhost:3002/health

# Check CORS configuration
# Check API_URL environment variable in web UI
# Verify reverse proxy configuration
```

---

## Backup & Recovery

### Automated Backup Script
```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/opt/sagenscontact/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Backup SQLite
sqlite3 /opt/sagenscontact/data/contacts.db ".backup '$BACKUP_DIR/contacts_$DATE.db'"

# Backup attachments
tar -czf "$BACKUP_DIR/attachments_$DATE.tar.gz" /opt/sagenscontact/data/attachments/

# Cleanup old backups (keep 30 days)
find $BACKUP_DIR -mtime +30 -delete

# Upload to S3 (optional)
# aws s3 cp "$BACKUP_DIR/contacts_$DATE.db" s3://backups/sagenscontact/
```

Add to crontab:
```bash
0 2 * * * /opt/sagenscontact/scripts/backup.sh
```

---

## Rollback Procedure

### If deployment fails:

**1. Stop new services:**
```bash
sudo systemctl stop sagenscontact-sync.service
sudo systemctl stop sagenscontact-web.service
```

**2. Restore previous version:**
```bash
sudo cp -r /opt/sagenscontact.backup/* /opt/sagenscontact/
```

**3. Restore database:**
```bash
sqlite3 /opt/sagenscontact/data/contacts.db ".restore '/opt/sagenscontact/backups/contacts_YYYYMMDD.db'"
```

**4. Restart services:**
```bash
sudo systemctl start sagenscontact-sync.service
sudo systemctl start sagenscontact-web.service
```

---

## Next Steps After QA

### For Beta Release:
1. Implement Redis-backed rate limiting
2. Add OAuth2/OIDC authentication
3. Configure real email/SMS providers
4. Set up CDN for static assets
5. Implement advanced search (Elasticsearch/Meilisearch)
6. Add WebSocket real-time features
7. Performance optimizations based on benchmark results
8. Security audit by specialist
9. Load testing with realistic scenarios
10. Penetration testing

---

## Support

For deployment issues or questions:
- Check logs: `journalctl -u sagenscontact-sync.service -f`
- Review docs: `docs/TLS_HTTPS_SETUP.md`
- Contact: SagensContact Development Team
