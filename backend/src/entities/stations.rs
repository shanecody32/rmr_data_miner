use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "stations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub callsign: Option<String>,
    pub website_url: Option<String>,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::now_playing_connections::Entity")]
    NowPlayingConnections,
    #[sea_orm(has_many = "super::raw_now_playing_events::Entity")]
    RawNowPlayingEvents,
}

impl Related<super::now_playing_connections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NowPlayingConnections.def()
    }
}

impl Related<super::raw_now_playing_events::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RawNowPlayingEvents.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
