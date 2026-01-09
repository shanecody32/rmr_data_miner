use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use sea_orm::{prelude::*, QueryOrder, QuerySelect, EntityTrait};
use serde::Deserialize;
use uuid::Uuid;
use crate::entities::raw_now_playing_events;
use crate::api::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_events).on(axum::routing::MethodFilter::DELETE, clear_events))
        .route("/{id}", get(get_event))
}

async fn clear_events(State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    raw_now_playing_events::Entity::delete_many()
        .exec(&state.db)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize)]
pub struct EventQuery {
    pub station_id: Option<Uuid>,
    pub connection_id: Option<Uuid>,
    pub limit: Option<u64>,
    pub before: Option<DateTimeWithTimeZone>,
}

async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Vec<raw_now_playing_events::Model>>, StatusCode> {
    let mut select = raw_now_playing_events::Entity::find()
        .order_by_desc(raw_now_playing_events::Column::ObservedAt);

    if let Some(station_id) = query.station_id {
        select = select.filter(raw_now_playing_events::Column::StationId.eq(station_id));
    }

    if let Some(connection_id) = query.connection_id {
        select = select.filter(raw_now_playing_events::Column::ConnectionId.eq(connection_id));
    }

    if let Some(before) = query.before {
        select = select.filter(raw_now_playing_events::Column::ObservedAt.lt(before));
    }

    let limit = query.limit.unwrap_or(100);
    select = select.limit(limit);

    select.all(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<raw_now_playing_events::Model>, StatusCode> {
    raw_now_playing_events::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
