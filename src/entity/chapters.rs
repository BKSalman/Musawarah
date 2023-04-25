//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "chapters")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: i32,
    pub author_id: Uuid,
    pub comic_id: Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::chapter_pages::Entity")]
    ChapterPages,
    #[sea_orm(
        belongs_to = "super::comics::Entity",
        from = "Column::ComicId",
        to = "super::comics::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Comics,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AuthorId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::chapter_pages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChapterPages.def()
    }
}

impl Related<super::comics::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comics.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
