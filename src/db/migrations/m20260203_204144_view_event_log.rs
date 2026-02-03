use sea_orm_migration::{prelude::*, schema::*};

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
                    .table(Views::Table)
                    .if_not_exists()
                    .col(pk_auto(Views::Id))
                    .col(uuid_null(Views::ClientId)) // nullable for historical data
                    .col(float(Views::ProgressIncrease))
                    .col(integer(Views::EditionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Views::Table, Views::EditionId)
                            .to(Edition::Table, Edition::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // migrate old data
        let insert = Query::insert()
            .into_table(Views::Table)
            .columns([Views::ClientId, Views::EditionId, Views::ProgressIncrease])
            .select_from(
                Query::select()
                    .from(Edition::Table)
                    .expr(Expr::cust("null")) // no client ID for historical data
                    .column(Edition::Id)
                    .expr(Expr::column(Edition::Views).cast_as("float"))
                    .and_where(Edition::Views.into_column_ref().gt(0)),
            );
        manager.exec_stmt(insert).await?;

        // drop old data
        manager
            .alter_table(
                Table::alter()
                    .table(Edition::Table)
                    .drop_column(Edition::Views)
                    .to_owned(),
            )
            .await?;

        // create index
        manager
            .create_index(
                Index::create()
                    .table(Views::Table)
                    .name(INDEX_NAME)
                    .col(Views::EditionId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop index
        manager
            .drop_index(
                Index::drop()
                    .table(Views::Table)
                    .name(INDEX_NAME)
                    .to_owned(),
            )
            .await?;

        // add back views column
        manager
            .alter_table(
                Table::alter()
                    .table(Edition::Table)
                    .add_column_if_not_exists(integer(Edition::Views).default(0))
                    .to_owned(),
            )
            .await?;

        let aggregate_views = Query::select()
            .from(Views::Table)
            .column(Views::EditionId)
            .expr(Func::sum(Views::ProgressIncrease).cast_as("integer"))
            .and_where(Views::EditionId.into_column_ref().equals(Edition::Id))
            .to_owned();
        let aggregate_subquery =
            SimpleExpr::SubQuery(None, Box::new(aggregate_views.into_sub_query_statement()));
        let update_editions = Query::update()
            .table(Edition::Table)
            .value(
                Edition::Views,
                Func::coalesce([aggregate_subquery, Expr::value(0)]),
            )
            .to_owned();
        manager.exec_stmt(update_editions).await?;

        // drop table
        manager
            .drop_table(Table::drop().table(Views::Table).to_owned())
            .await
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
enum Edition {
    Table,
    Id,
    Views,
}
