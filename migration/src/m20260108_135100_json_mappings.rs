use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PayloadMappings::Table)
                    .add_column(ColumnDef::new(PayloadMappings::MappingJson).json_binary().not_null().default(Expr::value("{}")))
                    .drop_column(PayloadMappings::ArtistPath)
                    .drop_column(PayloadMappings::TitlePath)
                    .drop_column(PayloadMappings::AlbumPath)
                    .drop_column(PayloadMappings::ReportedAtPath)
                    .drop_column(PayloadMappings::DurationPath)
                    .drop_column(PayloadMappings::ListPath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PayloadMappings::Table)
                    .drop_column(PayloadMappings::MappingJson)
                    .add_column(ColumnDef::new(PayloadMappings::ArtistPath).string().not_null().default(""))
                    .add_column(ColumnDef::new(PayloadMappings::TitlePath).string().not_null().default(""))
                    .add_column(ColumnDef::new(PayloadMappings::AlbumPath).string())
                    .add_column(ColumnDef::new(PayloadMappings::ReportedAtPath).string())
                    .add_column(ColumnDef::new(PayloadMappings::DurationPath).string())
                    .add_column(ColumnDef::new(PayloadMappings::ListPath).string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum PayloadMappings {
    Table,
    MappingJson,
    ArtistPath,
    TitlePath,
    AlbumPath,
    ReportedAtPath,
    DurationPath,
    ListPath,
}
