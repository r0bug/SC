use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderName, HeaderValue, Response},
    middleware::Next,
};

/// Security headers middleware
/// Adds comprehensive security headers to all responses
pub async fn security_headers_middleware(
    req: Request,
    next: Next,
) -> Response<Body> {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // HSTS - require HTTPS for 1 year
    // Force all connections over HTTPS for 1 year, including subdomains
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Content Security Policy
    // Restrict resource loading to prevent XSS attacks
    // - default-src 'self': Only allow resources from same origin by default
    // - script-src 'self' 'unsafe-inline': Allow scripts from same origin and inline scripts
    // - style-src 'self' 'unsafe-inline': Allow styles from same origin and inline styles
    // - img-src 'self' data: blob:: Allow images from same origin, data URIs, and blobs
    // - font-src 'self': Only allow fonts from same origin
    // - connect-src 'self': Only allow AJAX/WebSocket connections to same origin
    // - frame-ancestors 'none': Prevent embedding in frames (defense in depth with X-Frame-Options)
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: blob:; \
             font-src 'self'; \
             connect-src 'self'; \
             frame-ancestors 'none'"
        ),
    );

    // X-Frame-Options
    // Prevent clickjacking by disallowing the page to be embedded in frames
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // X-Content-Type-Options
    // Prevent MIME type sniffing which could lead to XSS attacks
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // X-XSS-Protection
    // Enable browser's XSS filter (legacy header but still useful for older browsers)
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy
    // Control how much referrer information is sent with requests
    // strict-origin-when-cross-origin: Send full URL for same-origin, only origin for cross-origin
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy (formerly Feature-Policy)
    // Disable potentially dangerous browser features
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "geolocation=(), \
             microphone=(), \
             camera=(), \
             payment=(), \
             usb=(), \
             magnetometer=(), \
             gyroscope=(), \
             accelerometer=()"
        ),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::util::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_security_headers_are_added() {
        let app = Router::new()
            .route("/", get(test_handler))
            .layer(axum::middleware::from_fn(security_headers_middleware));

        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();

        // Check HSTS header
        assert!(headers.contains_key(header::STRICT_TRANSPORT_SECURITY));
        assert_eq!(
            headers.get(header::STRICT_TRANSPORT_SECURITY).unwrap(),
            "max-age=31536000; includeSubDomains"
        );

        // Check CSP header
        assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));

        // Check X-Frame-Options header
        assert!(headers.contains_key(header::X_FRAME_OPTIONS));
        assert_eq!(
            headers.get(header::X_FRAME_OPTIONS).unwrap(),
            "DENY"
        );

        // Check X-Content-Type-Options header
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert_eq!(
            headers.get("X-Content-Type-Options").unwrap(),
            "nosniff"
        );

        // Check X-XSS-Protection header
        assert!(headers.contains_key("X-XSS-Protection"));
        assert_eq!(
            headers.get("X-XSS-Protection").unwrap(),
            "1; mode=block"
        );

        // Check Referrer-Policy header
        assert!(headers.contains_key(header::REFERRER_POLICY));

        // Check Permissions-Policy header
        assert!(headers.contains_key("Permissions-Policy"));
    }
}
