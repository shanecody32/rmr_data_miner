pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_tables;
mod m20260108_134400_payload_mappings;
mod m20260108_135100_json_mappings;
mod m20260108_141000_station_mappings;
mod m20260108_142300_remove_station_mapping;
mod m20260109_000100_adaptive_polling;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_tables::Migration),
            Box::new(m20260108_134400_payload_mappings::Migration),
            Box::new(m20260108_135100_json_mappings::Migration),
            Box::new(m20260108_141000_station_mappings::Migration),
            Box::new(m20260108_142300_remove_station_mapping::Migration),
            Box::new(m20260109_000100_adaptive_polling::Migration),
        ]
    }
}
