use sea_orm::prelude::*;
use std::time::Duration;
use tokio::task::JoinHandle;
use crate::entities::now_playing_connections;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod utils;

pub fn start_poller(db: DatabaseConnection) -> JoinHandle<()> {
    let active_ws_connections: Arc<Mutex<HashSet<Uuid>>> = Arc::new(Mutex::new(HashSet::new()));
    let active_ws_connections_clone = active_ws_connections.clone();
    tokio::spawn(async move {
        tracing::info!("Starting poller scheduler loop");
        loop {
            if let Err(e) = poll_all_enabled(&db, active_ws_connections_clone.clone()).await {
                tracing::error!("Error in poller loop: {:?}", e);
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    })
}

async fn poll_all_enabled(
    db: &DatabaseConnection,
    active_ws_connections: Arc<Mutex<HashSet<Uuid>>>,
) -> Result<(), DbErr> {
    let connections = now_playing_connections::Entity::find()
        .filter(now_playing_connections::Column::Enabled.eq(true))
        .all(db)
        .await?;

    for conn in connections {
        if utils::is_ws_connection_type(&conn.connection_type) {
            ensure_ws_listener(db.clone(), conn.clone(), active_ws_connections.clone()).await;
            continue;
        }

        let db = db.clone();
        tokio::spawn(async move {
            if should_poll(&conn) {
                let db_inner = db.clone();
                let conn_inner = conn.clone();
                tokio::spawn(async move {
                    if let Err(e) = utils::poll_connection(&db_inner, &conn_inner).await {
                        tracing::error!("Error polling connection {}: {:?}", conn_inner.id, e);
                    }
                });
            }
        });
    }

    Ok(())
}

fn should_poll(conn: &now_playing_connections::Model) -> bool {
    match conn.last_polled_at {
        None => true,
        Some(last) => {
            let now = chrono::Utc::now().fixed_offset();
            let diff = now.signed_duration_since(last);
            diff.num_seconds() >= conn.poll_interval_seconds as i64
        }
    }
}

async fn ensure_ws_listener(
    db: DatabaseConnection,
    conn: now_playing_connections::Model,
    active_ws_connections: Arc<Mutex<HashSet<Uuid>>>,
) {
    let mut active = active_ws_connections.lock().await;
    if active.contains(&conn.id) {
        return;
    }
    active.insert(conn.id);
    drop(active);

    tokio::spawn(async move {
        if let Err(e) = utils::run_ws_connection(db.clone(), conn.clone()).await {
            tracing::error!("WS connection {} failed: {:?}", conn.id, e);
        }
        let mut active = active_ws_connections.lock().await;
        active.remove(&conn.id);
    });
}
