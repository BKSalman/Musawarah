use sea_orm_migration::prelude::*;

use crate::m20230425_040131_create_comments_table::Comments;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(CommentParentsChildren::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CommentParentsChildren::Id)
                            .uuid()
                            .primary_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CommentParentsChildren::ParentCommentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CommentParentsChildren::ChildCommentId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-comment_parents_children-parent_comment_id")
                            .from(
                                CommentParentsChildren::Table,
                                CommentParentsChildren::ParentCommentId,
                            )
                            .to(Comments::Table, Comments::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-comment_parents_children-child_comment_id")
                            .from(
                                CommentParentsChildren::Table,
                                CommentParentsChildren::ChildCommentId,
                            )
                            .to(Comments::Table, Comments::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(
                Table::drop()
                    .table(CommentParentsChildren::Table)
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum CommentParentsChildren {
    Table,
    ParentCommentId,
    ChildCommentId,
    Id,
}
