use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "now_playing_connections")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub station_id: Uuid,
    pub payload_mapping_id: Option<Uuid>,
    pub name: String,
    pub connection_type: String,
    pub url: String,
    pub poll_interval_seconds: i32,
    #[schema(value_type = Option<Object>)]
    pub headers_json: Option<Json>,
    pub enabled: bool,
    #[schema(value_type = Option<String>)]
    pub last_polled_at: Option<DateTimeWithTimeZone>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stations::Entity",
        from = "Column::StationId",
        to = "super::stations::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Stations,
    #[sea_orm(
        belongs_to = "super::payload_mappings::Entity",
        from = "Column::PayloadMappingId",
        to = "super::payload_mappings::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    PayloadMappings,
    #[sea_orm(has_many = "super::raw_now_playing_events::Entity")]
    RawNowPlayingEvents,
}

impl Related<super::stations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stations.def()
    }
}

impl Related<super::payload_mappings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PayloadMappings.def()
    }
}

impl Related<super::raw_now_playing_events::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RawNowPlayingEvents.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
