// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "userrole"))]
    pub struct Userrole;
}

diesel::table! {
    chapter_pages (id) {
        id -> Uuid,
        number -> Int4,
        path -> Text,
        content_type -> Text,
        comic_id -> Uuid,
        chapter_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    comic_chapters (id) {
        id -> Uuid,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        number -> Int4,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        user_id -> Uuid,
        comic_id -> Uuid,
    }
}

diesel::table! {
    comic_comments (id) {
        id -> Uuid,
        content -> Text,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        comic_id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    comic_comments_mapping (parent_comment_id, child_comment_id) {
        parent_comment_id -> Uuid,
        child_comment_id -> Uuid,
    }
}

diesel::table! {
    comic_genres (id) {
        id -> Int4,
        name -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    comic_genres_mapping (comic_id, genre_id) {
        comic_id -> Uuid,
        genre_id -> Int4,
    }
}

diesel::table! {
    comics (id) {
        id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        user_id -> Uuid,
    }
}

diesel::table! {
    profile_images (id) {
        id -> Uuid,
        path -> Text,
        content_type -> Text,
        user_id -> Uuid,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        user_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Userrole;

    users (id) {
        id -> Uuid,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        username -> Text,
        displayname -> Text,
        email -> Text,
        phone_number -> Nullable<Text>,
        password -> Text,
        role -> Userrole,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        last_login -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(chapter_pages -> comic_chapters (chapter_id));
diesel::joinable!(chapter_pages -> comics (comic_id));
diesel::joinable!(chapter_pages -> users (user_id));
diesel::joinable!(comic_chapters -> comics (comic_id));
diesel::joinable!(comic_chapters -> users (user_id));
diesel::joinable!(comic_comments -> comics (comic_id));
diesel::joinable!(comic_comments -> users (user_id));
diesel::joinable!(comic_genres_mapping -> comic_genres (genre_id));
diesel::joinable!(comic_genres_mapping -> comics (comic_id));
diesel::joinable!(comics -> users (user_id));
diesel::joinable!(profile_images -> users (user_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    chapter_pages,
    comic_chapters,
    comic_comments,
    comic_comments_mapping,
    comic_genres,
    comic_genres_mapping,
    comics,
    profile_images,
    sessions,
    users,
);