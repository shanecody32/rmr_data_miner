use sea_orm::{prelude::*, Set, QueryOrder};
use chrono::{Utc, DateTime, FixedOffset};
use sha2::{Sha256, Digest};
use crate::entities::{now_playing_connections, raw_now_playing_events, payload_mappings};
use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::escape::unescape;
use std::collections::HashMap;
use futures::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::Message;
use std::time::Duration;
use crate::http_headers::{
    browser_headers_value,
    default_headers_value,
    headers_value_to_map,
    should_default_headers,
};

pub struct FetchResult {
    pub status: i32,
    pub content_type: Option<String>,
    pub raw_payload: serde_json::Value,
    pub reported_artist: Option<String>,
    pub reported_title: Option<String>,
    pub reported_album: Option<String>,
    pub reported_at: Option<DateTime<FixedOffset>>,
}

pub async fn poll_connection(db: &DatabaseConnection, conn: &now_playing_connections::Model) -> Result<(), DbErr> {
    let now = Utc::now().fixed_offset();
    
    let mapping = if let Some(mapping_id) = conn.payload_mapping_id {
        payload_mappings::Entity::find_by_id(mapping_id).one(db).await?
    } else {
        None
    };

    let result = match fetch_and_parse(conn, mapping.as_ref()).await {
        Ok(res) => res,
        Err(e) => {
            let mut active_conn: now_playing_connections::ActiveModel = conn.clone().into();
            active_conn.last_polled_at = Set(Some(now));
            active_conn.last_status = Set(Some("FETCH_ERROR".to_string()));
            active_conn.last_error = Set(Some(e.to_string()));
            active_conn.update(db).await?;
            return Ok(());
        }
    };

    process_fetch_result(db, conn, result, now).await?;

    Ok(())
}

pub async fn fetch_and_parse(
    conn: &now_playing_connections::Model,
    mapping: Option<&payload_mappings::Model>,
) -> Result<FetchResult, Box<dyn std::error::Error + Send + Sync>> {
    if is_ws_connection_type(&conn.connection_type) {
        return Err("WebSocket connections are handled by the WS listener".into());
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let (headers_map, used_default_headers) = resolve_headers_for_request(conn);

    let mut resp = match send_request(&client, &conn.url, &headers_map).await {
        Ok(response) => response,
        Err(err) => {
            if used_default_headers {
                let browser_headers = headers_value_to_map(
                    &browser_headers_value(&conn.connection_type, &conn.url),
                );
                send_request(&client, &conn.url, &browser_headers).await?
            } else {
                return Err(err.into());
            }
        }
    };

    if used_default_headers && !resp.status().is_success() {
        let browser_headers = headers_value_to_map(
            &browser_headers_value(&conn.connection_type, &conn.url),
        );
        if let Ok(retry_resp) = send_request(&client, &conn.url, &browser_headers).await {
            resp = retry_resp;
        }
    }
    let status = resp.status().as_u16() as i32;
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let body_bytes = resp.bytes().await?;
    let raw_payload: serde_json::Value = if is_xml_connection_type(&conn.connection_type) {
        let body_str = String::from_utf8_lossy(&body_bytes).to_string();
        let normalized_xml = normalize_xml_storage(&body_str);
        serde_json::Value::String(normalized_xml)
    } else if let Ok(json) = serde_json::from_slice(&body_bytes) {
        json
    } else {
        // Try XML if it looks like XML or if content-type suggests it
        let body_str = String::from_utf8_lossy(&body_bytes).to_string();
        if body_str.trim_start().starts_with('<') {
            let normalized_xml = normalize_xml_storage(&body_str);
            let parse_xml = normalize_xml_for_parse(&normalized_xml);
            match serde_xml_rs::from_str::<serde_json::Value>(&parse_xml) {
                Ok(json) => json,
                Err(_) => serde_json::Value::String(normalized_xml),
            }
        } else {
            serde_json::Value::String(body_str.to_string())
        }
    };

    let (artist, title, album, reported_at) = extract_fields(
        &raw_payload,
        mapping,
        &conn.connection_type,
    );

    Ok(FetchResult {
        status,
        content_type,
        raw_payload,
        reported_artist: artist,
        reported_title: title,
        reported_album: album,
        reported_at,
    })
}

fn resolve_headers_for_request(
    conn: &now_playing_connections::Model,
) -> (HashMap<String, String>, bool) {
    if !should_default_headers(&conn.connection_type) {
        if let Some(headers) = &conn.headers_json {
            return (headers_value_to_map(headers), false);
        }
        return (HashMap::new(), false);
    }

    if let Some(headers) = &conn.headers_json {
        if headers.as_object().map(|obj| !obj.is_empty()).unwrap_or(false) {
            return (headers_value_to_map(headers), false);
        }
    }

    let default_headers = default_headers_value(&conn.connection_type);
    (headers_value_to_map(&default_headers), true)
}

async fn send_request(
    client: &reqwest::Client,
    url: &str,
    headers: &HashMap<String, String>,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut rb = client.get(url);
    for (k, v) in headers {
        rb = rb.header(k, v);
    }
    rb.send().await
}

pub async fn run_ws_connection(
    db: DatabaseConnection,
    conn: now_playing_connections::Model,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mapping = if let Some(mapping_id) = conn.payload_mapping_id {
        payload_mappings::Entity::find_by_id(mapping_id).one(&db).await?
    } else {
        None
    };

    let mut backoff_seconds = 1u64;

    loop {
        if !is_connection_enabled(&db, conn.id).await? {
            update_connection_status(&db, &conn, Some("DISABLED".to_string()), None).await?;
            return Ok(());
        }

        match tokio_tungstenite::connect_async(&conn.url).await {
            Ok((ws_stream, _)) => {
                update_connection_status(&db, &conn, Some("WS_CONNECTED".to_string()), None).await?;
                let (mut write, mut read) = ws_stream.split();

                let subscribe_message = build_ws_subscribe_message(&conn)?;
                write.send(Message::Text(subscribe_message.into())).await?;

                backoff_seconds = 1;
                let mut health_check = tokio::time::interval(Duration::from_secs(30));

                loop {
                    tokio::select! {
                        msg = read.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                        handle_ws_payload(&db, &conn, mapping.as_ref(), json).await?;
                                    }
                                }
                                Some(Ok(Message::Binary(bin))) => {
                                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bin) {
                                        handle_ws_payload(&db, &conn, mapping.as_ref(), json).await?;
                                    }
                                }
                                Some(Ok(Message::Ping(payload))) => {
                                    write.send(Message::Pong(payload)).await?;
                                }
                                Some(Ok(Message::Close(_))) => {
                                    update_connection_status(&db, &conn, Some("WS_CLOSED".to_string()), None).await?;
                                    break;
                                }
                                Some(Err(e)) => {
                                    update_connection_status(&db, &conn, Some("WS_ERROR".to_string()), Some(e.to_string())).await?;
                                    break;
                                }
                                None => {
                                    update_connection_status(&db, &conn, Some("WS_DISCONNECTED".to_string()), None).await?;
                                    break;
                                }
                                _ => {}
                            }
                        }
                        _ = health_check.tick() => {
                            if !is_connection_enabled(&db, conn.id).await? {
                                update_connection_status(&db, &conn, Some("DISABLED".to_string()), None).await?;
                                return Ok(());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                update_connection_status(&db, &conn, Some("WS_CONNECT_ERROR".to_string()), Some(e.to_string())).await?;
            }
        }

        tokio::time::sleep(Duration::from_secs(backoff_seconds)).await;
        backoff_seconds = (backoff_seconds * 2).min(60);
    }
}

async fn handle_ws_payload(
    db: &DatabaseConnection,
    conn: &now_playing_connections::Model,
    mapping: Option<&payload_mappings::Model>,
    raw_payload: serde_json::Value,
) -> Result<(), DbErr> {
    let now = Utc::now().fixed_offset();
    let (artist, title, album, reported_at) = extract_fields(
        &raw_payload,
        mapping,
        &conn.connection_type,
    );

    let result = FetchResult {
        status: 200,
        content_type: Some("application/json".to_string()),
        raw_payload,
        reported_artist: artist,
        reported_title: title,
        reported_album: album,
        reported_at,
    };

    process_fetch_result(db, conn, result, now).await
}

async fn process_fetch_result(
    db: &DatabaseConnection,
    conn: &now_playing_connections::Model,
    result: FetchResult,
    now: DateTime<FixedOffset>,
) -> Result<(), DbErr> {
    let payload_str = serde_json::to_string(&result.raw_payload).unwrap_or_default();
    let payload_hash = calculate_hash(conn.station_id, conn.id, &payload_str);

    // Check for deduplication
    let last_event = raw_now_playing_events::Entity::find()
        .filter(raw_now_playing_events::Column::ConnectionId.eq(conn.id))
        .order_by_desc(raw_now_playing_events::Column::ObservedAt)
        .one(db)
        .await?;

    let is_payload_duplicate = last_event.as_ref().map(|e| e.payload_hash == payload_hash).unwrap_or(false);

    let is_content_duplicate = if let (Some(last), current_artist, current_title) = (&last_event, &result.reported_artist, &result.reported_title) {
        let last_artist = last.reported_artist.as_ref();
        let last_title = last.reported_title.as_ref();

        // If both are None/empty, we can't really say it's a duplicate based on content,
        // but we rely on payload hash then.
        // If they are identical to last seen, it's a duplicate.
        last_artist == current_artist.as_ref() && last_title == current_title.as_ref()
    } else {
        false
    };

    if !is_payload_duplicate && !is_content_duplicate {
        let event = raw_now_playing_events::ActiveModel {
            id: Set(Uuid::new_v4()),
            station_id: Set(conn.station_id),
            connection_id: Set(conn.id),
            observed_at: Set(now),
            reported_at: Set(result.reported_at),
            reported_artist: Set(result.reported_artist),
            reported_title: Set(result.reported_title),
            reported_album: Set(result.reported_album),
            raw_payload: Set(result.raw_payload),
            payload_hash: Set(payload_hash),
            http_status: Set(Some(result.status)),
            content_type: Set(result.content_type),
            created_at: Set(now),
            ..Default::default()
        };
        event.insert(db).await?;
    }

    let mut active_conn: now_playing_connections::ActiveModel = conn.clone().into();
    active_conn.last_polled_at = Set(Some(now));
    active_conn.last_status = Set(Some("OK".to_string()));
    active_conn.last_error = Set(None);
    active_conn.update(db).await?;

    Ok(())
}

fn extract_fields(
    payload: &serde_json::Value,
    mapping: Option<&payload_mappings::Model>,
    connection_type: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<DateTime<FixedOffset>>,
) {
    if let Some(m) = mapping {
        let mapping_obj = m.mapping_json.as_object();

        if is_xml_connection_type(connection_type) {
            if let Some(xml_str) = payload.as_str() {
                let xml_values = extract_xml_values(xml_str);
                let list_path = mapping_obj
                    .and_then(|o| o.get("list_path"))
                    .and_then(|v| v.as_str());

                let artist = mapping_obj
                    .and_then(|o| o.get("artist_path"))
                    .and_then(|v| v.as_str())
                    .and_then(|p| xml_lookup(&xml_values, list_path, p));

                let title = mapping_obj
                    .and_then(|o| o.get("title_path"))
                    .and_then(|v| v.as_str())
                    .and_then(|p| xml_lookup(&xml_values, list_path, p));

                let album = mapping_obj
                    .and_then(|o| o.get("album_path"))
                    .and_then(|v| v.as_str())
                    .and_then(|p| xml_lookup(&xml_values, list_path, p));

                let reported_at = mapping_obj
                    .and_then(|o| o.get("reported_at_path"))
                    .and_then(|v| v.as_str())
                    .and_then(|p| xml_lookup(&xml_values, list_path, p))
                    .as_deref()
                    .and_then(parse_reported_at);

                return (artist, title, album, reported_at);
            }
        }

        let mut candidates: Vec<&serde_json::Value> = vec![payload];
        if let Some(obj) = payload.as_object() {
            if obj.len() == 1 {
                if let Some((_, value)) = obj.iter().next() {
                    candidates.push(value);
                }
            }
        }

        for base in candidates {
            let mut target_payload = base;

            if let Some(list_path) = mapping_obj
                .and_then(|o| o.get("list_path"))
                .and_then(|v| v.as_str())
            {
                if let Some(list) = get_path(base, list_path) {
                    if let Some(arr) = list.as_array() {
                        if let Some(first) = arr.first() {
                            target_payload = first;
                        }
                    }
                }
            }

            let artist = mapping_obj
                .and_then(|o| o.get("artist_path"))
                .and_then(|v| v.as_str())
                .and_then(|p| get_path(target_payload, p))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let title = mapping_obj
                .and_then(|o| o.get("title_path"))
                .and_then(|v| v.as_str())
                .and_then(|p| get_path(target_payload, p))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let album = mapping_obj
                .and_then(|o| o.get("album_path"))
                .and_then(|v| v.as_str())
                .and_then(|p| get_path(target_payload, p))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let reported_at = mapping_obj
                .and_then(|o| o.get("reported_at_path"))
                .and_then(|v| v.as_str())
                .and_then(|p| get_path(target_payload, p))
                .and_then(|v| v.as_str())
                .and_then(parse_reported_at);

            if artist.is_some() || title.is_some() || album.is_some() || reported_at.is_some() {
                return (artist, title, album, reported_at);
            }
        }

        return (None, None, None, None);
    }

    // Best-effort extraction (legacy)
    let mut artist = None;
    let mut title = None;
    let mut album = None;

    if let Some(obj) = payload.as_object() {
        artist = obj.get("artist").or_else(|| obj.get("artistName")).and_then(|v| v.as_str()).map(|s| s.to_string());
        title = obj.get("title").or_else(|| obj.get("song")).or_else(|| obj.get("trackName")).and_then(|v| v.as_str()).map(|s| s.to_string());
        album = obj.get("album").or_else(|| obj.get("collectionName")).and_then(|v| v.as_str()).map(|s| s.to_string());
    } else if let Some(arr) = payload.as_array() {
        if let Some(first) = arr.first() {
            return extract_fields(first, None, connection_type);
        }
    }

    (artist, title, album, None)
}

fn parse_reported_at(value: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .or_else(|| DateTime::parse_from_str(value, "%d %b %Y %H:%M:%S").ok())
}

pub fn is_ws_connection_type(connection_type: &str) -> bool {
    matches!(connection_type.to_ascii_lowercase().as_str(), "ws_json")
}

fn is_xml_connection_type(connection_type: &str) -> bool {
    matches!(
        connection_type.to_ascii_lowercase().as_str(),
        "http_xml" | "rss"
    )
}

fn build_ws_subscribe_message(
    conn: &now_playing_connections::Model,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(headers) = &conn.headers_json {
        if let Some(obj) = headers.as_object() {
            if let Some(payload) = obj.get("subscribe_payload").or_else(|| obj.get("subscribe_message")) {
                if let Some(text) = payload.as_str() {
                    return Ok(text.to_string());
                }
                return Ok(payload.to_string());
            }

            let service_id = obj.get("serviceId").or_else(|| obj.get("service_id"));
            if let Some(value) = service_id {
                let service_id_value = if let Some(text) = value.as_str() {
                    serde_json::Value::String(text.to_string())
                } else {
                    value.clone()
                };
                let payload = serde_json::json!({
                    "action": "subscribe",
                    "serviceId": service_id_value,
                });
                return Ok(payload.to_string());
            }
        }
    }

    Err("Missing subscribe_payload or serviceId in headers_json for ws_json connection".into())
}

async fn is_connection_enabled(db: &DatabaseConnection, id: Uuid) -> Result<bool, DbErr> {
    let conn = now_playing_connections::Entity::find_by_id(id).one(db).await?;
    Ok(conn.map(|c| c.enabled).unwrap_or(false))
}

async fn update_connection_status(
    db: &DatabaseConnection,
    conn: &now_playing_connections::Model,
    status: Option<String>,
    error: Option<String>,
) -> Result<(), DbErr> {
    let mut active_conn: now_playing_connections::ActiveModel = conn.clone().into();
    active_conn.last_status = Set(status);
    active_conn.last_error = Set(error);
    active_conn.updated_at = Set(Utc::now().fixed_offset());
    active_conn.update(db).await?;
    Ok(())
}

fn extract_xml_values(xml: &str) -> HashMap<String, String> {
    let normalized = normalize_xml_for_parse(xml);
    let mut reader = Reader::from_str(&normalized);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut values: HashMap<String, String> = HashMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                stack.push(name);
            }
            Ok(Event::End(_)) => {
                stack.pop();
            }
            Ok(Event::Text(e)) => {
                if let Ok(raw) = std::str::from_utf8(e.as_ref()) {
                    if let Ok(unescaped) = unescape(raw) {
                        let text = unescaped.into_owned();
                        if !text.is_empty() {
                            if let Some(path) = xml_stack_path(&stack) {
                                values.entry(path).or_insert(text);
                            }
                        }
                    }
                }
            }
            Ok(Event::CData(e)) => {
                if let Ok(text) = std::str::from_utf8(e.as_ref()) {
                    if !text.is_empty() {
                        if let Some(path) = xml_stack_path(&stack) {
                            values.entry(path).or_insert(text.to_string());
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    values
}

fn xml_stack_path(stack: &[String]) -> Option<String> {
    if stack.is_empty() {
        None
    } else {
        Some(stack.join("."))
    }
}

fn xml_lookup(
    values: &HashMap<String, String>,
    list_path: Option<&str>,
    field_path: &str,
) -> Option<String> {
    let combined = list_path
        .map(|base| format!("{}.{}", base, field_path))
        .unwrap_or_else(|| field_path.to_string());

    if let Some(value) = values.get(&combined) {
        return Some(value.clone());
    }

    let needle = format!(".{}", field_path);
    values
        .iter()
        .find_map(|(key, value)| {
            if key == field_path || key.ends_with(&needle) {
                Some(value.clone())
            } else {
                None
            }
        })
}

fn normalize_xml_storage(input: &str) -> String {
    input.replace('\n', "").replace('\t', "").replace('\r', "")
}

fn normalize_xml_for_parse(input: &str) -> String {
    let normalized = normalize_xml_storage(input);
    if normalized.trim_start().starts_with("<?xml") {
        if let Some(idx) = normalized.find("?>") {
            return normalized[idx + 2..].to_string();
        }
    }
    normalized
}

fn get_path<'a>(val: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut curr = val;
    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if let Some(obj) = curr.as_object() {
            if let Some(next) = obj.get(part) {
                curr = next;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    Some(curr)
}

fn calculate_hash(station_id: Uuid, conn_id: Uuid, payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(station_id.as_bytes());
    hasher.update(conn_id.as_bytes());
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}
