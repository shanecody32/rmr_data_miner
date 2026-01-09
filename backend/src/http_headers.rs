use serde_json::{json, Value};
use std::collections::HashMap;

pub fn normalize_headers_for_storage(
    connection_type: &str,
    headers_json: Option<Value>,
) -> Option<Value> {
    if should_default_headers(connection_type) && is_empty_headers(&headers_json) {
        Some(default_headers_value(connection_type))
    } else {
        headers_json
    }
}

pub fn headers_value_to_map(headers: &Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(obj) = headers.as_object() {
        for (k, v) in obj {
            if let Some(v_str) = v.as_str() {
                map.insert(k.to_string(), v_str.to_string());
            } else if v.is_number() || v.is_boolean() {
                map.insert(k.to_string(), v.to_string());
            }
        }
    }
    map
}

pub fn default_headers_value(connection_type: &str) -> Value {
    match connection_type.to_ascii_lowercase().as_str() {
        "http_xml" => json!({
            "Accept": "application/xml, text/xml;q=0.9, */*;q=0.8",
            "Cache-Control": "no-cache",
            "Pragma": "no-cache",
        }),
        "http_text" => json!({
            "Accept": "text/plain, */*;q=0.8",
            "Cache-Control": "no-cache",
            "Pragma": "no-cache",
        }),
        "rss" => json!({
            "Accept": "application/rss+xml, application/xml;q=0.9, */*;q=0.8",
            "Cache-Control": "no-cache",
            "Pragma": "no-cache",
        }),
        _ => json!({
            "Accept": "application/json, text/javascript, */*; q=0.01",
            "Cache-Control": "no-cache",
            "Pragma": "no-cache",
        }),
    }
}

pub fn browser_headers_value(connection_type: &str, url: &str) -> Value {
    let mut headers = default_headers_value(connection_type);
    if let Some(obj) = headers.as_object_mut() {
        obj.insert(
            "Accept-Language".to_string(),
            Value::String("en-US,en;q=0.9".to_string()),
        );
        obj.insert(
            "Cache-Control".to_string(),
            Value::String("no-cache".to_string()),
        );
        obj.insert("Pragma".to_string(), Value::String("no-cache".to_string()));
        if let Some(origin) = origin_for_url(url) {
            obj.insert("Origin".to_string(), Value::String(origin.clone()));
            obj.insert("Referer".to_string(), Value::String(format!("{}/", origin)));
        }
    }
    headers
}

pub fn should_default_headers(connection_type: &str) -> bool {
    !matches!(connection_type.to_ascii_lowercase().as_str(), "ws_json")
}

fn is_empty_headers(headers: &Option<Value>) -> bool {
    match headers {
        None => true,
        Some(value) => value.as_object().map(|obj| obj.is_empty()).unwrap_or(false),
    }
}

fn origin_for_url(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    let scheme = parsed.scheme();
    let host = parsed.host_str()?;
    let port = parsed.port();
    let origin = match port {
        Some(p) if !is_default_port(scheme, p) => format!("{}://{}:{}", scheme, host, p),
        _ => format!("{}://{}", scheme, host),
    };
    Some(origin)
}

fn is_default_port(scheme: &str, port: u16) -> bool {
    matches!((scheme, port), ("http", 80) | ("https", 443))
}
