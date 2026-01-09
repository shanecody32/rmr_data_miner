use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PayloadMappings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PayloadMappings::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PayloadMappings::Name).string().not_null())
                    .col(ColumnDef::new(PayloadMappings::Description).string())
                    .col(
                        ColumnDef::new(PayloadMappings::ArtistPath)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PayloadMappings::TitlePath)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PayloadMappings::AlbumPath).string())
                    .col(ColumnDef::new(PayloadMappings::ReportedAtPath).string())
                    .col(ColumnDef::new(PayloadMappings::DurationPath).string())
                    .col(ColumnDef::new(PayloadMappings::ListPath).string())
                    .col(
                        ColumnDef::new(PayloadMappings::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PayloadMappings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(NowPlayingConnections::Table)
                    .add_column(ColumnDef::new(NowPlayingConnections::PayloadMappingId).uuid())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-connection-payload_mapping_id")
                    .from(
                        NowPlayingConnections::Table,
                        NowPlayingConnections::PayloadMappingId,
                    )
                    .to(PayloadMappings::Table, PayloadMappings::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-connection-payload_mapping_id")
                    .table(NowPlayingConnections::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(NowPlayingConnections::Table)
                    .drop_column(NowPlayingConnections::PayloadMappingId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(PayloadMappings::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum PayloadMappings {
    Table,
    Id,
    Name,
    Description,
    ArtistPath,
    TitlePath,
    AlbumPath,
    ReportedAtPath,
    DurationPath,
    ListPath,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum NowPlayingConnections {
    Table,
    PayloadMappingId,
}
