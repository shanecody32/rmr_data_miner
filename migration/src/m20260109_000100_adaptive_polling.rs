use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(NowPlayingConnections::Table)
                    .add_column(
                        ColumnDef::new(NowPlayingConnections::UseDurationPolling)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(ColumnDef::new(NowPlayingConnections::NextPollAt).timestamp_with_time_zone())
                    .add_column(
                        ColumnDef::new(NowPlayingConnections::SameSongBackoffSeconds)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(NowPlayingConnections::ErrorBackoffSeconds)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(NowPlayingConnections::Table)
                    .drop_column(NowPlayingConnections::UseDurationPolling)
                    .drop_column(NowPlayingConnections::NextPollAt)
                    .drop_column(NowPlayingConnections::SameSongBackoffSeconds)
                    .drop_column(NowPlayingConnections::ErrorBackoffSeconds)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum NowPlayingConnections {
    Table,
    UseDurationPolling,
    NextPollAt,
    SameSongBackoffSeconds,
    ErrorBackoffSeconds,
}
