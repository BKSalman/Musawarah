use sea_orm_migration::prelude::*;

use crate::{
    m20220101_000001_create_user_table::Users, m20230419_074453_create_comics_table::Comics,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Comments::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Comments::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Comments::Content).string().not_null())
                    .col(ColumnDef::new(Comments::CommenterId).uuid().not_null())
                    .col(ColumnDef::new(Comments::ComicId).uuid().not_null())
                    .col(ColumnDef::new(Comments::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Comments::UpdatedAt).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-comments-commenter_id")
                            .from(Comments::Table, Comments::CommenterId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-comments-comic_id")
                            .from(Comments::Table, Comments::ComicId)
                            .to(Comics::Table, Comics::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Comments::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Comments {
    Table,
    Id,
    Content,
    ComicId,
    CommenterId,
    UpdatedAt,
    CreatedAt,
}
