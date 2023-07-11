use std::fs;

use chrono::DateTime;
use derive_builder::Builder;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    comics::models::Comic,
    common::models::ImageResponse,
    schema::{chapter_pages, chapter_ratings, comic_chapters},
    users::models::User,
    utils::average_rating,
    Rating,
};

#[derive(Insertable, Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Comic))]
#[diesel(table_name = comic_chapters)]
pub struct Chapter {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: i32,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub published_at: Option<DateTime<chrono::Utc>>,
    pub is_visible: bool,
    pub user_id: Uuid,
    pub comic_id: Uuid,
}

impl Chapter {
    pub fn into_chapter_response(
        self,
        chapter_pages: &Vec<ChapterPage>,
        chapter_ratings: &Vec<ChapterRating>,
    ) -> ChapterResponse {
        ChapterResponse {
            id: self.id,
            title: self.title,
            number: self.number,
            description: self.description,
            created_at: self.created_at,
            pages: chapter_pages
                .iter()
                .map(|page| ChapterPageResponse {
                    id: page.id,
                    number: page.number,
                    image: ImageResponse {
                        content_type: page.content_type.clone(),
                        path: page.path.clone(),
                    },
                })
                .collect(),
            rating: average_rating(chapter_ratings),
        }
    }

    pub fn into_chapter_response_brief(
        self,
        chapter_pages: &Vec<ChapterPage>,
    ) -> ChapterResponseBrief {
        ChapterResponseBrief {
            id: self.id,
            title: self.title,
            number: self.number,
            description: self.description,
            created_at: self.created_at,
            pages: chapter_pages
                .iter()
                .map(|page| ChapterPageResponse {
                    id: page.id,
                    number: page.number,
                    image: ImageResponse {
                        content_type: page.content_type.clone(),
                        path: page.path.clone(),
                    },
                })
                .collect(),
        }
    }
}

#[derive(Insertable, Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(Comic))]
#[diesel(belongs_to(Chapter))]
#[diesel(belongs_to(User))]
#[diesel(table_name = chapter_pages)]
pub struct ChapterPage {
    pub id: Uuid,
    pub number: i32,
    pub path: String,
    pub content_type: String,
    pub comic_id: Uuid,
    pub chapter_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
}

#[derive(Insertable, Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Chapter))]
#[diesel(table_name = chapter_ratings)]
pub struct ChapterRating {
    pub id: Uuid,
    pub rating: f64,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub user_id: Uuid,
    pub chapter_id: Uuid,
}

impl Rating for ChapterRating {
    fn rating(&self) -> f64 {
        self.rating
    }
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateChapter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: i32,
}

#[derive(AsChangeset, Deserialize, ToSchema, Debug)]
#[diesel(table_name = comic_chapters)]
pub struct UpdateChapter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub number: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub rating: f64,
    pub number: i32,
    pub description: Option<String>,
    pub pages: Vec<ChapterPageResponse>,
    pub created_at: DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterResponseBrief {
    pub id: Uuid,
    pub title: Option<String>,
    pub number: i32,
    pub description: Option<String>,
    pub created_at: DateTime<chrono::Utc>,
    pub pages: Vec<ChapterPageResponse>,
}

#[derive(ToSchema)]
#[allow(dead_code)]
pub struct CreateChapterPage {
    chapter_id: Uuid,
    comic_id: Uuid,
    number: u32,
    #[schema(value_type = String, format = Binary)]
    image: fs::File,
}

#[derive(Builder, Deserialize, ToSchema)]
#[builder(pattern = "owned")]
pub struct ChapterPageData {
    pub chapter_id: Uuid,
    pub comic_id: Uuid,
    pub number: i32,
    pub description: Option<String>,
}

impl ChapterPageData {
    #[must_use]
    pub fn builder() -> ChapterPageDataBuilder {
        ChapterPageDataBuilder::default()
    }
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct ChapterPageResponse {
    pub id: Uuid,
    pub number: i32,
    pub image: ImageResponse,
}

#[derive(garde::Validate, Serialize, Deserialize, ToSchema, TS, Debug)]
#[ts(export)]
pub struct NewChapterRating {
    #[garde(range(min = 0, max = 10))]
    pub rating: i32,
}
