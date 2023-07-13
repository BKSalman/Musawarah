use std::io::Write;

use chrono::{DateTime, NaiveDateTime};
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
    AsExpression, FromSqlRow,
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    common::models::ImageResponse,
    schema::{profile_images, users},
};
#[derive(Deserialize, Serialize, Debug, AsExpression, FromSqlRow, TS, Copy, Clone, ToSchema)]
#[diesel(sql_type = crate::schema::sql_types::Userrole)]
#[repr(u32)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Staff,
    User,
    VerifiedUser,
}

impl ToSql<crate::schema::sql_types::Userrole, Pg> for UserRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            UserRole::Admin => out.write_all(b"admin"),
            UserRole::Staff => out.write_all(b"staff"),
            UserRole::User => out.write_all(b"user"),
            UserRole::VerifiedUser => out.write_all(b"verified_user"),
        }?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::Userrole, Pg> for UserRole {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"admin" => Ok(UserRole::Admin),
            b"staff" => Ok(UserRole::Staff),
            b"user" => Ok(UserRole::User),
            b"verified_user" => Ok(UserRole::VerifiedUser),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Insertable, Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub displayname: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub bio: Option<String>,
    pub password: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub last_login: Option<NaiveDateTime>,
}

#[derive(Insertable, Queryable, Identifiable, Associations, Selectable, Debug)]
#[diesel(belongs_to(User))]
#[diesel(table_name = profile_images)]
pub struct ProfileImage {
    pub id: Uuid,
    pub path: String,
    pub content_type: String,
    pub user_id: Uuid,
    pub updated_at: Option<DateTime<chrono::Utc>>,
}

#[derive(Validate, Deserialize, ToSchema, TS)]
pub struct CreateUser {
    #[garde(length(min = 5, max = 60))]
    pub username: String,
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Serialize, ToSchema, TS)]
pub struct CreateUserReponse {
    pub user_id: Uuid,
}

#[derive(Validate, Deserialize, ToSchema, TS)]
pub struct UserLogin {
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 8))]
    pub password: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct UserResponse {
    pub id: Uuid,
    pub displayname: String,
    pub username: String,
    pub email: String,
    pub profile_image: ImageResponse,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct UserResponseBrief {
    pub id: Uuid,
    pub displayname: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
pub struct UserClaims {
    pub user: UserResponse,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct UserToken {
    pub access_token: String,
    pub r#type: String,
}
