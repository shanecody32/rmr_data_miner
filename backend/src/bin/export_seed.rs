use backend::entities::{now_playing_connections, payload_mappings, stations};
use sea_orm::{Database, EntityTrait};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use uuid::Uuid;

#[derive(Serialize)]
struct SeedData {
    stations: Vec<SeedStation>,
    payload_mappings: Vec<SeedMapping>,
    connections: Vec<SeedConnection>,
}

#[derive(Serialize)]
struct SeedStation {
    id: Uuid,
    name: String,
    callsign: Option<String>,
    website_url: Option<String>,
}

#[derive(Serialize)]
struct SeedMapping {
    id: Uuid,
    name: String,
    description: Option<String>,
    mapping_json: Value,
}

#[derive(Serialize)]
struct SeedConnection {
    id: Uuid,
    station_id: Uuid,
    station_name: Option<String>,
    payload_mapping_id: Option<Uuid>,
    payload_mapping_name: Option<String>,
    name: String,
    connection_type: String,
    url: String,
    poll_interval_seconds: i32,
    headers_json: Option<Value>,
    enabled: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let db_url = env::var("EXPORT_DB_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .expect("EXPORT_DB_URL or DATABASE_URL must be set");
    let db = Database::connect(db_url).await?;

    let mut stations_out: Vec<SeedStation> = stations::Entity::find()
        .all(&db)
        .await?
        .into_iter()
        .map(|s| SeedStation {
            id: s.id,
            name: s.name,
            callsign: s.callsign,
            website_url: s.website_url,
        })
        .collect();
    stations_out.sort_by(|a, b| a.name.cmp(&b.name));

    let mut mappings_out: Vec<SeedMapping> = payload_mappings::Entity::find()
        .all(&db)
        .await?
        .into_iter()
        .map(|m| SeedMapping {
            id: m.id,
            name: m.name,
            description: m.description,
            mapping_json: m.mapping_json,
        })
        .collect();
    mappings_out.sort_by(|a, b| a.name.cmp(&b.name));

    let station_names: HashMap<Uuid, String> = stations_out
        .iter()
        .map(|s| (s.id, s.name.clone()))
        .collect();
    let mapping_names: HashMap<Uuid, String> = mappings_out
        .iter()
        .map(|m| (m.id, m.name.clone()))
        .collect();

    let mut connections_out: Vec<SeedConnection> = now_playing_connections::Entity::find()
        .all(&db)
        .await?
        .into_iter()
        .map(|c| SeedConnection {
            id: c.id,
            station_id: c.station_id,
            station_name: station_names.get(&c.station_id).cloned(),
            payload_mapping_id: c.payload_mapping_id,
            payload_mapping_name: c
                .payload_mapping_id
                .and_then(|id| mapping_names.get(&id).cloned()),
            name: c.name,
            connection_type: c.connection_type,
            url: c.url,
            poll_interval_seconds: c.poll_interval_seconds,
            headers_json: c.headers_json,
            enabled: c.enabled,
        })
        .collect();
    connections_out.sort_by(|a, b| a.name.cmp(&b.name));

    let seed_data = SeedData {
        stations: stations_out,
        payload_mappings: mappings_out,
        connections: connections_out,
    };

    let output_path = resolve_output_path();
    let payload = serde_json::to_string_pretty(&seed_data)?;
    fs::write(&output_path, payload)?;
    println!("Seed exported to {}", output_path);

    Ok(())
}

fn resolve_output_path() -> String {
    if let Ok(path) = env::var("SEED_OUTPUT_PATH") {
        return path;
    }

    let primary = "backend/seed_data.json";
    if fs::metadata(primary).is_ok() {
        return primary.to_string();
    }

    "seed_data.json".to_string()
}
