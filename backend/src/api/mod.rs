use axum::Router;
use sea_orm::prelude::*;

pub mod stations_api;
pub mod connections_api;
pub mod events_api;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/stations", stations_api::router())
        .nest("/connections", connections_api::router())
        .nest("/events", events_api::router())
        .with_state(state)
}
