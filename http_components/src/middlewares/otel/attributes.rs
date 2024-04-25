use actix_http::header;
use actix_web::{
    dev::ServiceRequest,
    http::{Method, Version},
};
use opentelemetry::{Key, KeyValue, Value};
use opentelemetry_semantic_conventions::{
    resource::HOST_NAME,
    trace::{
        CLIENT_ADDRESS, HTTP_REQUEST_METHOD, HTTP_ROUTE, NETWORK_PROTOCOL_VERSION, SERVER_PORT,
        URL_PATH, URL_SCHEME, USER_AGENT_ORIGINAL,
    },
};
use otel::keys::{HTTP_SERVER_NAME, NET_PEER_IP};

#[inline]
pub(super) fn http_method_str(method: &Method) -> Value {
    match method {
        &Method::OPTIONS => "OPTIONS".into(),
        &Method::GET => "GET".into(),
        &Method::POST => "POST".into(),
        &Method::PUT => "PUT".into(),
        &Method::DELETE => "DELETE".into(),
        &Method::HEAD => "HEAD".into(),
        &Method::TRACE => "TRACE".into(),
        &Method::CONNECT => "CONNECT".into(),
        &Method::PATCH => "PATCH".into(),
        other => other.to_string().into(),
    }
}

#[inline]
pub(super) fn http_flavor(version: Version) -> Value {
    match version {
        Version::HTTP_09 => "HTTP/0.9".into(),
        Version::HTTP_10 => "HTTP/1.0".into(),
        Version::HTTP_11 => "HTTP/1.1".into(),
        Version::HTTP_2 => "HTTP/2".into(),
        Version::HTTP_3 => "HTTP/3".into(),
        other => format!("{:?}", other).into(),
    }
}

#[inline]
pub(super) fn http_scheme(scheme: &str) -> Value {
    match scheme {
        "http" => "http".into(),
        "https" => "https".into(),
        other => other.to_string().into(),
    }
}

pub(super) fn trace_attributes_from_request(
    req: &ServiceRequest,
    http_route: &str,
) -> Vec<KeyValue> {
    let conn_info = req.connection_info();

    let mut attributes = Vec::with_capacity(11);
    attributes.push(KeyValue::new::<Key, Value>(
        HTTP_REQUEST_METHOD.into(),
        http_method_str(req.method()),
    ));
    attributes.push(KeyValue::new::<Key, Value>(
        NETWORK_PROTOCOL_VERSION.into(),
        http_flavor(req.version()),
    ));
    attributes.push(KeyValue::new::<Key, Value>(
        HOST_NAME.into(),
        conn_info.host().to_string().into(),
    ));
    attributes.push(KeyValue::new::<Key, Value>(
        HTTP_ROUTE.into(),
        http_route.to_owned().into(),
    ));
    attributes.push(KeyValue::new::<Key, Value>(
        URL_SCHEME.into(),
        http_scheme(conn_info.scheme()),
    ));

    let server_name = req.app_config().host();
    if server_name != conn_info.host() {
        attributes.push(KeyValue::new::<Key, Value>(
            HTTP_SERVER_NAME,
            server_name.to_string().into(),
        ));
    }
    if let Some(port) = conn_info
        .host()
        .split_terminator(':')
        .nth(1)
        .and_then(|port| port.parse::<i64>().ok())
    {
        if port != 80 && port != 443 {
            attributes.push(KeyValue::new::<Key, Value>(SERVER_PORT.into(), port.into()));
        }
    }
    if let Some(path) = req.uri().path_and_query() {
        attributes.push(KeyValue::new::<Key, Value>(
            URL_PATH.into(),
            path.as_str().to_string().into(),
        ));
    }
    if let Some(user_agent) = req
        .headers()
        .get(header::USER_AGENT)
        .and_then(|s| s.to_str().ok())
    {
        attributes.push(KeyValue::new::<Key, Value>(
            USER_AGENT_ORIGINAL.into(),
            user_agent.to_string().into(),
        ));
    }
    let remote_addr = conn_info.realip_remote_addr();
    if let Some(remote) = remote_addr {
        attributes.push(KeyValue::new::<Key, Value>(
            CLIENT_ADDRESS.into(),
            remote.to_string().into(),
        ));
    }
    if let Some(peer_addr) = req.peer_addr().map(|socket| socket.ip().to_string()) {
        if Some(peer_addr.as_str()) != remote_addr {
            // Client is going through a proxy
            attributes.push(KeyValue::new::<Key, Value>(NET_PEER_IP, peer_addr.into()));
        }
    }

    attributes
}

pub(super) fn metrics_attributes_from_request(
    req: &ServiceRequest,
    http_target: &str,
) -> Vec<KeyValue> {
    let conn_info = req.connection_info();
    let host = conn_info.host().to_owned();

    let mut attributes = Vec::with_capacity(11);
    attributes.push(KeyValue::new::<Key, Value>(
        HTTP_REQUEST_METHOD.into(),
        http_method_str(req.method()),
    ));
    attributes.push(KeyValue::new::<Key, Value>(
        NETWORK_PROTOCOL_VERSION.into(),
        http_flavor(req.version()),
    ));
    attributes.push(KeyValue::new::<Key, Value>(
        HOST_NAME.into(),
        host.clone().into(),
    ));
    attributes.push(KeyValue::new::<Key, Value>(
        URL_PATH.into(),
        http_target.to_owned().into(),
    ));
    attributes.push(KeyValue::new::<Key, Value>(
        URL_SCHEME.into(),
        http_scheme(conn_info.scheme()),
    ));

    let server_name = req.app_config().host();
    if !server_name.eq(&host) {
        attributes.push(HTTP_SERVER_NAME.string(server_name.to_string()));
    }

    if let Some(port) = host.split_terminator(':').nth(1) {
        attributes.push(KeyValue::new::<Key, Value>(
            SERVER_PORT.into(),
            port.to_string().into(),
        ))
    };

    let remote_addr = conn_info.realip_remote_addr();
    if let Some(peer_addr) = req.peer_addr().map(|socket| socket.ip().to_string()) {
        if Some(peer_addr.as_str()) != remote_addr {
            // Client is going through a proxy
            attributes.push(NET_PEER_IP.string(peer_addr))
        }
    }

    attributes
}
