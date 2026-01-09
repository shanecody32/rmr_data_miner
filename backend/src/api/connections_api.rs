use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use sea_orm::{prelude::*, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::entities::{now_playing_connections, payload_mappings};
use crate::api::AppState;
use crate::http_headers::normalize_headers_for_storage;
use crate::poller::utils::fetch_and_parse;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_connections).post(create_connection))
        .route("/{id}", get(get_connection).put(update_connection).delete(delete_connection))
        .route("/{id}/enable", post(enable_connection))
        .route("/{id}/disable", post(disable_connection))
        .route("/{id}/test", post(test_connection))
        .route("/mappings", get(list_mappings).post(create_mapping))
        .route("/mappings/{id}", get(get_mapping).put(update_mapping).delete(delete_mapping))
}

#[derive(Deserialize)]
pub struct CreateConnection {
    pub station_id: Uuid,
    pub payload_mapping_id: Option<Uuid>,
    pub name: String,
    pub connection_type: String,
    pub url: String,
    pub poll_interval_seconds: i32,
    pub headers_json: Option<serde_json::Value>,
    pub enabled: bool,
    #[serde(default)]
    pub use_duration_polling: bool,
}

#[derive(Deserialize)]
pub struct CreateMapping {
    pub name: String,
    pub description: Option<String>,
    pub mapping_json: serde_json::Value,
}

async fn list_mappings(State(state): State<AppState>) -> Result<Json<Vec<payload_mappings::Model>>, StatusCode> {
    payload_mappings::Entity::find()
        .all(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_mapping(
    State(state): State<AppState>,
    Json(payload): Json<CreateMapping>,
) -> Result<Json<payload_mappings::Model>, StatusCode> {
    let now = Utc::now().fixed_offset();
    let mapping = payload_mappings::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        description: Set(payload.description),
        mapping_json: Set(payload.mapping_json),
        created_at: Set(now),
        updated_at: Set(now),
    };

    mapping.insert(&state.db)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to create mapping: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn get_mapping(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<payload_mappings::Model>, StatusCode> {
    payload_mappings::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_mapping(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateMapping>,
) -> Result<Json<payload_mappings::Model>, StatusCode> {
    let mapping = payload_mappings::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut mapping: payload_mappings::ActiveModel = mapping.into();
    mapping.name = Set(payload.name);
    mapping.description = Set(payload.description);
    mapping.mapping_json = Set(payload.mapping_json);
    mapping.updated_at = Set(Utc::now().fixed_offset());

    mapping.update(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn delete_mapping(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let res = payload_mappings::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn list_connections(State(state): State<AppState>) -> Result<Json<Vec<now_playing_connections::Model>>, StatusCode> {
    now_playing_connections::Entity::find()
        .all(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_connection(
    State(state): State<AppState>,
    Json(payload): Json<CreateConnection>,
) -> Result<Json<now_playing_connections::Model>, StatusCode> {
    let now = Utc::now().fixed_offset();
    let headers_json = normalize_headers_for_storage(
        &payload.connection_type,
        payload.headers_json,
    );
    let conn = now_playing_connections::ActiveModel {
        id: Set(Uuid::new_v4()),
        station_id: Set(payload.station_id),
        payload_mapping_id: Set(payload.payload_mapping_id),
        name: Set(payload.name),
        connection_type: Set(payload.connection_type),
        url: Set(payload.url),
        poll_interval_seconds: Set(payload.poll_interval_seconds),
        headers_json: Set(headers_json),
        enabled: Set(payload.enabled),
        use_duration_polling: Set(payload.use_duration_polling),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    conn.insert(&state.db)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to create connection: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn get_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<now_playing_connections::Model>, StatusCode> {
    now_playing_connections::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateConnection>,
) -> Result<Json<now_playing_connections::Model>, StatusCode> {
    let conn = now_playing_connections::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut conn: now_playing_connections::ActiveModel = conn.into();
    let headers_json = normalize_headers_for_storage(
        &payload.connection_type,
        payload.headers_json,
    );
    conn.station_id = Set(payload.station_id);
    conn.payload_mapping_id = Set(payload.payload_mapping_id);
    conn.name = Set(payload.name);
    conn.connection_type = Set(payload.connection_type);
    conn.url = Set(payload.url);
    conn.poll_interval_seconds = Set(payload.poll_interval_seconds);
    conn.headers_json = Set(headers_json);
    conn.enabled = Set(payload.enabled);
    conn.use_duration_polling = Set(payload.use_duration_polling);
    conn.updated_at = Set(Utc::now().fixed_offset());

    conn.update(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn delete_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let res = now_playing_connections::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn enable_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let conn = now_playing_connections::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut conn: now_playing_connections::ActiveModel = conn.into();
    conn.enabled = Set(true);
    conn.updated_at = Set(Utc::now().fixed_offset());

    conn.update(&state.db)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn disable_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let conn = now_playing_connections::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut conn: now_playing_connections::ActiveModel = conn.into();
    conn.enabled = Set(false);
    conn.updated_at = Set(Utc::now().fixed_offset());

    conn.update(&state.db)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Serialize)]
pub struct TestResult {
    pub status: i32,
    pub content_type: Option<String>,
    pub raw_payload: serde_json::Value,
    pub extracted: ExtractedFields,
}

#[derive(Serialize)]
pub struct ExtractedFields {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
}

async fn test_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TestResult>, StatusCode> {
    let conn = now_playing_connections::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if conn.connection_type.eq_ignore_ascii_case("ws_json") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mapping = if let Some(mapping_id) = conn.payload_mapping_id {
        payload_mappings::Entity::find_by_id(mapping_id)
            .one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        None
    };

    let result = fetch_and_parse(&conn, mapping.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Test fetch failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(TestResult {
        status: result.status,
        content_type: result.content_type,
        raw_payload: result.raw_payload,
        extracted: ExtractedFields {
            artist: result.reported_artist,
            title: result.reported_title,
            album: result.reported_album,
        },
    }))
}
