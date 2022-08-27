use actix_web::middleware as actix_middleware;

pub fn config() -> actix_middleware::DefaultHeaders {
    actix_middleware::DefaultHeaders::new()
            .add(("Access-Control-Allow-Origin", "*"))
            .add(("Content-Security-Policy", "default-src 'self';base-uri 'self';block-all-mixed-content;font-src 'self' https: data:;frame-ancestors 'self';img-src 'self' data:;object-src 'none';script-src 'self';script-src-attr 'none';style-src 'self' https: 'unsafe-inline';upgrade-insecure-requests"))
            .add(("X-DNS-Prefetch-Control", "	off"))
            .add(("Expect-CT", "max-age=0"))
            .add(("X-Frame-Options", "SAMEORIGIN"))
            .add(("Strict-Transport-Security", "max-age=15552000; includeSubDomains"))
            .add(("X-Download-Options", "noopen"))
            .add(("X-Content-Type-Options", "nosniff"))
            .add(("X-Permitted-Cross-Domain-Policies", "none"))
            .add(("Referrer-Policy", "no-referrer"))
            .add(("X-XSS-Protection","0"))
            .add(("Vary", "Accept-Encoding"))
}
