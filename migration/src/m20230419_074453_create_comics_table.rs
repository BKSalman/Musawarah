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
                    .table(Comics::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Comics::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Comics::Title)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Comics::Description).string().not_null())
                    .col(ColumnDef::new(Comics::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Comics::UpdatedAt).date_time().not_null())
                    .col(ColumnDef::new(Comics::AuthorId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Comics::Table, Comics::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Comics::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Comics {
    Table,
    Id,
    Title,
    Description,
    CreatedAt,
    UpdatedAt,
    AuthorId,
}
