use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    comics::chapters::models::ChapterResponseBrief,
    comics::comic_genres::models::ComicGenre,
    schema::{comic_ratings, comics},
    users::models::{User, UserResponseBrief},
    Rating, SortingOrder,
};

use super::{
    chapters::models::{Chapter, ChapterPage},
    comic_genres::models::Genre,
};

#[derive(Insertable, Queryable, Selectable, Associations, Identifiable, Debug, Clone)]
#[diesel(belongs_to(User))]
#[diesel(table_name = comics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Comic {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub is_visible: bool,
    pub published_at: Option<DateTime<chrono::Utc>>,
    pub poster_path: Option<String>,
    pub poster_content_type: Option<String>,
    pub user_id: Uuid,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ComicResponse {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub rating: f64,
    pub created_at: String,
    pub author: UserResponseBrief,
    pub chapters: Vec<ChapterResponseBrief>,
    pub genres: Vec<ComicGenre>,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ComicResponseBrief {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub rating: f64,
    pub chapters_count: i64,
    pub created_at: String,
    pub genres: Vec<ComicGenre>,
}

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ComicsPagination {
    #[serde(default = "Uuid::max")]
    pub max_id: Uuid,
    #[serde(default)]
    pub order: Order,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ComicsParams {
    #[serde(default)]
    pub genre: Option<i32>,
    #[serde(default)]
    pub sorting: Option<SortingOrder>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Order {
    Latest(DateTime<chrono::Utc>),
    Best(f64),
}

impl Default for Order {
    fn default() -> Self {
        Self::Latest(Utc::now())
    }
}

impl Default for ComicsPagination {
    fn default() -> Self {
        ComicsPagination {
            max_id: Uuid::max(),
            order: Order::default(),
        }
    }
}

impl Comic {
    pub fn into_resonse(
        self,
        user: UserResponseBrief,
        genres: Vec<Genre>,
        chapter_and_pages: Vec<(Chapter, Vec<ChapterPage>)>,
        rating: f64,
    ) -> ComicResponse {
        ComicResponse {
            id: self.id,
            title: self.title,
            slug: self.slug,
            description: self.description,
            created_at: self.created_at.to_string(),
            rating,
            author: user,
            chapters: chapter_and_pages
                .into_iter()
                .map(|(chapter, pages)| chapter.into_response_brief(pages))
                .collect(),
            genres: genres
                .into_iter()
                .map(|genre| ComicGenre {
                    id: genre.id,
                    name: genre.name,
                })
                .collect(),
        }
    }

    pub fn into_response_brief(
        self,
        genres: Vec<Genre>,
        chapters_count: i64,
        rating: f64,
    ) -> ComicResponseBrief {
        ComicResponseBrief {
            id: self.id,
            title: self.title,
            slug: self.slug,
            description: self.description,
            rating,
            chapters_count,
            created_at: self.created_at.to_string(),
            genres: genres
                .into_iter()
                .map(|genre| ComicGenre {
                    id: genre.id,
                    name: genre.name,
                })
                .collect(),
        }
    }
}

#[derive(Insertable, Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Comic))]
#[diesel(table_name = comic_ratings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ComicRating {
    pub id: Uuid,
    pub rating: f64,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub user_id: Uuid,
    pub comic_id: Uuid,
}

impl Rating for ComicRating {
    fn rating(&self) -> f64 {
        self.rating
    }
}

#[derive(Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct CreateComic {
    pub title: String,
    pub description: Option<String>,
    pub genres: Option<Vec<i32>>,
    pub is_visible: bool,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[diesel(table_name = comics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateComic {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(garde::Validate, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct NewComicRating {
    #[garde(range(min = 0, max = 5))]
    pub rating: i32,
}
