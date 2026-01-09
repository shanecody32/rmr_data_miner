use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-station-payload_mapping_id")
                    .table(Stations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Stations::Table)
                    .drop_column(Stations::PayloadMappingId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Stations::Table)
                    .add_column(ColumnDef::new(Stations::PayloadMappingId).uuid())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-station-payload_mapping_id")
                    .from(Stations::Table, Stations::PayloadMappingId)
                    .to(PayloadMappings::Table, PayloadMappings::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Stations {
    Table,
    PayloadMappingId,
}

#[derive(Iden)]
enum PayloadMappings {
    Table,
    Id,
}
