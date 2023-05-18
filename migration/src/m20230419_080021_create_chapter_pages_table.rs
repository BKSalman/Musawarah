use sea_orm_migration::prelude::*;

use crate::{
    m20220101_000001_create_user_table::Users, m20230419_074453_create_comics_table::Comics,
    m20230419_075009_create_chapters_table::Chapters,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // chapter_id uuid NOT NULL REFERENCES chapters(id),
        // comic_id uuid NOT NULL REFERENCES comics(id),
        // author_id uuid NOT NULL REFERENCES users(id),

        // UNIQUE (chapter_id, number)
        manager
            .create_table(
                Table::create()
                    .table(ChapterPages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChapterPages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChapterPages::Number).integer().not_null())
                    .col(ColumnDef::new(ChapterPages::Path).string().not_null())
                    .col(
                        ColumnDef::new(ChapterPages::ContentType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChapterPages::AuthorId).uuid().not_null())
                    .col(ColumnDef::new(ChapterPages::ComicId).uuid().not_null())
                    .col(ColumnDef::new(ChapterPages::ChapterId).uuid().not_null())
                    .col(
                        ColumnDef::new(ChapterPages::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChapterPages::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChapterPages::Table, ChapterPages::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChapterPages::Table, ChapterPages::ComicId)
                            .to(Comics::Table, Comics::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChapterPages::Table, ChapterPages::ChapterId)
                            .to(Chapters::Table, Chapters::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .col(ChapterPages::ComicId)
                            .col(ChapterPages::Number)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(ChapterPages::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ChapterPages {
    Table,
    Id,
    Number,
    Path,
    ContentType,
    ChapterId,
    AuthorId,
    ComicId,
    UpdatedAt,
    CreatedAt,
}
