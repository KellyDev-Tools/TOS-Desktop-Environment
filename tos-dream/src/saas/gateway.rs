//! Phase 16 Week 2: API Gateway
//!
//! Request routing, rate limiting, authentication, and load balancing
//! for multi-tenant SaaS deployments.

use super::{TenantId, SessionId, SaasResult, RateLimitConfig};
use std::collections::HashMap;
use std::net::IpAddr;
use serde::{Deserialize, Serialize};
use chrono;

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Connect,
    Trace,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Connect => "CONNECT",
            Self::Trace => "TRACE",
        }
    }
}

/// Route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Route ID
    pub id: String,
    /// Path pattern (e.g., "/api/v1/sectors/*")
    pub path: String,
    /// HTTP methods allowed
    pub methods: Vec<HttpMethod>,
    /// Target service URL
    pub target: String,
    /// Strip path prefix
    pub strip_prefix: bool,
    /// Add headers
    pub add_headers: HashMap<String, String>,
    /// Remove headers
    pub remove_headers: Vec<String>,
    /// Require authentication
    pub require_auth: bool,
    /// Rate limit configuration
    pub rate_limit: Option<RateLimitConfig>,
    /// Circuit breaker enabled
    pub circuit_breaker: bool,
    /// Timeout in seconds
    pub timeout_seconds: u64,
}

impl Route {
    /// Create a new route
    pub fn new(
        id: impl Into<String>,
        path: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            methods: vec![HttpMethod::Get, HttpMethod::Post, HttpMethod::Put, HttpMethod::Delete],
            target: target.into(),
            strip_prefix: true,
            add_headers: HashMap::new(),
            remove_headers: Vec::new(),
            require_auth: true,
            rate_limit: None,
            circuit_breaker: true,
            timeout_seconds: 30,
        }
    }
    
    /// Match path against route pattern
    pub fn matches(&self, path: &str, method: HttpMethod) -> bool {
        // Check method
        if !self.methods.contains(&method) {
            return false;
        }
        
        // Simple wildcard matching
        if self.path.ends_with("/*") {
            let prefix = &self.path[..self.path.len() - 2];
            path.starts_with(prefix)
        } else {
            self.path == path
        }
    }
    
    /// Set require_auth flag (builder pattern)
    pub fn require_auth(mut self, require: bool) -> Self {
        self.require_auth = require;
        self
    }
}

/// Rate limit tracking
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Client identifier (IP or API key)
    pub client_id: String,
    /// Window start time
    pub window_start: chrono::DateTime<chrono::Utc>,
    /// Request count in current window
    pub request_count: u32,
    /// Rate limit configuration
    pub config: RateLimitConfig,
}

impl RateLimit {
    /// Create new rate limit tracker
    pub fn new(client_id: String, config: RateLimitConfig) -> Self {
        Self {
            client_id,
            window_start: chrono::Utc::now(),
            request_count: 0,
            config,
        }
    }
    
    /// Check if request is allowed
    pub fn check_request(&mut self) -> bool {
        let now = chrono::Utc::now();
        let window_duration = chrono::Duration::minutes(1);
        
        // Reset window if expired
        if now - self.window_start > window_duration {
            self.window_start = now;
            self.request_count = 0;
        }
        
        // Check burst capacity
        if self.request_count < self.config.burst_capacity {
            self.request_count += 1;
            return true;
        }
        
        // Check rate limit
        if self.request_count < self.config.requests_per_minute {
            self.request_count += 1;
            return true;
        }
        
        false
    }
    
    /// Get remaining requests
    pub fn remaining(&self) -> u32 {
        if self.request_count >= self.config.requests_per_minute {
            0
        } else {
            self.config.requests_per_minute - self.request_count
        }
    }
    
    /// Get reset time
    pub fn reset_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.window_start + chrono::Duration::minutes(1)
    }
}

/// Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Listen address
    pub bind_address: String,
    /// Listen port
    pub port: u16,
    /// Enable TLS
    pub tls_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS key path
    pub tls_key_path: Option<String>,
    /// Default rate limits
    pub default_rate_limits: RateLimitConfig,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Enable CORS
    pub cors_enabled: bool,
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
    /// Enable request logging
    pub request_logging: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            default_rate_limits: RateLimitConfig::default(),
            max_request_size: 10 * 1024 * 1024, // 10 MB
            request_timeout: 30,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
            request_logging: true,
        }
    }
}

/// Request information
#[derive(Debug, Clone)]
pub struct Request {
    /// Request ID
    pub id: String,
    /// Client IP address
    pub client_ip: IpAddr,
    /// HTTP method
    pub method: HttpMethod,
    /// Request path
    pub path: String,
    /// Query parameters
    pub query: HashMap<String, String>,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (if any)
    pub body: Option<Vec<u8>>,
    /// Tenant ID (if authenticated)
    pub tenant_id: Option<TenantId>,
    /// Session ID (if authenticated)
    pub session_id: Option<SessionId>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Response information
#[derive(Debug, Clone)]
pub struct Response {
    /// Status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

impl Response {
    /// Create a success response
    pub fn success(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body,
            processing_time_ms: 0,
        }
    }
    
    /// Create an error response
    pub fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: message.as_bytes().to_vec(),
            processing_time_ms: 0,
        }
    }
    
    /// Add header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for fault tolerance
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Service name
    pub service: String,
    /// Current state
    pub state: CircuitState,
    /// Failure count
    pub failure_count: u32,
    /// Success count (for half-open)
    pub success_count: u32,
    /// Last failure time
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    /// Failure threshold
    pub failure_threshold: u32,
    /// Success threshold (to close from half-open)
    pub success_threshold: u32,
    /// Timeout before attempting reset
    pub timeout_seconds: u64,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure: None,
            failure_threshold: 5,
            success_threshold: 3,
            timeout_seconds: 60,
        }
    }
    
    /// Record success
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    tracing::info!("Circuit breaker for {} closed", self.service);
                }
            }
            CircuitState::Open => {}
        }
    }
    
    /// Record failure
    pub fn record_failure(&mut self) {
        self.last_failure = Some(chrono::Utc::now());
        
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    tracing::warn!("Circuit breaker for {} opened", self.service);
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }
    
    /// Check if request is allowed
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = self.last_failure {
                    let elapsed = (chrono::Utc::now() - last_failure).num_seconds() as u64;
                    if elapsed >= self.timeout_seconds {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        tracing::info!("Circuit breaker for {} half-open", self.service);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
}

/// API Gateway
#[derive(Debug)]
pub struct ApiGateway {
    config: GatewayConfig,
    routes: std::sync::Arc<std::sync::Mutex<Vec<Route>>>,
    rate_limits: std::sync::Arc<std::sync::Mutex<HashMap<String, RateLimit>>>,
    circuit_breakers: std::sync::Arc<std::sync::Mutex<HashMap<String, CircuitBreaker>>>,
    request_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl ApiGateway {
    /// Create new API gateway
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            config,
            routes: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            rate_limits: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            circuit_breakers: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            request_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
    
    /// Initialize gateway
    pub async fn initialize(&mut self) -> SaasResult<()> {
        tracing::info!("Initializing API gateway on {}:{}", 
            self.config.bind_address, self.config.port);
        
        // Add default routes
        self.add_default_routes();
        
        tracing::info!("API gateway initialized with {} routes", 
            self.routes.lock().unwrap().len());
        
        Ok(())
    }
    
    /// Shutdown gateway
    pub async fn shutdown(&mut self) -> SaasResult<()> {
        tracing::info!("Shutting down API gateway");
        Ok(())
    }
    
    /// Add default routes
    fn add_default_routes(&self) {
        let mut routes = self.routes.lock().unwrap();
        
        // Health check route
        routes.push(Route::new(
            "health",
            "/health",
            "http://localhost:9000/health",
        ).require_auth(false));
        
        // API routes
        routes.push(Route::new(
            "api-sectors",
            "/api/v1/sectors/*",
            "http://localhost:9001",
        ));
        
        routes.push(Route::new(
            "api-sessions",
            "/api/v1/sessions/*",
            "http://localhost:9002",
        ));
        
        routes.push(Route::new(
            "api-tenants",
            "/api/v1/tenants/*",
            "http://localhost:9003",
        ));
        
        routes.push(Route::new(
            "api-containers",
            "/api/v1/containers/*",
            "http://localhost:9004",
        ));
        
        // WebSocket route
        routes.push(Route {
            id: "ws".to_string(),
            path: "/ws/*".to_string(),
            methods: vec![HttpMethod::Get],
            target: "ws://localhost:9005".to_string(),
            strip_prefix: true,
            add_headers: HashMap::new(),
            remove_headers: Vec::new(),
            require_auth: true,
            rate_limit: None,
            circuit_breaker: true,
            timeout_seconds: 0, // No timeout for WebSocket
        });
    }
    
    /// Add custom route
    pub fn add_route(&self, route: Route) {
        self.routes.lock().unwrap().push(route);
    }
    
    /// Remove route
    pub fn remove_route(&self, route_id: &str) {
        self.routes.lock().unwrap().retain(|r| r.id != route_id);
    }
    
    /// List routes
    pub fn list_routes(&self) -> Vec<Route> {
        self.routes.lock().unwrap().clone()
    }
    
    /// Process request
    pub async fn process_request(&self, request: Request) -> Response {
        let start = std::time::Instant::now();
        
        // Increment request count
        self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Log request
        if self.config.request_logging {
            tracing::info!("{} {} {} - {:?}",
                request.client_ip,
                request.method.as_str(),
                request.path,
                request.tenant_id,
            );
        }
        
        // Check rate limit
        let client_id = request.tenant_id.clone()
            .unwrap_or_else(|| request.client_ip.to_string());
        
        if let Some(rate_limit) = self.check_rate_limit(&client_id).await {
            if !rate_limit {
                return Response::error(429, "Rate limit exceeded")
                    .with_header("X-RateLimit-Remaining", "0");
            }
        }
        
        // Find matching route
        let route = {
            let routes = self.routes.lock().unwrap();
            routes.iter()
                .find(|r| r.matches(&request.path, request.method))
                .cloned()
        };
        
        let mut response = if let Some(route) = route {
            // Check authentication
            if route.require_auth && request.tenant_id.is_none() {
                return Response::error(401, "Authentication required");
            }
            
            // Check circuit breaker
            if route.circuit_breaker {
                let mut breakers = self.circuit_breakers.lock().unwrap();
                let breaker = breakers.entry(route.target.clone())
                    .or_insert_with(|| CircuitBreaker::new(&route.target));
                
                if !breaker.can_execute() {
                    return Response::error(503, "Service temporarily unavailable");
                }
            }
            
            // Route request
            self.route_request(&request, &route).await
        } else {
            Response::error(404, "Not found")
        };
        
        // Calculate processing time
        response.processing_time_ms = start.elapsed().as_millis() as u64;
        
        // Add rate limit headers
        if let Some(remaining) = self.get_rate_limit_remaining(&client_id).await {
            response = response.with_header("X-RateLimit-Remaining", remaining.to_string());
        }
        
        response
    }
    
    /// Check rate limit for client
    async fn check_rate_limit(&self, client_id: &str) -> Option<bool> {
        let mut limits = self.rate_limits.lock().unwrap();
        let limit = limits.entry(client_id.to_string())
            .or_insert_with(|| RateLimit::new(
                client_id.to_string(),
                self.config.default_rate_limits.clone(),
            ));
        
        Some(limit.check_request())
    }
    
    /// Get remaining rate limit
    async fn get_rate_limit_remaining(&self, client_id: &str) -> Option<u32> {
        let limits = self.rate_limits.lock().unwrap();
        limits.get(client_id).map(|l| l.remaining())
    }
    
    /// Route request to target service
    async fn route_request(&self, request: &Request, route: &Route) -> Response {
        // In real implementation, this would:
        // 1. Build target URL
        // 2. Forward request
        // 3. Handle response
        // 4. Update circuit breaker
        
        // Mock implementation
        let target_path = if route.strip_prefix {
            // Remove matched prefix
            if route.path.ends_with("/*") {
                let prefix = &route.path[..route.path.len() - 2];
                request.path.strip_prefix(prefix).unwrap_or(&request.path).to_string()
            } else {
                request.path.clone()
            }
        } else {
            request.path.clone()
        };
        
        let target_url = format!("{}{}", route.target, target_path);
        
        tracing::debug!("Routing {} to {}", request.path, target_url);
        
        // Simulate successful response
        Response::success(
            format!(r#"{{"routed_to": "{}"}}"#, target_url).into_bytes()
        )
    }
    
    /// Get request count
    pub fn request_count(&self) -> u64 {
        self.request_count.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Get circuit breaker status
    pub fn circuit_breaker_status(&self) -> HashMap<String, CircuitState> {
        self.circuit_breakers.lock().unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.state))
            .collect()
    }
}

/// Gateway builder for fluent API
#[derive(Debug)]
pub struct GatewayBuilder {
    config: GatewayConfig,
}

impl GatewayBuilder {
    /// Create new gateway builder
    pub fn new() -> Self {
        Self {
            config: GatewayConfig::default(),
        }
    }
    
    /// Bind address
    pub fn bind_address(mut self, address: impl Into<String>) -> Self {
        self.config.bind_address = address.into();
        self
    }
    
    /// Port
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }
    
    /// Enable TLS
    pub fn tls(mut self, cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        self.config.tls_enabled = true;
        self.config.tls_cert_path = Some(cert_path.into());
        self.config.tls_key_path = Some(key_path.into());
        self
    }
    
    /// Rate limits
    pub fn rate_limits(mut self, requests_per_minute: u32, burst: u32) -> Self {
        self.config.default_rate_limits = RateLimitConfig {
            requests_per_minute,
            requests_per_hour: requests_per_minute * 60,
            burst_capacity: burst,
        };
        self
    }
    
    /// Request timeout
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.config.request_timeout = seconds;
        self
    }
    
    /// Max request size
    pub fn max_request_size(mut self, size_mb: usize) -> Self {
        self.config.max_request_size = size_mb * 1024 * 1024;
        self
    }
    
    /// CORS origins
    pub fn cors_origins(mut self, origins: Vec<String>) -> Self {
        self.config.cors_origins = origins;
        self
    }
    
    /// Build gateway
    pub fn build(self) -> ApiGateway {
        ApiGateway::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_http_method() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
    }
    
    #[test]
    fn test_route_matching() {
        let route = Route::new("test", "/api/v1/sectors/*", "http://localhost:9001");
        
        assert!(route.matches("/api/v1/sectors/123", HttpMethod::Get));
        assert!(route.matches("/api/v1/sectors", HttpMethod::Post));
        assert!(!route.matches("/api/v1/users", HttpMethod::Get));
        assert!(!route.matches("/api/v1/sectors", HttpMethod::Patch)); // Not in default methods
    }
    
    #[test]
    fn test_rate_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 5,
            requests_per_hour: 100,
            burst_capacity: 2,
        };
        
        let mut limit = RateLimit::new("client-1".to_string(), config);
        
        // Burst capacity
        assert!(limit.check_request());
        assert!(limit.check_request());
        
        // Rate limit
        assert!(limit.check_request());
        assert!(limit.check_request());
        assert!(limit.check_request());
        
        // Exceeded
        assert!(!limit.check_request());
        
        // Check remaining
        assert_eq!(limit.remaining(), 0);
    }
    
    #[test]
    fn test_circuit_breaker() {
        let mut breaker = CircuitBreaker::new("test-service");
        
        // Initially closed
        assert!(breaker.can_execute());
        
        // Record failures to open
        for _ in 0..5 {
            breaker.record_failure();
        }
        
        assert_eq!(breaker.state, CircuitState::Open);
        assert!(!breaker.can_execute());
        
        // Simulate timeout (would need to wait in real test)
        // For unit test, manually set state
        breaker.state = CircuitState::HalfOpen;
        assert!(breaker.can_execute());
        
        // Record successes to close
        for _ in 0..3 {
            breaker.record_success();
        }
        
        assert_eq!(breaker.state, CircuitState::Closed);
    }
    
    #[test]
    fn test_response_helpers() {
        let response = Response::success(r#"{"status": "ok"}"#.as_bytes().to_vec());
        assert_eq!(response.status, 200);
        
        let error = Response::error(404, "Not found");
        assert_eq!(error.status, 404);
        assert_eq!(error.body, b"Not found");
        
        let with_header = response.with_header("Content-Type", "application/json");
        assert_eq!(with_header.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }
    
    #[tokio::test]
    async fn test_gateway_initialization() {
        let mut gateway = ApiGateway::new(GatewayConfig::default());
        gateway.initialize().await.unwrap();
        
        let routes = gateway.list_routes();
        assert!(!routes.is_empty());
        assert!(routes.iter().any(|r| r.id == "health"));
    }
    
    #[tokio::test]
    async fn test_gateway_request_routing() {
        let mut gateway = ApiGateway::new(GatewayConfig::default());
        gateway.initialize().await.unwrap();
        
        let request = Request {
            id: "req-1".to_string(),
            client_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            method: HttpMethod::Get,
            path: "/api/v1/sectors/123".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            tenant_id: Some("tenant-1".to_string()),
            session_id: Some("session-1".to_string()),
            timestamp: chrono::Utc::now(),
        };
        
        let response = gateway.process_request(request).await;
        assert_eq!(response.status, 200);
        assert!(!response.body.is_empty());
    }
    
    #[tokio::test]
    async fn test_gateway_404() {
        let mut gateway = ApiGateway::new(GatewayConfig::default());
        gateway.initialize().await.unwrap();
        
        let request = Request {
            id: "req-1".to_string(),
            client_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            method: HttpMethod::Get,
            path: "/unknown/path".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            tenant_id: None,
            session_id: None,
            timestamp: chrono::Utc::now(),
        };
        
        let response = gateway.process_request(request).await;
        assert_eq!(response.status, 404);
    }
    
    #[tokio::test]
    async fn test_gateway_auth_required() {
        let mut gateway = ApiGateway::new(GatewayConfig::default());
        gateway.initialize().await.unwrap();
        
        let request = Request {
            id: "req-1".to_string(),
            client_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            method: HttpMethod::Get,
            path: "/api/v1/sectors/123".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            tenant_id: None, // No authentication
            session_id: None,
            timestamp: chrono::Utc::now(),
        };
        
        let response = gateway.process_request(request).await;
        assert_eq!(response.status, 401);
    }
    
    #[test]
    fn test_gateway_builder() {
        let gateway = GatewayBuilder::new()
            .bind_address("127.0.0.1")
            .port(3000)
            .tls("/path/to/cert.pem", "/path/to/key.pem")
            .rate_limits(120, 20)
            .timeout(60)
            .max_request_size(50)
            .cors_origins(vec!["https://example.com".to_string()])
            .build();
        
        assert_eq!(gateway.config.bind_address, "127.0.0.1");
        assert_eq!(gateway.config.port, 3000);
        assert!(gateway.config.tls_enabled);
        assert_eq!(gateway.config.default_rate_limits.requests_per_minute, 120);
        assert_eq!(gateway.config.request_timeout, 60);
        assert_eq!(gateway.config.max_request_size, 50 * 1024 * 1024);
    }
}
