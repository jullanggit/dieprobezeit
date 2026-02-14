//! Mark current edition views as old, so they can later be approximated by read times

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .rename_column(Edition::Views, Edition::OldViews)
                    .add_column(integer(Edition::Views).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::update()
                    .table(Edition::Table)
                    .value(
                        Edition::Views,
                        Edition::Views.into_column_ref().add(Edition::OldViews),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(Table::alter().drop_column(Edition::OldViews).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Edition {
    Table,
    Views,
    OldViews,
}
