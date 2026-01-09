use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "payload_mappings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[schema(value_type = Object)]
    pub mapping_json: Json,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::now_playing_connections::Entity")]
    NowPlayingConnections,
}

impl Related<super::now_playing_connections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NowPlayingConnections.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
