use backend::entities::{now_playing_connections, payload_mappings, stations};
use backend::http_headers::normalize_headers_for_storage;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use uuid::Uuid;

#[derive(Deserialize)]
struct SeedData {
    stations: Vec<SeedStation>,
    payload_mappings: Vec<SeedMapping>,
    connections: Vec<SeedConnection>,
}

#[derive(Deserialize)]
struct SeedStation {
    id: Option<Uuid>,
    name: String,
    callsign: Option<String>,
    website_url: Option<String>,
}

#[derive(Deserialize)]
struct SeedMapping {
    id: Option<Uuid>,
    name: String,
    description: Option<String>,
    mapping_json: Value,
}

#[derive(Deserialize)]
struct SeedConnection {
    id: Option<Uuid>,
    station_id: Option<Uuid>,
    station_name: Option<String>,
    payload_mapping_id: Option<Uuid>,
    payload_mapping_name: Option<String>,
    name: String,
    connection_type: String,
    url: String,
    poll_interval_seconds: Option<i32>,
    headers_json: Option<Value>,
    enabled: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(db_url).await?;

    let seed_path = resolve_seed_path();
    let seed_raw = fs::read_to_string(&seed_path)
        .map_err(|e| format!("Failed to read {}: {}", seed_path, e))?;
    let seed: SeedData = serde_json::from_str(&seed_raw)
        .map_err(|e| format!("Failed to parse {}: {}", seed_path, e))?;

    let now = Utc::now().fixed_offset();
    let mut station_ids: HashMap<String, Uuid> = HashMap::new();
    let mut mapping_ids: HashMap<String, Uuid> = HashMap::new();

    for station in seed.stations {
        let station_entry = upsert_station(&db, station, now).await?;
        station_ids.insert(station_entry.0.clone(), station_entry.1);
    }

    for mapping in seed.payload_mappings {
        let mapping_entry = upsert_mapping(&db, mapping, now).await?;
        mapping_ids.insert(mapping_entry.0.clone(), mapping_entry.1);
    }

    for connection in seed.connections {
        upsert_connection(&db, connection, now, &station_ids, &mapping_ids).await?;
    }

    println!("Seed completed successfully.");
    Ok(())
}

fn resolve_seed_path() -> String {
    if let Ok(path) = env::var("SEED_DATA_PATH") {
        return path;
    }

    let primary = "backend/seed_data.json";
    if fs::metadata(primary).is_ok() {
        return primary.to_string();
    }

    "seed_data.json".to_string()
}

async fn upsert_station(
    db: &sea_orm::DatabaseConnection,
    station: SeedStation,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<(String, Uuid), Box<dyn std::error::Error>> {
    let existing = if let Some(id) = station.id {
        stations::Entity::find_by_id(id).one(db).await?
    } else {
        stations::Entity::find()
            .filter(stations::Column::Name.eq(station.name.clone()))
            .one(db)
            .await?
    };

    let id = if let Some(existing) = existing {
        let mut active: stations::ActiveModel = existing.into();
        active.name = Set(station.name.clone());
        active.callsign = Set(station.callsign);
        active.website_url = Set(station.website_url);
        active.updated_at = Set(now);
        active.update(db).await?.id
    } else {
        let id = station.id.unwrap_or_else(Uuid::new_v4);
        let active = stations::ActiveModel {
            id: Set(id),
            name: Set(station.name.clone()),
            callsign: Set(station.callsign),
            website_url: Set(station.website_url),
            created_at: Set(now),
            updated_at: Set(now),
        };
        active.insert(db).await?.id
    };

    Ok((station.name, id))
}

async fn upsert_mapping(
    db: &sea_orm::DatabaseConnection,
    mapping: SeedMapping,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<(String, Uuid), Box<dyn std::error::Error>> {
    let existing = if let Some(id) = mapping.id {
        payload_mappings::Entity::find_by_id(id).one(db).await?
    } else {
        payload_mappings::Entity::find()
            .filter(payload_mappings::Column::Name.eq(mapping.name.clone()))
            .one(db)
            .await?
    };

    let id = if let Some(existing) = existing {
        let mut active: payload_mappings::ActiveModel = existing.into();
        active.name = Set(mapping.name.clone());
        active.description = Set(mapping.description);
        active.mapping_json = Set(mapping.mapping_json);
        active.updated_at = Set(now);
        active.update(db).await?.id
    } else {
        let id = mapping.id.unwrap_or_else(Uuid::new_v4);
        let active = payload_mappings::ActiveModel {
            id: Set(id),
            name: Set(mapping.name.clone()),
            description: Set(mapping.description),
            mapping_json: Set(mapping.mapping_json),
            created_at: Set(now),
            updated_at: Set(now),
        };
        active.insert(db).await?.id
    };

    Ok((mapping.name, id))
}

async fn upsert_connection(
    db: &sea_orm::DatabaseConnection,
    connection: SeedConnection,
    now: chrono::DateTime<chrono::FixedOffset>,
    station_ids: &HashMap<String, Uuid>,
    mapping_ids: &HashMap<String, Uuid>,
) -> Result<(), Box<dyn std::error::Error>> {
    let station_id = match (connection.station_id, connection.station_name.as_ref()) {
        (Some(id), _) => id,
        (None, Some(name)) => *station_ids
            .get(name)
            .ok_or_else(|| format!("Unknown station_name: {}", name))?,
        (None, None) => return Err("Connection missing station_id or station_name".into()),
    };

    let mapping_id = match (connection.payload_mapping_id, connection.payload_mapping_name.as_ref()) {
        (Some(id), _) => Some(id),
        (None, Some(name)) => Some(
            *mapping_ids
                .get(name)
                .ok_or_else(|| format!("Unknown payload_mapping_name: {}", name))?,
        ),
        (None, None) => None,
    };

    let existing = if let Some(id) = connection.id {
        now_playing_connections::Entity::find_by_id(id).one(db).await?
    } else {
        now_playing_connections::Entity::find()
            .filter(now_playing_connections::Column::StationId.eq(station_id))
            .filter(now_playing_connections::Column::Name.eq(connection.name.clone()))
            .one(db)
            .await?
    };

    let headers_json = normalize_headers_for_storage(
        &connection.connection_type,
        connection.headers_json,
    );
    let poll_interval_seconds = connection.poll_interval_seconds.unwrap_or(30);
    let enabled = connection.enabled.unwrap_or(true);

    if let Some(existing) = existing {
        let mut active: now_playing_connections::ActiveModel = existing.into();
        active.station_id = Set(station_id);
        active.payload_mapping_id = Set(mapping_id);
        active.name = Set(connection.name);
        active.connection_type = Set(connection.connection_type);
        active.url = Set(connection.url);
        active.poll_interval_seconds = Set(poll_interval_seconds);
        active.headers_json = Set(headers_json);
        active.enabled = Set(enabled);
        active.updated_at = Set(now);
        active.update(db).await?;
    } else {
        let id = connection.id.unwrap_or_else(Uuid::new_v4);
        let active = now_playing_connections::ActiveModel {
            id: Set(id),
            station_id: Set(station_id),
            payload_mapping_id: Set(mapping_id),
            name: Set(connection.name),
            connection_type: Set(connection.connection_type),
            url: Set(connection.url),
            poll_interval_seconds: Set(poll_interval_seconds),
            headers_json: Set(headers_json),
            enabled: Set(enabled),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active.insert(db).await?;
    }

    Ok(())
}
