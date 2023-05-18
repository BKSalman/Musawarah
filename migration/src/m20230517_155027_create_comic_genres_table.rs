use chrono::Utc;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ComicGenres::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ComicGenres::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ComicGenres::Name).string().not_null())
                    .col(ColumnDef::new(ComicGenres::CreateAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        let query = Query::insert()
            .into_table(ComicGenres::Table)
            .columns([ComicGenres::Name, ComicGenres::CreateAt])
            .values_panic(["Arabic".into(), Utc::now().naive_utc().into()])
            .values_panic(["English".into(), Utc::now().naive_utc().into()])
            .values_panic(["Action".into(), Utc::now().naive_utc().into()])
            .values_panic(["Adventure".into(), Utc::now().naive_utc().into()])
            .values_panic(["Horror".into(), Utc::now().naive_utc().into()])
            .values_panic(["Romance".into(), Utc::now().naive_utc().into()])
            .values_panic(["Comedy".into(), Utc::now().naive_utc().into()])
            .to_owned();

        manager.exec_stmt(query).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ComicGenres::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum ComicGenres {
    Table,
    Id,
    Name,
    CreateAt,
}
