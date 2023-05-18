use crate::m20220101_000001_create_user_table::Users;

use super::m20230419_074453_create_comics_table::Comics;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Chapters::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Chapters::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Chapters::Title).string())
                    .col(ColumnDef::new(Chapters::Description).string())
                    .col(ColumnDef::new(Chapters::Number).integer().not_null())
                    .col(ColumnDef::new(Chapters::AuthorId).uuid().not_null())
                    .col(ColumnDef::new(Chapters::ComicId).uuid().not_null())
                    .col(ColumnDef::new(Chapters::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Chapters::UpdatedAt).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chapters-author_id")
                            .from(Chapters::Table, Chapters::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chapters-comic_id")
                            .from(Chapters::Table, Chapters::ComicId)
                            .to(Comics::Table, Comics::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .col(Chapters::ComicId)
                            .col(Chapters::Number)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Chapters::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Chapters {
    Table,
    Id,
    Description,
    Number,
    ComicId,
    AuthorId,
    CreatedAt,
    UpdatedAt,
    Title,
}
