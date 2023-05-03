use garde::Validate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::comics::models::ImageResponse;

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

#[derive(Debug, Serialize, Deserialize, ToSchema, TS)]
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
