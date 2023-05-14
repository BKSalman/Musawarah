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
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Sessions::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Sessions::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Sessions::ExpiresAt).date_time().not_null())
                    .col(ColumnDef::new(Sessions::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Sessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Sessions {
    Table,
    Id,
    UserId,
    CreatedAt,
    ExpiresAt,
}
