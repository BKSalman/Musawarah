use chrono::Utc;
use sea_orm_migration::prelude::*;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ComicCategories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ComicCategories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ComicCategories::Name).string().not_null())
                    .col(
                        ColumnDef::new(ComicCategories::CreateAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        let query = Query::insert()
            .into_table(ComicCategories::Table)
            .columns([
                ComicCategories::Id,
                ComicCategories::Name,
                ComicCategories::CreateAt,
            ])
            .values_panic([
                Uuid::now_v7().into(),
                "Action".into(),
                Utc::now().naive_utc().into(),
            ])
            .values_panic([
                Uuid::now_v7().into(),
                "Adventure".into(),
                Utc::now().naive_utc().into(),
            ])
            .to_owned();

        manager.exec_stmt(query).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ComicCategories::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum ComicCategories {
    Table,
    Id,
    Name,
    CreateAt,
}
