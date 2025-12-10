use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Edition::Table)
                    .add_column_if_not_exists(boolean(Edition::Hidden).default(false))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Edition::Table)
                    .drop_column(Edition::Hidden)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Edition {
    Table,
    Hidden,
}
