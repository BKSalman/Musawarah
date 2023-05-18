pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_user_table;
mod m20230419_074453_create_comics_table;
mod m20230419_075009_create_chapters_table;
mod m20230419_080021_create_chapter_pages_table;
mod m20230422_082609_create_profile_images_table;
mod m20230425_040131_create_comments_table;
mod m20230425_040303_create_comment_parents_table;
mod m20230426_121507_create_user_roles_table;
mod m20230506_092634_create_session_table;
mod m20230517_155027_create_comic_categories_table;
mod m20230517_160145_create_comics_categories_mapping_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user_table::Migration),
            Box::new(m20230419_074453_create_comics_table::Migration),
            Box::new(m20230419_075009_create_chapters_table::Migration),
            Box::new(m20230419_080021_create_chapter_pages_table::Migration),
            Box::new(m20230422_082609_create_profile_images_table::Migration),
            Box::new(m20230425_040131_create_comments_table::Migration),
            Box::new(m20230425_040303_create_comment_parents_table::Migration),
            Box::new(m20230426_121507_create_user_roles_table::Migration),
            Box::new(m20230506_092634_create_session_table::Migration),
            Box::new(m20230517_155027_create_comic_categories_table::Migration),
            Box::new(m20230517_160145_create_comics_categories_mapping_table::Migration),
        ]
    }
}
