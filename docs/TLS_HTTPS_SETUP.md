# TLS/HTTPS Setup Guide

This guide provides comprehensive instructions for securing your SagensContact deployment with TLS/HTTPS encryption.

## Table of Contents

1. [Overview](#overview)
2. [Development Setup (Self-Signed Certificates)](#development-setup-self-signed-certificates)
3. [Production Setup (Real Certificates)](#production-setup-real-certificates)
4. [Reverse Proxy Configuration](#reverse-proxy-configuration)
5. [Certificate Management](#certificate-management)
6. [Security Best Practices](#security-best-practices)
7. [Troubleshooting](#troubleshooting)

---

## Overview

SagensContact supports three TLS/HTTPS deployment approaches:

1. **Self-Signed Certificates**: For development/testing environments
2. **Let's Encrypt**: Free automated certificates for production
3. **Commercial CA Certificates**: Enterprise-grade certificates from commercial providers

### Architecture Options

```
┌─────────────────────────────────────────────────────────────┐
│ Option 1: Direct TLS (Not Recommended for Production)      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Internet → SagensContact Sync Service (TLS termination)   │
│                                                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Option 2: Reverse Proxy (Recommended)                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Internet → Nginx/Caddy (TLS) → Sync Service (HTTP)        │
│                                                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Option 3: Load Balancer (Enterprise)                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Internet → AWS ELB/CloudFlare (TLS) → Sync Service (HTTP) │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Recommendation**: Use Option 2 (Reverse Proxy) for production deployments. This provides:
- Centralized certificate management
- Better performance through connection pooling
- Load balancing capabilities
- Static file serving
- Request/response caching
- DDoS protection

---

## Development Setup (Self-Signed Certificates)

### Generate Self-Signed Certificate

```bash
# Create certificates directory
mkdir -p certs
cd certs

# Generate private key and certificate (valid for 365 days)
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout key.pem \
  -out cert.pem \
  -days 365 \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"

# Verify certificate
openssl x509 -in cert.pem -text -noout
```

### Configure Rust Application (If Using Direct TLS)

**Note**: Direct TLS in sync_service is not currently implemented. Use reverse proxy instead.

If you need direct TLS support:

1. Add dependencies to `crates/sync_service/Cargo.toml`:
```toml
[dependencies]
axum-server = { version = "0.5", features = ["tls-rustls"] }
```

2. Update `main.rs`:
```rust
use axum_server::tls_rustls::RustlsConfig;

#[tokio::main]
async fn main() {
    let config = RustlsConfig::from_pem_file("certs/cert.pem", "certs/key.pem")
        .await
        .expect("Failed to load TLS certificates");

    let addr = "0.0.0.0:3443".parse().unwrap();

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### Trust Self-Signed Certificate (Client Side)

**macOS**:
```bash
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain cert.pem
```

**Linux (Ubuntu/Debian)**:
```bash
sudo cp cert.pem /usr/local/share/ca-certificates/sagenscontact.crt
sudo update-ca-certificates
```

**Windows**:
```powershell
Import-Certificate -FilePath cert.pem -CertStoreLocation Cert:\LocalMachine\Root
```

---

## Production Setup (Real Certificates)

### Option A: Let's Encrypt with Certbot

**Prerequisites**:
- Domain name pointing to your server
- Port 80 and 443 accessible
- Email address for certificate notifications

**Installation**:
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install certbot

# CentOS/RHEL
sudo yum install certbot

# macOS
brew install certbot
```

**Obtain Certificate**:
```bash
# Standalone mode (stops temporarily on port 80)
sudo certbot certonly --standalone \
  -d sagenscontact.example.com \
  --email admin@example.com \
  --agree-tos

# Webroot mode (if web server already running)
sudo certbot certonly --webroot \
  -w /var/www/html \
  -d sagenscontact.example.com \
  --email admin@example.com \
  --agree-tos
```

**Certificate Location**:
```
Certificate: /etc/letsencrypt/live/sagenscontact.example.com/fullchain.pem
Private Key: /etc/letsencrypt/live/sagenscontact.example.com/privkey.pem
```

**Auto-Renewal Setup**:
```bash
# Test renewal
sudo certbot renew --dry-run

# Cron job (already configured by certbot)
# Runs twice daily, renews if certificate expires within 30 days
# Check: sudo systemctl status certbot.timer
```

### Option B: Commercial CA Certificates

**Generate Certificate Signing Request (CSR)**:
```bash
# Generate private key
openssl genrsa -out private.key 4096

# Generate CSR
openssl req -new -key private.key -out request.csr \
  -subj "/C=US/ST=State/L=City/O=Company/CN=sagenscontact.example.com"

# Submit request.csr to your CA (DigiCert, GoDaddy, etc.)
```

**Install Received Certificate**:
```bash
# Your CA will provide:
# - certificate.crt (your certificate)
# - intermediate.crt (intermediate certificate)
# - root.crt (root certificate)

# Create certificate chain
cat certificate.crt intermediate.crt root.crt > fullchain.pem
cp private.key privkey.pem

# Set permissions
chmod 600 privkey.pem
chmod 644 fullchain.pem
```

---

## Reverse Proxy Configuration

### Nginx (Recommended)

**Installation**:
```bash
# Ubuntu/Debian
sudo apt install nginx

# CentOS/RHEL
sudo yum install nginx

# macOS
brew install nginx
```

**Configuration** (`/etc/nginx/sites-available/sagenscontact`):
```nginx
# Redirect HTTP to HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name sagenscontact.example.com;

    # Let's Encrypt ACME challenge
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    # Redirect all other traffic to HTTPS
    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTPS server
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name sagenscontact.example.com;

    # TLS Configuration
    ssl_certificate /etc/letsencrypt/live/sagenscontact.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/sagenscontact.example.com/privkey.pem;

    # Modern TLS configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384';
    ssl_prefer_server_ciphers off;

    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/sagenscontact.example.com/chain.pem;

    # Security Headers (additional layer on top of app headers)
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;

    # Logging
    access_log /var/log/nginx/sagenscontact_access.log;
    error_log /var/log/nginx/sagenscontact_error.log;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;

    # Proxy settings
    client_max_body_size 100M;

    # API proxy (Sync Service)
    location /api/ {
        proxy_pass http://127.0.0.1:3002;
        proxy_http_version 1.1;

        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # WebSocket support
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # Health check
    location /health {
        proxy_pass http://127.0.0.1:3002/health;
        access_log off;
    }

    # Metrics (restrict to internal network)
    location /metrics {
        proxy_pass http://127.0.0.1:3002/metrics;
        allow 10.0.0.0/8;
        allow 172.16.0.0/12;
        allow 192.168.0.0/16;
        deny all;
    }

    # Web UI (if served separately)
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Static files (if serving directly)
    location /static/ {
        alias /var/www/sagenscontact/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

**Enable Configuration**:
```bash
# Test configuration
sudo nginx -t

# Enable site
sudo ln -s /etc/nginx/sites-available/sagenscontact /etc/nginx/sites-enabled/

# Reload Nginx
sudo systemctl reload nginx

# Enable on boot
sudo systemctl enable nginx
```

### Caddy (Automatic HTTPS)

**Installation**:
```bash
# Ubuntu/Debian
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/caddy-stable-archive-keyring.gpg] https://dl.cloudsmith.io/public/caddy/stable/deb/debian any-version main" | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy

# macOS
brew install caddy
```

**Configuration** (`/etc/caddy/Caddyfile`):
```caddy
sagenscontact.example.com {
    # Automatic HTTPS via Let's Encrypt
    # Caddy handles certificate issuance and renewal automatically

    # Email for Let's Encrypt notifications
    tls admin@example.com

    # Rate limiting
    rate_limit {
        zone dynamic {
            key {remote_host}
            events 100
            window 1m
        }
    }

    # Logging
    log {
        output file /var/log/caddy/sagenscontact.log
        format json
    }

    # API proxy
    handle /api/* {
        reverse_proxy localhost:3002 {
            header_up X-Real-IP {remote_host}
            header_up X-Forwarded-For {remote_host}
            header_up X-Forwarded-Proto {scheme}
        }
    }

    # Health check
    handle /health {
        reverse_proxy localhost:3002
    }

    # Metrics (restrict to internal)
    @internal {
        remote_ip 10.0.0.0/8 172.16.0.0/12 192.168.0.0/16
    }
    handle @internal /metrics {
        reverse_proxy localhost:3002
    }

    # Web UI
    handle {
        reverse_proxy localhost:3001
    }
}
```

**Start Caddy**:
```bash
sudo systemctl start caddy
sudo systemctl enable caddy

# Reload after config changes
sudo systemctl reload caddy
```

**Advantages of Caddy**:
- Automatic HTTPS with zero configuration
- Automatic certificate renewal
- Modern defaults (HTTP/2, TLS 1.3)
- Simpler configuration syntax

---

## Certificate Management

### Certificate Renewal Automation

**Let's Encrypt (Certbot)**:
```bash
# Certbot auto-renews via systemd timer
sudo systemctl status certbot.timer

# Manual renewal
sudo certbot renew

# Reload services after renewal
sudo certbot renew --deploy-hook "systemctl reload nginx"
```

**Caddy**:
```bash
# Caddy handles renewal automatically
# No manual intervention required
# Check logs for renewal activity
sudo journalctl -u caddy -n 100
```

### Certificate Monitoring

**Create monitoring script** (`/usr/local/bin/check-cert-expiry.sh`):
```bash
#!/bin/bash

DOMAIN="sagenscontact.example.com"
CERT_FILE="/etc/letsencrypt/live/${DOMAIN}/fullchain.pem"
ALERT_DAYS=30

if [ ! -f "$CERT_FILE" ]; then
    echo "Certificate file not found: $CERT_FILE"
    exit 1
fi

EXPIRY_DATE=$(openssl x509 -in "$CERT_FILE" -noout -enddate | cut -d= -f2)
EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s)
NOW_EPOCH=$(date +%s)
DAYS_LEFT=$(( ($EXPIRY_EPOCH - $NOW_EPOCH) / 86400 ))

echo "Certificate expires in $DAYS_LEFT days"

if [ $DAYS_LEFT -lt $ALERT_DAYS ]; then
    echo "WARNING: Certificate expires soon!" >&2
    # Send alert (email, Slack, etc.)
    # mail -s "Certificate Expiry Warning" admin@example.com <<< "Certificate expires in $DAYS_LEFT days"
    exit 1
fi

exit 0
```

**Schedule monitoring**:
```bash
# Add to crontab
sudo crontab -e

# Check daily at 3 AM
0 3 * * * /usr/local/bin/check-cert-expiry.sh >> /var/log/cert-check.log 2>&1
```

### Certificate Backup

```bash
#!/bin/bash
# Backup certificates daily

BACKUP_DIR="/backup/certificates"
DATE=$(date +%Y%m%d)

mkdir -p "$BACKUP_DIR"

# Backup Let's Encrypt certificates
sudo tar czf "$BACKUP_DIR/letsencrypt-$DATE.tar.gz" \
    /etc/letsencrypt/

# Keep only last 30 days
find "$BACKUP_DIR" -name "letsencrypt-*.tar.gz" -mtime +30 -delete
```

---

## Security Best Practices

### TLS Configuration

1. **Use TLS 1.2 and 1.3 only**
   - Disable TLS 1.0 and 1.1 (deprecated)
   - Disable SSL v2 and v3 (vulnerable)

2. **Strong Cipher Suites**
   ```
   ECDHE-ECDSA-AES128-GCM-SHA256
   ECDHE-RSA-AES128-GCM-SHA256
   ECDHE-ECDSA-AES256-GCM-SHA384
   ECDHE-RSA-AES256-GCM-SHA384
   ```

3. **Enable HSTS**
   ```
   Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
   ```

4. **OCSP Stapling**
   - Improves performance
   - Enhances privacy
   - Provides real-time revocation status

### Testing TLS Configuration

**SSL Labs Test**:
```
https://www.ssllabs.com/ssltest/analyze.html?d=sagenscontact.example.com
```

**Command Line Testing**:
```bash
# Test TLS connection
openssl s_client -connect sagenscontact.example.com:443 -servername sagenscontact.example.com

# Check certificate details
echo | openssl s_client -connect sagenscontact.example.com:443 -servername sagenscontact.example.com 2>/dev/null | openssl x509 -noout -dates

# Test specific TLS version
openssl s_client -connect sagenscontact.example.com:443 -tls1_2
openssl s_client -connect sagenscontact.example.com:443 -tls1_3

# Test cipher suites
nmap --script ssl-enum-ciphers -p 443 sagenscontact.example.com
```

### Certificate Pinning (Optional)

For mobile/desktop clients, consider certificate pinning:

```rust
// Example for Rust client
use reqwest::Certificate;

let cert_pem = include_str!("sagenscontact.pem");
let cert = Certificate::from_pem(cert_pem.as_bytes())?;

let client = reqwest::Client::builder()
    .add_root_certificate(cert)
    .build()?;
```

---

## Troubleshooting

### Common Issues

#### "Certificate Expired"

**Symptoms**: Browser shows "Your connection is not private" or "NET::ERR_CERT_DATE_INVALID"

**Solutions**:
```bash
# Check certificate expiry
openssl x509 -in /etc/letsencrypt/live/DOMAIN/fullchain.pem -noout -dates

# Renew certificate
sudo certbot renew --force-renewal

# Reload web server
sudo systemctl reload nginx
```

#### "Wrong Host in Certificate"

**Symptoms**: Certificate shows different domain than requested

**Solutions**:
- Ensure server_name in Nginx matches certificate CN/SAN
- Check DNS records point to correct server
- Regenerate certificate with correct domain:
  ```bash
  sudo certbot certonly --standalone -d correct-domain.com
  ```

#### "Mixed Content" Warnings

**Symptoms**: Page loads over HTTPS but some resources load over HTTP

**Solutions**:
- Update API client to use relative URLs: `/api/contacts` instead of `http://...`
- Enable HSTS to force HTTPS
- Check CSP headers allow current resources

#### Nginx "Unable to Load Certificate"

**Symptoms**: Nginx fails to start with certificate error

**Solutions**:
```bash
# Check file permissions
ls -l /etc/letsencrypt/live/DOMAIN/

# Should be readable by nginx user
sudo chmod 755 /etc/letsencrypt/live/
sudo chmod 755 /etc/letsencrypt/archive/

# Test Nginx config
sudo nginx -t

# Check certificate validity
openssl verify -CAfile /etc/letsencrypt/live/DOMAIN/chain.pem \
    /etc/letsencrypt/live/DOMAIN/cert.pem
```

#### Rate Limit Errors from Let's Encrypt

**Symptoms**: "too many certificates already issued"

**Solutions**:
- Let's Encrypt limits: 50 certificates per week per domain
- Use staging environment for testing:
  ```bash
  sudo certbot certonly --dry-run --staging -d domain.com
  ```
- Wait for rate limit window to reset (1 week)
- Use wildcard certificates to cover subdomains

### Debug Mode

**Enable verbose logging**:

Nginx:
```nginx
error_log /var/log/nginx/error.log debug;
```

Certbot:
```bash
sudo certbot renew --verbose
```

Caddy:
```json
{
    "logging": {
        "logs": {
            "default": {
                "level": "DEBUG"
            }
        }
    }
}
```

---

## Production Deployment Checklist

- [ ] Domain name configured with DNS A/AAAA records
- [ ] Firewall allows inbound traffic on ports 80 and 443
- [ ] Certificate obtained (Let's Encrypt or commercial)
- [ ] Reverse proxy installed and configured
- [ ] TLS 1.2/1.3 enabled, older versions disabled
- [ ] Strong cipher suites configured
- [ ] HSTS header enabled with appropriate max-age
- [ ] Certificate auto-renewal configured
- [ ] Certificate expiry monitoring configured
- [ ] Certificate backup strategy implemented
- [ ] SSL Labs test shows A+ rating
- [ ] Application configured to trust X-Forwarded-Proto header
- [ ] Rate limiting configured at proxy level
- [ ] Logging configured for security monitoring
- [ ] Health check endpoint accessible
- [ ] Metrics endpoint restricted to internal network
- [ ] Load testing performed over HTTPS
- [ ] Documentation updated with production URLs

---

## Additional Resources

- **Mozilla SSL Configuration Generator**: https://ssl-config.mozilla.org/
- **Let's Encrypt Documentation**: https://letsencrypt.org/docs/
- **Nginx TLS Best Practices**: https://wiki.mozilla.org/Security/Server_Side_TLS
- **Caddy Documentation**: https://caddyserver.com/docs/
- **OWASP Transport Layer Protection**: https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Protection_Cheat_Sheet.html

---

## Support

For issues specific to SagensContact TLS setup:
- GitHub Issues: https://github.com/yourusername/sagenscontact/issues
- Documentation: See `docs/` directory for additional guides
- Security Issues: Email security@example.com (do not file public issues)
