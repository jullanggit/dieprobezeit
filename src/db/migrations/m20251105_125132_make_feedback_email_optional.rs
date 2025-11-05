use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn helper<'c>(
    manager: &SchemaManager<'c>,
    f: impl Fn(FeedbackTmp) -> ColumnDef,
) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(FeedbackTmp::Table)
                .if_not_exists()
                .col(pk_auto(FeedbackTmp::Id))
                .col(string(FeedbackTmp::Content))
                .col(f(FeedbackTmp::Email))
                .col(integer_null(FeedbackTmp::EditionId))
                .foreign_key(
                    ForeignKey::create()
                        .from(FeedbackTmp::Table, FeedbackTmp::EditionId)
                        .to(Edition::Table, Edition::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    let insert = Query::insert()
        .into_table(FeedbackTmp::Table)
        .columns([
            FeedbackTmp::Id,
            FeedbackTmp::Content,
            FeedbackTmp::Email,
            FeedbackTmp::EditionId,
        ])
        .select_from(
            Query::select()
                .columns([
                    Feedback::Id,
                    Feedback::Content,
                    Feedback::Email,
                    Feedback::EditionId,
                ])
                .from(Feedback::Table)
                .to_owned(),
        )
        .unwrap()
        .to_owned();
    manager.exec_stmt(insert).await?;

    manager
        .drop_table(Table::drop().table(Feedback::Table).to_owned())
        .await?;

    manager
        .rename_table(
            Table::rename()
                .table(FeedbackTmp::Table, Feedback::Table)
                .to_owned(),
        )
        .await
}

// Workaround for sqlite not allowing column modifications
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        helper(manager, string_null).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        helper(manager, string).await
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
enum FeedbackTmp {
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
