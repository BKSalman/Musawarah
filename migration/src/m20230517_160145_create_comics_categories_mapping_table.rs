use sea_orm_migration::prelude::*;

use crate::{
    m20230419_074453_create_comics_table::Comics, m20230517_155027_create_comic_categories_table::ComicCategories,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ComicsCategoriesMapping::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .col(ComicsCategoriesMapping::ComicId)
                            .col(ComicsCategoriesMapping::CategoryId),
                    )
                    .col(
                        ColumnDef::new(ComicsCategoriesMapping::ComicId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ComicsCategoriesMapping::CategoryId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(ComicsCategoriesMapping::ComicId)
                            .to(Comics::Table, Comics::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(ComicsCategoriesMapping::CategoryId)
                            .to(ComicCategories::Table, ComicCategories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ComicsCategoriesMapping::Table)
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ComicsCategoriesMapping {
    Table,
    ComicId,
    CategoryId,
}
