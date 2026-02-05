use sea_orm_migration::{prelude::*, schema::*};

use crate::track_views::NO_ID;

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn helper<'c>(
    manager: &SchemaManager<'c>,
    f: impl Fn(ViewsTmp) -> ColumnDef,
) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ViewsTmp::Table)
                .if_not_exists()
                .col(pk_auto(ViewsTmp::Id))
                .col(f(ViewsTmp::ClientId))
                .col(float(ViewsTmp::ProgressIncrease))
                .col(integer(ViewsTmp::EditionId))
                .foreign_key(
                    ForeignKey::create()
                        .from(ViewsTmp::Table, ViewsTmp::EditionId)
                        .to(Edition::Table, Edition::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    let insert = Query::insert()
        .into_table(ViewsTmp::Table)
        .columns([
            ViewsTmp::Id,
            ViewsTmp::ClientId,
            ViewsTmp::EditionId,
            ViewsTmp::ProgressIncrease,
        ])
        .select_from(
            Query::select()
                .column(Views::Id)
                .expr(Expr::col(Views::ClientId).if_null(NO_ID))
                .column(Views::EditionId)
                .column(Views::ProgressIncrease)
                .from(Views::Table)
                .to_owned(),
        )
        .unwrap()
        .to_owned();
    manager.exec_stmt(insert).await?;

    manager
        .drop_table(Table::drop().table(Views::Table).to_owned())
        .await?;

    manager
        .rename_table(
            Table::rename()
                .table(ViewsTmp::Table, Views::Table)
                .to_owned(),
        )
        .await
}

// Workaround for sqlite not allowing column modifications
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        helper(manager, uuid_null).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        helper(manager, uuid).await
    }
}

#[derive(DeriveIden)]
enum Views {
    Table,
    Id,
    ClientId,
    EditionId,
    ProgressIncrease,
}

#[derive(DeriveIden)]
enum ViewsTmp {
    Table,
    Id,
    ClientId,
    EditionId,
    ProgressIncrease,
}

#[derive(DeriveIden)]
enum Edition {
    Table,
    Id,
}
