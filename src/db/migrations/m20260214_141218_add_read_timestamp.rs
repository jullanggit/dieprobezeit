//! Add a read event log of (clientId, EditionId, PageNumber, Readtime)

use sea_orm_migration::{prelude::*, schema::*};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(Reads::Table, ReadsTmp::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Reads::Table)
                    .if_not_exists()
                    .col(pk_auto(Reads::Id))
                    .col(uuid(Reads::ClientId).default(Uuid::nil()))
                    .col(integer(Reads::EditionId))
                    .col(integer(Reads::PageNumber))
                    .col(float(Reads::ReadTime))
                    .col(timestamp(Reads::Timestamp).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Reads::Table, Reads::EditionId)
                            .to(Edition::Table, Edition::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        let insert = Query::insert()
            .into_table(Reads::Table)
            .columns([
                Reads::Id,
                Reads::ClientId,
                Reads::EditionId,
                Reads::PageNumber,
                Reads::ReadTime,
            ])
            .select_from(
                Query::select()
                    .columns([
                        ReadsTmp::Id,
                        ReadsTmp::ClientId,
                        ReadsTmp::EditionId,
                        ReadsTmp::PageNumber,
                        ReadsTmp::ReadTime,
                    ])
                    .from(ReadsTmp::Table)
                    .to_owned(),
            )
            .unwrap()
            .to_owned();
        manager.exec_stmt(insert).await?;

        manager
            .drop_table(Table::drop().table(ReadsTmp::Table).to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Reads::Table)
                    .drop_column(Reads::Timestamp)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Reads {
    Table,
    Id,
    ClientId,
    EditionId,
    PageNumber,
    ReadTime,
    Timestamp,
}

#[derive(DeriveIden)]
enum ReadsTmp {
    Table,
    Id,
    ClientId,
    EditionId,
    PageNumber,
    ReadTime,
}

#[derive(DeriveIden)]
enum Edition {
    Table,
    Id,
}
