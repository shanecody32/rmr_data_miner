use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "raw_now_playing_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub station_id: Uuid,
    pub connection_id: Uuid,
    #[schema(value_type = String)]
    pub observed_at: DateTimeWithTimeZone,
    #[schema(value_type = Option<String>)]
    pub reported_at: Option<DateTimeWithTimeZone>,
    pub reported_artist: Option<String>,
    pub reported_title: Option<String>,
    pub reported_album: Option<String>,
    #[schema(value_type = Object)]
    pub raw_payload: Json,
    pub payload_hash: String,
    pub http_status: Option<i32>,
    pub content_type: Option<String>,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::now_playing_connections::Entity",
        from = "Column::ConnectionId",
        to = "super::now_playing_connections::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    NowPlayingConnections,
    #[sea_orm(
        belongs_to = "super::stations::Entity",
        from = "Column::StationId",
        to = "super::stations::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Stations,
}

impl Related<super::now_playing_connections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NowPlayingConnections.def()
    }
}

impl Related<super::stations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
