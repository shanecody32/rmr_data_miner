use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use sea_orm::{prelude::*, Set};
use serde::Deserialize;
use uuid::Uuid;
use chrono::Utc;
use crate::entities::stations;
use crate::api::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_stations).post(create_station))
        .route("/{id}", get(get_station).put(update_station).delete(delete_station))
}

#[derive(Deserialize)]
pub struct CreateStation {
    pub name: String,
    pub callsign: Option<String>,
    pub website_url: Option<String>,
}

async fn list_stations(State(state): State<AppState>) -> Result<Json<Vec<stations::Model>>, StatusCode> {
    stations::Entity::find()
        .all(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_station(
    State(state): State<AppState>,
    Json(payload): Json<CreateStation>,
) -> Result<Json<stations::Model>, StatusCode> {
    let now = Utc::now().fixed_offset();
    let station = stations::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        callsign: Set(payload.callsign),
        website_url: Set(payload.website_url),
        created_at: Set(now),
        updated_at: Set(now),
    };

    station.insert(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_station(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<stations::Model>, StatusCode> {
    stations::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_station(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateStation>,
) -> Result<Json<stations::Model>, StatusCode> {
    let station = stations::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut station: stations::ActiveModel = station.into();
    station.name = Set(payload.name);
    station.callsign = Set(payload.callsign);
    station.website_url = Set(payload.website_url);
    station.updated_at = Set(Utc::now().fixed_offset());

    station.update(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn delete_station(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let res = stations::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
