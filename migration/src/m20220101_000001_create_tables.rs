use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Stations::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Stations::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Stations::Name).string().not_null())
                    .col(ColumnDef::new(Stations::Callsign).string())
                    .col(ColumnDef::new(Stations::WebsiteUrl).string())
                    .col(
                        ColumnDef::new(Stations::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Stations::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(NowPlayingConnections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NowPlayingConnections::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NowPlayingConnections::StationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NowPlayingConnections::Name).string().not_null())
                    .col(
                        ColumnDef::new(NowPlayingConnections::ConnectionType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NowPlayingConnections::Url).string().not_null())
                    .col(
                        ColumnDef::new(NowPlayingConnections::PollIntervalSeconds)
                            .integer()
                            .not_null()
                            .default(30),
                    )
                    .col(ColumnDef::new(NowPlayingConnections::HeadersJson).json_binary())
                    .col(
                        ColumnDef::new(NowPlayingConnections::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(NowPlayingConnections::LastPolledAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(NowPlayingConnections::LastStatus).string())
                    .col(ColumnDef::new(NowPlayingConnections::LastError).string())
                    .col(
                        ColumnDef::new(NowPlayingConnections::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(NowPlayingConnections::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-connection-station_id")
                            .from(NowPlayingConnections::Table, NowPlayingConnections::StationId)
                            .to(Stations::Table, Stations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RawNowPlayingEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RawNowPlayingEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RawNowPlayingEvents::StationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RawNowPlayingEvents::ConnectionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RawNowPlayingEvents::ObservedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RawNowPlayingEvents::ReportedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(RawNowPlayingEvents::ReportedArtist).string())
                    .col(ColumnDef::new(RawNowPlayingEvents::ReportedTitle).string())
                    .col(ColumnDef::new(RawNowPlayingEvents::ReportedAlbum).string())
                    .col(
                        ColumnDef::new(RawNowPlayingEvents::RawPayload)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RawNowPlayingEvents::PayloadHash)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RawNowPlayingEvents::HttpStatus).integer())
                    .col(ColumnDef::new(RawNowPlayingEvents::ContentType).string())
                    .col(
                        ColumnDef::new(RawNowPlayingEvents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-event-station_id")
                            .from(RawNowPlayingEvents::Table, RawNowPlayingEvents::StationId)
                            .to(Stations::Table, Stations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-event-connection_id")
                            .from(RawNowPlayingEvents::Table, RawNowPlayingEvents::ConnectionId)
                            .to(NowPlayingConnections::Table, NowPlayingConnections::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RawNowPlayingEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(NowPlayingConnections::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Stations::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Stations {
    Table,
    Id,
    Name,
    Callsign,
    WebsiteUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum NowPlayingConnections {
    Table,
    Id,
    StationId,
    Name,
    ConnectionType,
    Url,
    PollIntervalSeconds,
    HeadersJson,
    Enabled,
    LastPolledAt,
    LastStatus,
    LastError,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum RawNowPlayingEvents {
    Table,
    Id,
    StationId,
    ConnectionId,
    ObservedAt,
    ReportedAt,
    ReportedArtist,
    ReportedTitle,
    ReportedAlbum,
    RawPayload,
    PayloadHash,
    HttpStatus,
    ContentType,
    CreatedAt,
}
