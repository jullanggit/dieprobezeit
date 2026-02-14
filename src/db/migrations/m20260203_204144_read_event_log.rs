//! Add a read event log of (clientId, EditionId, PageNumber, Readtime)

use sea_orm_migration::{prelude::*, schema::*};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

const INDEX_NAME: &str = "index-views-by-edition-id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create table
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

        // create index
        manager
            .create_index(
                Index::create()
                    .table(Reads::Table)
                    .name(INDEX_NAME)
                    .col(Reads::EditionId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop index
        manager
            .drop_index(
                Index::drop()
                    .table(Reads::Table)
                    .name(INDEX_NAME)
                    .to_owned(),
            )
            .await?;

        // drop table
        manager
            .drop_table(Table::drop().table(Reads::Table).to_owned())
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
}

#[derive(DeriveIden)]
enum Edition {
    Table,
    Id,
}
