use sea_orm_migration::prelude::*;

use crate::{
    m20230419_074453_create_comics_table::Comics,
    m20230517_155027_create_comic_genres_table::ComicGenres,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ComicsGenresMapping::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .col(ComicsGenresMapping::ComicId)
                            .col(ComicsGenresMapping::GenreId),
                    )
                    .col(
                        ColumnDef::new(ComicsGenresMapping::ComicId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ComicsGenresMapping::GenreId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(ComicsGenresMapping::ComicId)
                            .to(Comics::Table, Comics::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(ComicsGenresMapping::GenreId)
                            .to(ComicGenres::Table, ComicGenres::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ComicsGenresMapping::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ComicsGenresMapping {
    Table,
    ComicId,
    GenreId,
}
