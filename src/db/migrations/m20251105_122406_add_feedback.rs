use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Feedback::Table)
                    .if_not_exists()
                    .col(pk_auto(Feedback::Id))
                    .col(string(Feedback::Content))
                    .col(string(Feedback::Email))
                    .col(integer_null(Feedback::EditionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Feedback::Table, Feedback::EditionId)
                            .to(Edition::Table, Edition::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Feedback::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Feedback {
    Table,
    Id,
    Content,
    Email,
    EditionId,
}

#[derive(DeriveIden)]
enum Edition {
    Table,
    Id,
}
