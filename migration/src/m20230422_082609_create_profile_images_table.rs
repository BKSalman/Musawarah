use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_user_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProfileImages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProfileImages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProfileImages::Path).string().not_null())
                    .col(
                        ColumnDef::new(ProfileImages::ContentType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProfileImages::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-profile_images-user_id")
                            .from(ProfileImages::Table, ProfileImages::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProfileImages::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum ProfileImages {
    Table,
    Id,
    Path,
    ContentType,
    UserId,
}
